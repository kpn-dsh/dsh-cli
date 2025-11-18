use crate::context::Context;
use crate::get_target_platform;
use crate::refresh_token_store::{delete_stored_refresh_token, get_stored_refresh_token, store_refresh_token};
use clap::ArgMatches;
use dsh_api::dsh_jwt::DshJwt;
use dsh_api::platform::DshPlatform;
use openidconnect::core::{
  CoreAuthDisplay, CoreAuthPrompt, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreErrorResponseType, CoreGenderClaim, CoreGrantType, CoreJsonWebKey,
  CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreResponseMode, CoreResponseType, CoreRevocableToken, CoreRevocationErrorResponse, CoreSubjectIdentifierType,
  CoreTokenIntrospectionResponse, CoreTokenResponse,
};
use openidconnect::reqwest::blocking::{Client as BlockingClient, ClientBuilder as BlockingClientBuilder};
use openidconnect::reqwest::redirect::Policy;
use openidconnect::{
  AccessToken, AdditionalProviderMetadata, AuthType, Client, ClientId, DeviceAccessTokenRequest, DeviceAuthorizationUrl, EmptyAdditionalClaims,
  EmptyExtraDeviceAuthorizationFields, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, OAuth2TokenResponse, ProviderMetadata, RequestTokenError, Scope,
  StandardErrorResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use tokio::task::spawn_blocking;

#[derive(clap::ValueEnum, Eq, Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub enum AuthenticationMethod {
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

pub async fn login(matches: &ArgMatches, context: Context) -> Result<(), String> {
  let platform = get_target_platform(matches, context.settings())?;
  context.print_explanation(format!("login to platform {}", platform));
  spawn_blocking(move || {
    let issuer_url = IssuerUrl::new(platform.issuer_endpoint().to_string()).map_err(|error| error.to_string()).unwrap();
    let client_id = ClientId::new(format!("{}-cli-device", platform.realm()));
    let http_client = BlockingClientBuilder::new().redirect(Policy::none()).build().unwrap();
    let provider_metadata: DeviceProviderMetadata = DeviceProviderMetadata::discover(&issuer_url, &http_client).map_err(|e| e.to_string()).unwrap();
    match get_access_token(&provider_metadata, &client_id, &platform, &http_client).unwrap() {
      Some(access_token) => {
        let dsh_access_token = DshJwt::from_token(access_token.secret().clone()).map_err(|e| e.to_string()).unwrap();
        match &dsh_access_token.payload.preferred_username {
          Some(preferred_username) => context.print_outcome(format!("you are already logged in as {}", preferred_username)),
          None => context.print_outcome("you are already logged in"),
        }
        if !&dsh_access_token.tenant_permissions.is_empty() {
          context.print_outcome(format!("authorized tenants: {}", dsh_access_token.authorized_tenants().join(", ")));
        } else {
          context.print_outcome("you are not authorized for tenants");
        }
      }
      None => authenticate_and_persist_get_new_refresh_token(&provider_metadata, platform.realm(), &client_id, &platform, &http_client)
        .map_err(|e| e.to_string())
        .unwrap(),
    }
  })
  .await
  .unwrap();
  Ok(())
}

pub async fn logout(matches: &ArgMatches, context: Context) -> Result<(), String> {
  let platform = get_target_platform(matches, context.settings())?;
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
      context.open_url(endpoint, "logout page");
      Ok(())
    }
    Err(error) => {
      context.print_error(format!("logout error: {}", error));
      Ok(())
    }
  }
}

// fn _get_bucket_list(platform: &DshPlatform, tenant: &str, http_client: &ReqwestClient, dsh_access_token: DshJwt) -> Result<(), String> {
//   let api_url = format!("{}/allocation/{}/bucket", platform.rest_api_endpoint(), tenant);
//   println!("calling {} to test access token", api_url);
//   let api_response = http_client
//     .get(api_url)
//     .bearer_auth(dsh_access_token.token.secret())
//     .send()
//     .map_err(|e| e.to_string())?;
//   if api_response.status().is_success() {
//     let body = api_response.text().map_err(|e| e.to_string())?;
//     println!("tenant rest api response:\n{}", body);
//     Ok(())
//   } else {
//     Err(format!(
//       "api call failed: {} {}",
//       api_response.status(),
//       api_response.text().unwrap_or_default()
//     ))
//   }
// }

/// Get access token
///
/// # Parameters
/// * `provider_metadata` - Device provider metadata.
//  * `client_id` - Openid client id.
//  * `platform` - Platform.
//  * `http_client` - Http client (blocking).
fn get_access_token(provider_metadata: &DeviceProviderMetadata, client_id: &ClientId, platform: &DshPlatform, http_client: &BlockingClient) -> Result<Option<AccessToken>, String> {
  match get_stored_refresh_token(platform)? {
    Some(refresh_token) => {
      let dsh_token = DshJwt::from_token(refresh_token.secret().clone())?;
      println!("expires in {}", dsh_token.expires_in());
      println!("{:#?}", dsh_token);

      let openid_connect_client = get_openid_client(provider_metadata, client_id);
      let refresh_token_request = openid_connect_client.exchange_refresh_token(&refresh_token).unwrap();
      match refresh_token_request.request(http_client) {
        Ok(token_response) => {
          match token_response.refresh_token() {
            Some(refresh_token) => {
              println!("refresh token 1");
              println!("{}", refresh_token.secret());
              store_refresh_token(refresh_token, platform)?;
              println!("refresh token was valid, successfully exchanged for new access token");
            }
            None => {
              println!("no refresh token received");
              delete_stored_refresh_token(platform)?
            }
          }
          println!("access token 1");
          println!("{}", token_response.access_token().secret());
          Ok(Some(token_response.access_token().clone()))
        }
        Err(error) => match error {
          RequestTokenError::ServerResponse(_) => {
            println!("refresh token expired");
            delete_stored_refresh_token(platform)?;
            Ok(None)
          }
          _ => {
            println!("refresh token error: {}", error);
            Err(error.to_string())
          }
        },
      }
    }
    None => Ok(None),
  }
}

fn authenticate_and_persist_get_new_refresh_token(
  provider_metadata: &DeviceProviderMetadata,
  realm: &str,
  client_id: &ClientId,
  platform: &DshPlatform,
  http_client: &BlockingClient,
) -> Result<(), String> {
  let openid_connect_client = get_openid_client(provider_metadata, client_id);
  let device_authorization_request = openid_connect_client
    .exchange_device_code()
    .add_scope(Scope::new("openid".to_string()))
    .add_scope(Scope::new(format!("manage:{}", realm)))
    .add_scope(Scope::new("dsh_perms".to_string()));
  match device_authorization_request.request(http_client) {
    Ok(device_authorization_response) => {
      println!("open browser");
      println!("{}", device_authorization_response.user_code().secret());
      println!("{:?}", device_authorization_response.device_code());
      println!("{:?}", device_authorization_response.expires_in());
      println!("{:?}", device_authorization_response.interval());
      println!("{}", device_authorization_response.verification_uri());
      println!("{}", device_authorization_response.verification_uri_complete().unwrap().secret());
      open::that(device_authorization_response.verification_uri_complete().unwrap().secret()).map_err(|error| format!("could not open browser ({})", error))?;
      let device_access_token_request: DeviceAccessTokenRequest<CoreTokenResponse, EmptyExtraDeviceAuthorizationFields> =
        openid_connect_client.exchange_device_access_token(&device_authorization_response).unwrap();
      match device_access_token_request.request(http_client, std::thread::sleep, None) {
        Ok(token_response) => {
          println!("retrieved device access token");
          if let Some(refresh_token) = token_response.refresh_token() {
            println!("refresh token 2");
            println!("{}", refresh_token.secret());
            println!("device access token contains refresh token");
            store_refresh_token(refresh_token, platform)
          } else {
            Err("device access token does not contain refresh token".to_string())
          }
        }
        Err(error) => Err(error.to_string()),
      }
    }
    Err(error) => Err(error.to_string()),
  }
}

fn get_openid_client(provider_metadata: &DeviceProviderMetadata, client_id: &ClientId) -> OpenIdConnectClient {
  let device_authorization_endpoint = provider_metadata.additional_metadata().device_authorization_endpoint.clone();
  CoreClient::from_provider_metadata(provider_metadata.clone(), client_id.clone(), None)
    .set_device_authorization_url(device_authorization_endpoint)
    .set_auth_type(AuthType::RequestBody)
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
