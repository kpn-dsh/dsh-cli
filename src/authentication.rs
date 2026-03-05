use crate::cipher::{decrypt, encrypt};
use crate::context::{BrowserMethod, Context};
use crate::directory::{delete_refresh_token, read_refresh_token, write_refresh_token};
use crate::error::DshCliError;
use crate::{err, error_map, DshCliResult};
use dsh_api::dsh_jwt::{jwt_into_header_payload_json, DshJwt};
use dsh_api::platform::DshPlatform;
use futures::future::try_join_all;
use futures::FutureExt;
use itertools::Itertools;
use log::{debug, error, log_enabled, trace, warn, Level};
use openidconnect::core::{
  CoreAuthDisplay, CoreAuthPrompt, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreErrorResponseType, CoreGenderClaim, CoreGrantType, CoreJsonWebKey,
  CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreResponseMode, CoreResponseType, CoreRevocableToken, CoreRevocationErrorResponse, CoreSubjectIdentifierType,
  CoreTokenIntrospectionResponse, CoreTokenResponse,
};
use openidconnect::reqwest::blocking::{Client as BlockingClient, ClientBuilder as BlockingClientBuilder};
use openidconnect::reqwest::redirect::Policy;
use openidconnect::{
  AccessToken, AdditionalProviderMetadata, AuthType, Client, ClientId, DeviceAccessTokenRequest, DeviceAuthorizationResponse, DeviceAuthorizationUrl, EmptyAdditionalClaims,
  EmptyExtraDeviceAuthorizationFields, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, OAuth2TokenResponse, ProviderMetadata, RefreshToken, RequestTokenError, Scope,
  StandardErrorResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use tokio::task::spawn_blocking;

#[derive(clap::ValueEnum, Eq, Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum AuthenticationMethod {
  /// Use the robot account to authenticate and authorize
  #[serde(rename = "robot")]
  Robot,
  /// Use single sign on to authenticate and authorize
  #[serde(rename = "sso")]
  #[value(alias("sso"))]
  SingleSignOn,
}

impl TryFrom<&str> for AuthenticationMethod {
  type Error = DshCliError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "robot" => Ok(Self::Robot),
      "sso" | "single-sign-on" => Ok(Self::SingleSignOn),
      _ => err!("invalid authentication method '{}'", value),
    }
  }
}

impl Display for AuthenticationMethod {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Robot => write!(f, "robot"),
      Self::SingleSignOn => write!(f, "single-sign-on"),
    }
  }
}

impl Default for AuthenticationMethod {
  fn default() -> Self {
    Self::Robot
  }
}

/// Get access token using stored refresh token
///
/// # Parameters
/// * `platform`
///
/// Returns
/// * `Ok(Some((access_token, jwt)))` - Access token was successfully obtained.
/// * `Ok(None)` - User needs to log in.
pub(crate) async fn get_access_token(platform: DshPlatform) -> DshCliResult<Option<(String, DshJwt)>> {
  spawn_blocking(move || {
    let issuer_url = IssuerUrl::new(platform.issuer_endpoint().to_string())?;
    let http_client = BlockingClientBuilder::new().redirect(Policy::none()).build()?;
    let provider_metadata: DeviceProviderMetadata = DeviceProviderMetadata::discover(&issuer_url, &http_client)?;
    debug!("provider metadata read from '{}'", &issuer_url);
    match get_access_token_from_stored_refresh_token(&provider_metadata, &platform, &http_client)? {
      Some(access_token) => {
        let access_token_jwt = DshJwt::from_str(access_token.secret())?;
        Ok(Some((access_token.into_secret(), access_token_jwt)))
      }
      None => Ok(None),
    }
  })
  .await?
}

/// Let the user log in
///
/// # Parameters
/// * `platform`
/// * `context`
///
/// Returns
/// * `Ok(())` - Log in was successful.
pub(crate) async fn login(platform: DshPlatform, context: Context) -> DshCliResult<()> {
  context.print_explanation(format!("login to platform {}", platform));
  let issuer_url = IssuerUrl::new(platform.issuer_endpoint().to_string())?;
  spawn_blocking(move || {
    let http_client = BlockingClientBuilder::new().redirect(Policy::none()).build()?;
    let provider_metadata: DeviceProviderMetadata = DeviceProviderMetadata::discover(&issuer_url, &http_client)?;
    match get_access_token_from_stored_refresh_token(&provider_metadata, &platform, &http_client)? {
      Some(access_token) => {
        let access_token_jwt = DshJwt::from_str(access_token.secret())?;
        debug!("get access token from stored refresh token: {}", access_token_jwt);
        trace!("{:#}", access_token_jwt);
        match &access_token_jwt.payload.preferred_username {
          Some(preferred_username) => context.print(format!("you are already logged in as {}", preferred_username)),
          None => context.print("you are already logged in"),
        }
        print_authorizations(&context, &access_token_jwt);
        Ok(())
      }
      None => match authenticate_and_get_access_and_refresh_tokens(&provider_metadata, &platform, &http_client, &context) {
        Ok((access_token, refresh_token)) => {
          let encrypted_refresh_token = encrypt(refresh_token.secret())?;
          let _ = write_refresh_token(&platform, &encrypted_refresh_token);
          let access_token_jwt = DshJwt::from_str(access_token.secret())?;
          match &access_token_jwt.payload.preferred_username {
            Some(preferred_username) => context.print(format!("you are logged in as {}", preferred_username)),
            None => context.print("you are logged in"),
          }
          print_authorizations(&context, &access_token_jwt);
          Ok(())
        }
        Err(error) => err!("could not authenticate and get access token: {}", error),
      },
    }
  })
  .await?
}

fn print_authorizations(context: &Context, access_token_jwt: &DshJwt) {
  match &access_token_jwt.authorized_tenants() {
    Some(authorized_tenants) => {
      if !authorized_tenants.is_empty() {
        context.print(format!("authorized tenants: {}", authorized_tenants.join(", ")));
      } else {
        context.print("you are not authorized for tenants");
      }
    }
    None => context.print_warning("json web token does not provide authorized tenants"),
  }
}

/// Let the user log out
///
/// # Parameters
/// * `platform`
/// * `context`
///
/// Returns
/// * `Ok(())` - Log out was successful.
pub(crate) async fn logout(platform: DshPlatform, context: Context) -> DshCliResult<()> {
  context.print_explanation(format!("logout from platform {}", platform));
  spawn_blocking(move || {
    let http_client = BlockingClientBuilder::new().redirect(Policy::none()).build()?;
    let provider_metadata_response = http_client.get(format!("{}/.well-known/openid-configuration", platform.issuer_endpoint())).send()?;
    if provider_metadata_response.status().is_success() {
      let response_body = provider_metadata_response.text()?;
      let json_value = serde_json::from_str::<Value>(&response_body)?;
      match json_value.get("end_session_endpoint") {
        Some(end_session_endpoint) => match end_session_endpoint.as_str() {
          Some(endpoint) => {
            context.open_url(endpoint, format!("logout page for platform {}", platform));
            Ok(())
          }
          None => err!("response from provider metadata request has illegal format"),
        },
        None => err!("provider metadata does not contain a end-session endpoint"),
      }
    } else {
      err!("illegal response from provider metadata request")
    }
  })
  .await?
}

/// Get access tokens for all current authentications
///
/// Returns
/// `Ok(Vec)` of tuples containing
/// * `DshPlatform` - Platform for which the user is currently authenticated.
/// * `DshJwt` - Permissions for this authentication.
pub(crate) async fn get_access_tokens() -> DshCliResult<Vec<(DshPlatform, DshJwt)>> {
  try_join_all(
    DshPlatform::all()
      .iter()
      .map(|platform| get_access_token(platform.clone()).map(|access_token| access_token.map(|jwt| (platform.clone(), jwt)))),
  )
  .await
  .map(|tokens| {
    tokens
      .into_iter()
      .filter_map(|(platform, jwt)| jwt.map(|(_, dsh_jwt)| (platform, dsh_jwt)))
      .collect_vec()
  })
}

fn client_id(platform: &DshPlatform) -> ClientId {
  ClientId::new(format!("{}-cli-device", platform.realm()))
}

/// Get access token using stored refresh token
///
/// * Get and decrypt stored refresh token
/// * Check if it is not expired, delete if it is
/// * Get access token and new refresh token
/// * Replace old refresh token with new refresh token
///
/// # Parameters
/// * `provider_metadata` - Device provider metadata.
/// * `platform` - Platform.
/// * `http_client` - Http client (blocking).
///
/// Returns
/// * `Ok(Some(AccessToken))` - Access token was successfully obtained.
/// * `Ok(None)` - If refresh toked does not exist or is expired. This typically means that
///   the user needs to log in (again).
fn get_access_token_from_stored_refresh_token(
  provider_metadata: &DeviceProviderMetadata,
  platform: &DshPlatform,
  http_client: &BlockingClient,
) -> DshCliResult<Option<AccessToken>> {
  match get_stored_refresh_token(platform)? {
    Some(stored_refresh_token) => {
      let refresh_jwt = DshJwt::from_str(stored_refresh_token.secret())?;
      if refresh_jwt.expired().is_some_and(|expired| expired) {
        delete_refresh_token(platform)?;
        Ok(None)
      } else {
        match get_access_token_and_exchanged_refresh_token(provider_metadata, platform, &stored_refresh_token, http_client)? {
          Some((access_token, new_refresh_token)) => {
            trace_token_payload("access token payload", access_token.secret());
            trace_token_payload("refresh token payload", new_refresh_token.secret());
            let encrypted_refresh_token = encrypt(new_refresh_token.secret())?;
            let _ = write_refresh_token(platform, &encrypted_refresh_token);
            Ok(Some(access_token))
          }
          None => {
            delete_refresh_token(platform)?;

            Ok(None)
          }
        }
      }
    }
    None => Ok(None),
  }
}

fn to_pretty_print(json: &str) -> DshCliResult<String> {
  serde_json::to_string_pretty(&serde_json::from_str::<Value>(json)?).map_err(error_map!("{}"))
}

fn trace_token_payload(kind: &str, token: &str) {
  if log_enabled!(Level::Trace) {
    match jwt_into_header_payload_json(token)
      .map_err(DshCliError::from)
      .and_then(|(_, payload)| to_pretty_print(&payload))
    {
      Ok(json) => trace!("{} -> {}", kind, json),
      Err(error) => error!("could not parse {} ({})", kind, error),
    }
  }
}

/// Get access token and exchanged refresh token
///
/// # Parameters
/// * `provider_metadata` - Device provider metadata.
/// * `platform` - Platform.
/// * `refresh_token` - Previous refresh token which will be replaced by a new one.
/// * `http_client` - Http client (blocking).
///
/// Returns
/// * `Ok(Some((access token, refresh token)))` - Successfully retrieved access token and refresh token.
/// * `Ok(None)` - Refresh token was expired.
fn get_access_token_and_exchanged_refresh_token(
  provider_metadata: &DeviceProviderMetadata,
  platform: &DshPlatform,
  refresh_token: &RefreshToken,
  http_client: &BlockingClient,
) -> DshCliResult<Option<(AccessToken, RefreshToken)>> {
  let openid_connect_client = get_openid_client(provider_metadata, &client_id(platform));
  let refresh_token_request = openid_connect_client.exchange_refresh_token(refresh_token)?;
  match refresh_token_request.request(http_client) {
    Ok(token_response) => match token_response.refresh_token() {
      Some(refresh_token) => Ok(Some((token_response.access_token().clone(), refresh_token.clone()))),
      None => {
        warn!("missing refresh token");
        Ok(None)
      }
    },
    Err(error) => match error {
      RequestTokenError::ServerResponse(_) => Ok(None),
      _ => err!("refresh token request error: {}", error),
    },
  }
}

fn authenticate_and_get_access_and_refresh_tokens(
  provider_metadata: &DeviceProviderMetadata,
  platform: &DshPlatform,
  http_client: &BlockingClient,
  context: &Context,
) -> DshCliResult<(AccessToken, RefreshToken)> {
  let openid_connect_client = get_openid_client(provider_metadata, &client_id(platform));
  let device_authorization_request = openid_connect_client
    .exchange_device_code()
    .add_scope(Scope::new("openid".to_string()))
    .add_scope(Scope::new(format!("manage:{}", platform.realm())))
    .add_scope(Scope::new("dsh_perms".to_string()));
  match device_authorization_request.request(http_client) {
    Ok(device_authorization_response) => {
      open_login_page(&device_authorization_response, platform, context);
      let device_access_token_request: DeviceAccessTokenRequest<CoreTokenResponse, EmptyExtraDeviceAuthorizationFields> =
        openid_connect_client.exchange_device_access_token(&device_authorization_response)?;
      match device_access_token_request.request(http_client, std::thread::sleep, None) {
        Ok(token_response) => {
          if let Some(refresh_token) = token_response.refresh_token() {
            Ok((token_response.access_token().clone(), refresh_token.clone()))
          } else {
            err!("device access token does not contain refresh token")
          }
        }
        Err(error) => err!("authentication request failed: {}", error),
      }
    }
    Err(error) => err!("authentication request failed: {}", error),
  }
}

fn get_openid_client(provider_metadata: &DeviceProviderMetadata, client_id: &ClientId) -> OpenIdConnectClient {
  let device_authorization_endpoint = provider_metadata.additional_metadata().device_authorization_endpoint.clone();
  CoreClient::from_provider_metadata(provider_metadata.clone(), client_id.clone(), None)
    .set_device_authorization_url(device_authorization_endpoint)
    .set_auth_type(AuthType::RequestBody)
}

/// Open login page in the system browser
fn open_login_page(response: &DeviceAuthorizationResponse<EmptyExtraDeviceAuthorizationFields>, platform: &DshPlatform, context: &Context) {
  if context.dry_run() {
    context.print_warning(format!("dry-run mode, login page for platform {} not opened", platform));
  } else {
    match context.browser_method() {
      BrowserMethod::Instruct => {
        context.print_explanation(format!(
          "open login page for platform {} in your browser and enter the provided user code",
          platform
        ));
        context.print(format!("login page: {}", response.verification_uri()));
        context.print(format!("user code: {}", response.user_code().secret()));
      }
      BrowserMethod::Open => match response.verification_uri_complete() {
        Some(verification_uri) => match open::that(verification_uri.secret()) {
          Ok(()) => {
            context.print(format!("opening login page for platform {}", platform));
          }
          Err(_) => {
            context.print_error("could not open your browser");
            context.print_explanation(format!(
              "open login page for platform {} in your browser and enter the provided user code",
              platform
            ));
            context.print(format!("login page: {}", response.verification_uri()));
            context.print(format!("user code: {}", response.user_code().secret()));
          }
        },
        None => unreachable!(),
      },
    }
  }
}

/// Get stored refresh token
///
/// # Parameters
/// * `platform` - Platform for which the token is requested.
///
/// # Returns
/// `Some(RefreshToken)`
/// `None`
pub(crate) fn get_stored_refresh_token(platform: &DshPlatform) -> DshCliResult<Option<RefreshToken>> {
  match read_refresh_token(platform)? {
    Some(encrypted_refresh_token) => match decrypt(encrypted_refresh_token.as_str()) {
      Ok(refresh_token) => {
        debug!("refresh token for platform '{}' found and decrypted", platform);
        Ok(Some(RefreshToken::new(refresh_token)))
      }
      Err(_) => {
        warn!("refresh token for platform '{}' found but could not be decrypted", platform);
        delete_refresh_token(platform)?;
        err!("error decrypting refresh token for platform '{}', token deleted", platform)
      }
    },
    None => {
      debug!("refresh token for platform '{}' not found", platform);
      Ok(None)
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DeviceEndpointProviderMetadata {
  device_authorization_endpoint: DeviceAuthorizationUrl,
}

impl AdditionalProviderMetadata for DeviceEndpointProviderMetadata {}

type DeviceProviderMetadata = ProviderMetadata<
  DeviceEndpointProviderMetadata,
  CoreAuthDisplay,
  CoreClientAuthMethod,
  CoreClaimName,
  CoreClaimType,
  CoreGrantType,
  CoreJweContentEncryptionAlgorithm,
  CoreJweKeyManagementAlgorithm,
  CoreJsonWebKey,
  CoreResponseMode,
  CoreResponseType,
  CoreSubjectIdentifierType,
>;

type OpenIdConnectClient = Client<
  EmptyAdditionalClaims,
  CoreAuthDisplay,
  CoreGenderClaim,
  CoreJweContentEncryptionAlgorithm,
  CoreJsonWebKey,
  CoreAuthPrompt,
  StandardErrorResponse<CoreErrorResponseType>,
  CoreTokenResponse,
  CoreTokenIntrospectionResponse,
  CoreRevocableToken,
  CoreRevocationErrorResponse,
  EndpointSet,
  EndpointSet,
  EndpointNotSet,
  EndpointNotSet,
  EndpointMaybeSet,
  EndpointMaybeSet,
>;
