use std::env;

use crate::platform::DshPlatform;

#[derive(Clone, Debug)]
pub struct DshApiTenant {
  name: String,
  user: String,
  platform: DshPlatform,
}

impl DshApiTenant {
  pub fn new(name: String, user: String, platform: DshPlatform) -> Self {
    Self { name, user, platform }
  }

  pub fn platform(&self) -> &DshPlatform {
    &self.platform
  }

  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn user(&self) -> &String {
    &self.user
  }

  pub fn app_domain(&self) -> Option<String> {
    self.platform.app_domain(&self.name)
  }

  pub fn console_url(&self) -> Option<String> {
    self.platform.console_url()
  }

  pub fn dsh_internal_domain(&self) -> Option<String> {
    self.platform.dsh_internal_domain(&self.name)
  }

  pub fn monitoring_url(&self) -> Option<String> {
    self.platform.monitoring_url(&self.name)
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

impl Default for DshApiTenant {
  fn default() -> Self {
    let tenant_name = match env::var("TRIFONIUS_TARGET_TENANT") {
      Ok(value) => value,
      Err(_) => panic!("environment variable TRIFONIUS_TARGET_TENANT not set"),
    };
    let user_env = format!("TRIFONIUS_TARGET_TENANT_{}_USER", tenant_name.to_ascii_uppercase().replace('-', "_"));
    let user = match env::var(&user_env) {
      Ok(value) => value,
      Err(_) => panic!("environment variable {} not set", user_env),
    };
    let platform = DshPlatform::default();
    DshApiTenant::new(tenant_name, user, platform)
  }
}
