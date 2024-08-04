use std::env;

use crate::platform::DshPlatform;
use crate::Client;
use dsh_sdk::{RestTokenFetcher, RestTokenFetcherBuilder};
use lazy_static::lazy_static;

const TRIFONIUS_TARGET: &str = "TRIFONIUS_TARGET";
const TRIFONIUS_TARGET_TENANT: &str = "TRIFONIUS_TARGET_TENANT";
const TRIFONIUS_TARGET_PLATFORM: &str = "TRIFONIUS_TARGET_PLATFORM";

#[derive(Clone, Debug)]
pub struct TargetTenant {
  pub platform: DshPlatform,
  pub tenant: String,
  pub user: String,
}

#[derive(Debug)]
pub struct TargetClientFactory {
  client: Client,
  token_fetcher: RestTokenFetcher,
  target_tenant: TargetTenant,
}

impl TargetClientFactory {
  pub fn platform(&self) -> &DshPlatform {
    &self.target_tenant.platform
  }

  pub fn target_tenant(&self) -> &TargetTenant {
    &self.target_tenant
  }

  pub fn tenant(&self) -> &str {
    &self.target_tenant.tenant
  }

  pub fn user(&self) -> &str {
    &self.target_tenant.user
  }
}

#[derive(Debug)]
pub struct TargetClient<'a> {
  tenant: &'a TargetTenant,
  client: &'a Client,
  token: String,
}

impl TargetClient<'_> {
  pub fn platform(&self) -> &DshPlatform {
    &self.tenant.platform
  }

  pub fn tenant(&self) -> &str {
    &self.tenant.tenant
  }

  pub fn user(&self) -> &str {
    &self.tenant.user
  }

  pub fn client(&self) -> &Client {
    self.client
  }

  pub fn token(&self) -> &str {
    &self.token
  }
}

lazy_static! {
  pub static ref DEFAULT_TARGET_CLIENT_FACTORY: TargetClientFactory = {
    let tenant = get_env(TRIFONIUS_TARGET_TENANT);
    let tenant_env_name = tenant.to_ascii_uppercase().replace('-', "_");
    let user = get_env(format!("{}_TENANT_{}_USER", TRIFONIUS_TARGET, tenant_env_name).as_str());
    let secret = get_env(format!("{}_TENANT_{}_SECRET", TRIFONIUS_TARGET, tenant_env_name).as_str());
    let platform = DshPlatform::try_from(get_env(TRIFONIUS_TARGET_PLATFORM).as_str()).unwrap();
    let target_tenant = TargetTenant { platform, tenant, user };
    TargetClientFactory::create(target_tenant, secret).expect("could not create static target client factory")
  };
}

impl TargetClientFactory {
  pub fn create(target_tenant: TargetTenant, client_secret: String) -> Result<Self, String> {
    match RestTokenFetcherBuilder::new(target_tenant.platform.sdk_platform())
      .tenant_name(target_tenant.tenant.clone())
      .client_secret(client_secret)
      .build()
    {
      Ok(token_fetcher) => {
        let client = Client::new(target_tenant.platform.endpoint_rest_api().as_str());
        Ok(TargetClientFactory { target_tenant, client, token_fetcher })
      }
      Err(e) => Err(format!("could not create token fetcher ({})", e)),
    }
  }

  pub async fn client(&self) -> Result<TargetClient, String> {
    match self.token_fetcher.get_token().await {
      Ok(token) => Ok(TargetClient { tenant: &self.target_tenant, client: &self.client, token }),
      Err(e) => Err(format!("could not create token ({})", e)),
    }
  }
}

impl TargetTenant {
  pub fn app_domain(&self) -> Option<String> {
    self.platform.app_domain(&self.tenant)
  }

  pub fn console_url(&self) -> Option<String> {
    self.platform.console_url()
  }

  pub fn dsh_internal_domain(&self) -> Option<String> {
    self.platform.dsh_internal_domain(&self.tenant)
  }

  pub fn monitoring_url(&self) -> Option<String> {
    self.platform.monitoring_url(&self.tenant)
  }

  pub fn public_vhosts_domain(&self) -> Option<String> {
    self.platform.public_vhosts_domain()
  }

  pub fn realm(&self) -> String {
    self.platform.realm()
  }

  pub fn endpoint_rest_access_token(&self) -> String {
    self.platform.endpoint_rest_access_token()
  }

  pub fn endpoint_rest_api(&self) -> String {
    self.platform.endpoint_rest_api()
  }
}

fn get_env(name: &str) -> String {
  match env::var(name) {
    Ok(value) => value,
    Err(_) => panic!("environment variable {} not set", name),
  }
}
