use crate::context::{BrowserMethod, Context};
use crate::refresh_token_store::{delete_stored_refresh_token, get_stored_refresh_token, store_refresh_token};
use dsh_api::dsh_jwt::DshJwt;
use dsh_api::platform::DshPlatform;
use log::{debug, trace};
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
use tokio::task::spawn_blocking;

#[derive(clap::ValueEnum, Eq, Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum AuthenticationMethod {
  /// Use the robot account to authenticate and authorize
  #[serde(rename = "robot")]
  Robot,
  /// Use single sign on to authenticate and authorize
  #[serde(rename = "single-sign-on")]
  SingleSignOn,
}

impl TryFrom<&str> for AuthenticationMethod {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "robot" => Ok(Self::Robot),
      "single-sign-on" => Ok(Self::SingleSignOn),
      _ => Err(format!("invalid authentication method '{}'", value)),
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
/// * `Ok(Some(access_token))` - Access token was successfully obtained.
/// * `Ok(None)` - User needs to log in.
/// * `Err(String)` - Something went wrong.
pub(crate) async fn get_access_token(platform: DshPlatform) -> Result<Option<DshJwt>, String> {
  spawn_blocking(move || {
    let issuer_url = IssuerUrl::new(platform.issuer_endpoint().to_string()).map_err(|error| error.to_string()).unwrap();
    let http_client = BlockingClientBuilder::new().redirect(Policy::none()).build().unwrap();
    let provider_metadata: DeviceProviderMetadata = DeviceProviderMetadata::discover(&issuer_url, &http_client).map_err(|e| e.to_string()).unwrap();
    match get_access_token_from_stored_refresh_token(&provider_metadata, &platform, &http_client) {
      Ok(Some(access_token)) => {
        let access_token_jwt = DshJwt::from_token(access_token.secret().clone())?;
        debug!("get access token: {}", access_token_jwt);
        trace!("{:#}", access_token_jwt);
        Ok(Some(access_token_jwt))
      }
      Ok(None) => Ok(None),
      Err(error) => Err(error),
    }
  })
  .await
  .unwrap()
}

/// Let the user log in
///
/// # Parameters
/// * `platform`
/// * `context`
///
/// Returns
/// * `Ok(())` - Log in was successful.
/// * `Err(String)` - Something went wrong.
pub(crate) async fn login(platform: DshPlatform, context: Context) -> Result<(), String> {
  context.print_explanation(format!("login to platform {}", platform));
  spawn_blocking(move || {
    let issuer_url = IssuerUrl::new(platform.issuer_endpoint().to_string()).map_err(|error| error.to_string()).unwrap();
    let http_client = BlockingClientBuilder::new().redirect(Policy::none()).build().unwrap();
    let provider_metadata: DeviceProviderMetadata = DeviceProviderMetadata::discover(&issuer_url, &http_client).map_err(|e| e.to_string()).unwrap();
    match get_access_token_from_stored_refresh_token(&provider_metadata, &platform, &http_client) {
      Ok(Some(access_token)) => {
        let access_token_jwt = DshJwt::from_token(access_token.secret().clone()).map_err(|e| e.to_string()).unwrap();
        debug!("get access token from stored refresh token: {}", access_token_jwt);
        trace!("{:#}", access_token_jwt);
        match &access_token_jwt.payload.preferred_username {
          Some(preferred_username) => context.print(format!("you are already logged in as {}", preferred_username)),
          None => context.print("you are already logged in"),
        }
        if !&access_token_jwt.tenant_permissions.is_empty() {
          context.print(format!("authorized tenants: {}", access_token_jwt.authorized_tenants().join(", ")));
        } else {
          context.print("you are not authorized for tenants");
        }
      }
      Ok(None) => match authenticate_and_get_access_and_refresh_tokens(&provider_metadata, &platform, &http_client, &context) {
        Ok((access_token, refresh_token)) => {
          let _ = store_refresh_token(&refresh_token, &platform);
          let access_token_jwt = DshJwt::from_token(access_token.secret().to_string()).unwrap();
          match &access_token_jwt.payload.preferred_username {
            Some(preferred_username) => context.print(format!("you are logged in as {}", preferred_username)),
            None => context.print("you are already logged in"),
          }
          if !&access_token_jwt.tenant_permissions.is_empty() {
            context.print(format!("authorized tenants: {}", access_token_jwt.authorized_tenants().join(", ")));
          } else {
            context.print("you are not authorized for tenants");
          }
        }
        Err(_) => {}
      },
      Err(_) => {}
    }
  })
  .await
  .unwrap();
  Ok(())
}

/// Let the user log out
///
/// # Parameters
/// * `platform`
/// * `context`
///
/// Returns
/// * `Ok(())` - Log out was successful.
/// * `Err(String)` - Something went wrong.
pub(crate) async fn logout(platform: &DshPlatform, context: &Context) -> Result<(), String> {
  let url = format!("{}/.well-known/openid-configuration", platform.issuer_endpoint());
  context.print_explanation(format!("logout from platform {}", platform));
  match spawn_blocking(move || {
    let api_response = BlockingClientBuilder::new()
      .redirect(Policy::none())
      .build()
      .and_then(|http_client| http_client.get(url).send())
      .map_err(|_| "")?;
    if api_response.status().is_success() {
      match api_response
        .text()
        .map_err(|_| ())
        .and_then(|body| serde_json::from_str::<Value>(&body).map_err(|_| ()))
        .map_err(|_| "illegal response from provider metadata")?
        .get("end_session_endpoint")
        .and_then(|end_session_endpoint_value| end_session_endpoint_value.as_str())
      {
        Some(endpoint) => Ok(endpoint.to_string()),
        None => Err("provider metadata does not contain end session endpoint"),
      }
    } else {
      Err("illegal response from provider metadata")
    }
  })
  .await
  .unwrap()
  {
    Ok(endpoint) => {
      context.open_url(endpoint, format!("logout page for platform {}", platform));
      Ok(())
    }
    Err(error) => {
      context.print_error(format!("logout error: {}", error));
      Ok(())
    }
  }
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
/// * `Err(String)` - Something went wrong.
fn get_access_token_from_stored_refresh_token(
  provider_metadata: &DeviceProviderMetadata,
  platform: &DshPlatform,
  http_client: &BlockingClient,
) -> Result<Option<AccessToken>, String> {
  match get_stored_refresh_token(platform)? {
    Some(stored_refresh_token) => {
      let refresh_jwt = DshJwt::from_token(stored_refresh_token.secret().clone())?;
      if refresh_jwt.expired() {
        delete_stored_refresh_token(platform)?;
        Ok(None)
      } else {
        match get_access_token_and_exchanged_refresh_token(provider_metadata, platform, &stored_refresh_token, http_client) {
          Ok(Some((access_token, new_refresh_token))) => {
            store_refresh_token(&new_refresh_token, platform)?;
            Ok(Some(access_token))
          }
          Ok(None) => {
            delete_stored_refresh_token(platform)?;
            Ok(None)
          }
          Err(error) => Err(error),
        }
      }
    }
    None => Ok(None),
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
/// * `Err(message)` - Could not process request.
fn get_access_token_and_exchanged_refresh_token(
  provider_metadata: &DeviceProviderMetadata,
  platform: &DshPlatform,
  refresh_token: &RefreshToken,
  http_client: &BlockingClient,
) -> Result<Option<(AccessToken, RefreshToken)>, String> {
  match get_openid_client(provider_metadata, &client_id(platform)).exchange_refresh_token(&refresh_token) {
    Ok(refresh_token_request) => match refresh_token_request.request(http_client).into() {
      Ok(token_response) => match token_response.refresh_token() {
        Some(refresh_token) => Ok(Some((token_response.access_token().clone(), refresh_token.clone()))),
        None => Ok(None),
      },
      Err(error) => match error {
        RequestTokenError::ServerResponse(_) => Ok(None),
        _ => Err(format!("refresh token request error: {}", error)),
      },
    },
    Err(error) => Err(format!("error exchanging token: {}", error)),
  }
}

fn authenticate_and_get_access_and_refresh_tokens(
  provider_metadata: &DeviceProviderMetadata,
  platform: &DshPlatform,
  http_client: &BlockingClient,
  context: &Context,
) -> Result<(AccessToken, RefreshToken), String> {
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
        openid_connect_client.exchange_device_access_token(&device_authorization_response).unwrap();
      match device_access_token_request.request(http_client, std::thread::sleep, None).into() {
        Ok(token_response) => {
          if let Some(refresh_token) = token_response.refresh_token() {
            Ok((token_response.access_token().clone(), refresh_token.clone()))
          } else {
            Err("device access token does not contain refresh token".to_string())
          }
        }
        Err(error) => Err(format!("authentication request failed: {}", error)),
      }
    }
    Err(error) => Err(format!("authentication request failed: {}", error)),
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
      BrowserMethod::Open => match open::that(&response.verification_uri_complete().unwrap().secret()) {
        Ok(()) => {
          context.print_explanation(format!("opening login page for platform {}", platform));
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
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DeviceEndpointProviderMetadata {
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
