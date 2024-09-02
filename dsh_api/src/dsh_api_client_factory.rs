use std::env;

use dsh_sdk::RestTokenFetcherBuilder;
use dsh_sdk::{Platform as SdkPlatform, RestTokenFetcher};

use crate::dsh_api_client::DshApiClient;
use crate::platform::DshPlatform;
use crate::{generated::Client as GeneratedClient, DshApiTenant};

#[derive(Debug)]
pub struct DshApiClientFactory {
  token_fetcher: RestTokenFetcher,
  generated_client: GeneratedClient,
  tenant: DshApiTenant,
}

impl DshApiClientFactory {
  pub fn platform(&self) -> &DshPlatform {
    self.tenant.platform()
  }

  pub fn tenant(&self) -> &DshApiTenant {
    &self.tenant
  }

  pub fn tenant_name(&self) -> &str {
    self.tenant.name()
  }

  pub fn user(&self) -> &str {
    self.tenant.user()
  }

  pub fn create(tenant: DshApiTenant, secret: String) -> Result<Self, String> {
    match RestTokenFetcherBuilder::new(SdkPlatform::from(tenant.platform()))
      .tenant_name(tenant.name().clone())
      .client_secret(secret)
      .build()
    {
      Ok(token_fetcher) => {
        let generated_client = GeneratedClient::new(tenant.platform().endpoint_rest_api().as_str());
        Ok(DshApiClientFactory { token_fetcher, generated_client, tenant })
      }
      Err(e) => Err(format!("could not create token fetcher ({})", e)),
    }
  }

  pub async fn client(&self) -> Result<DshApiClient, String> {
    match self.token_fetcher.get_token().await {
      Ok(token) => Ok(DshApiClient::new(token, &self.generated_client, &self.tenant)),
      Err(e) => Err(format!("could not create token ({})", e)),
    }
  }
}

impl Default for DshApiClientFactory {
  fn default() -> Self {
    let tenant = DshApiTenant::default();
    let secret_env = format!("TRIFONIUS_TARGET_TENANT_{}_SECRET", tenant.name().to_ascii_uppercase().replace('-', "_"));
    let secret = match env::var(&secret_env) {
      Ok(value) => value,
      Err(_) => panic!("environment variable {} not set", secret_env),
    };
    match Self::create(tenant, secret) {
      Ok(factory) => factory,
      Err(error) => panic!("{}", error),
    }
  }
}
