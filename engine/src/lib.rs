use std::env;

use dsh_rest_api_client::Client;
use dsh_sdk::{Platform, RestTokenFetcher, RestTokenFetcherBuilder};
use lazy_static::lazy_static;

pub mod processor;
pub mod resource;
pub mod topology;

const TRIFONIUS_CONFIG_DIR: &str = "TRIFONIUS_CONFIG_DIR";

const TRIFONIUS_TARGET_TENANT: &str = "TRIFONIUS_TARGET_TENANT";
const TRIFONIUS_TARGET_TENANT_USER: &str = "TRIFONIUS_TARGET_TENANT_USER";
const TRIFONIUS_TARGET_TENANT_SECRET: &str = "TRIFONIUS_TARGET_TENANT_SECRET";
const TRIFONIUS_TARGET_PLATFORM: &str = "TRIFONIUS_TARGET_PLATFORM";

pub struct TargetClientFactory {
  tenant: String,
  user: String,
  client: Client,
  token_fetcher: RestTokenFetcher,
}

lazy_static! {
  pub static ref DEFAULT_TARGET_CLIENT_FACTOR: TargetClientFactory = {
    let tenant_name = get_env(TRIFONIUS_TARGET_TENANT);
    let tenant_user = get_env(TRIFONIUS_TARGET_TENANT_USER);
    let client_secret = get_env(TRIFONIUS_TARGET_TENANT_SECRET);
    let platform_name = get_env(TRIFONIUS_TARGET_PLATFORM);
    let platform = match platform_name.as_str() {
      "nplz" => Platform::NpLz,
      "poc" => Platform::Poc,
      "prod" => Platform::Prod,
      "prodaz" => Platform::ProdAz,
      "prodlz" => Platform::ProdLz,
      _ => panic!("invalid platform specified {}", platform_name),
    };
    TargetClientFactory::create(tenant_name, tenant_user, client_secret, platform).expect("could not create static target client factory")
  };
}

pub struct TargetClient<'a> {
  pub tenant: &'a String,
  pub user: &'a String,
  pub client: &'a Client,
  pub token: String,
}

fn get_env(name: &str) -> String {
  match env::var(name) {
    Ok(value) => value,
    Err(_) => panic!("environment variable {} not set", name),
  }
}

impl TargetClientFactory {
  pub fn create(tenant: String, user: String, client_secret: String, platform: Platform) -> Result<Self, String> {
    match RestTokenFetcherBuilder::new(platform.clone())
      .tenant_name(tenant.clone())
      .client_secret(client_secret)
      .build()
    {
      Ok(token_fetcher) => Ok(TargetClientFactory { tenant, user, client: Client::new(platform.endpoint_rest_api()), token_fetcher }),
      Err(e) => Err(format!("could not create token fetcher ({})", e)),
    }
  }

  pub async fn get(&self) -> Result<TargetClient, String> {
    match self.token_fetcher.get_token().await {
      Ok(token) => Ok(TargetClient { tenant: &self.tenant, user: &self.user, client: &self.client, token }),
      Err(e) => Err(format!("could not create token ({})", e)),
    }
  }
}
