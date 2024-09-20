use std::env;

use crate::platform::DshPlatform;
use crate::{user_environment_variable, TENANT_ENVIRONMENT_VARIABLE};

#[derive(Clone, Debug)]
pub struct DshApiTenant {
  name: String,
  user: String,
  platform: DshPlatform,
}

impl DshApiTenant {
  /// # Create new dsh api tenant
  ///
  /// ## Parameters
  /// * `name` - tenant's name
  /// * `user` - tenant's group and user id
  /// * `platform` - target platform for the api
  ///
  /// ## Examples
  ///
  /// ```no_run
  /// # use dsh_api::dsh_api_tenant::DshApiTenant;
  /// # use dsh_api::platform::DshPlatform;
  /// let name = String::from("greenbox-dev");
  /// let user = String::from("1903:1903");
  /// let platform = DshPlatform::NpLz;
  /// let dsh_api_tenant = DshApiTenant::new(name, user, platform);
  /// if let Some(domain) = dsh_api_tenant.dsh_internal_domain() {
  ///   assert_eq!(domain, "greenbox-dev.marathon.mesos".to_string())
  /// }
  /// ```
  pub fn new(name: String, user: String, platform: DshPlatform) -> Self {
    Self { name, user, platform }
  }

  /// # Create new dsh api tenant from tenant's name
  ///
  /// This factory function will attempt to create a `DshapiTenant` instance from the provided
  /// tenant's name.
  /// The tenant's group and user ids will be read from the
  /// environment variable `DSH_API_USER_[TENANT]`.
  /// The platform will be read from the
  /// environment variable `DSH_API_PLATFORM`.
  /// The function will return an `Error<String>` if the environment variables are not set
  /// or contains illegal values.
  ///
  /// ## Parameters
  /// * `tenant_name` - tenant's name
  ///
  /// ## Examples
  ///
  /// ```no_run
  /// # use dsh_api::dsh_api_tenant::DshApiTenant;
  /// # use dsh_api::platform::DshPlatform;
  /// # fn main() -> Result<(), String> {
  /// let tenant_name = String::from("greenbox-dev");
  /// let dsh_api_tenant = DshApiTenant::from_tenant(tenant_name)?;
  /// println!("target platform: {}", dsh_api_tenant.platform());
  /// # Ok(())
  /// # }
  /// ```
  pub fn from_tenant(tenant_name: String) -> Result<Self, String> {
    let user_env = user_environment_variable(tenant_name.as_str());
    let user = match env::var(&user_env) {
      Ok(value) => value,
      Err(_) => return Err(format!("environment variable {} not set", user_env)),
    };
    let platform = DshPlatform::default();
    Ok(DshApiTenant::new(tenant_name, user, platform))
  }

  /// # Create new dsh api tenant from tenant's name and platform
  ///
  /// This factory function will attempt to create a `DshapiTenant` instance from the provided
  /// tenant's name. The group and user ids will be read from the
  /// environment variable `DSH_API_USER_[TENANT]`.
  /// The function will return an `Error<String>` if the environment variable is not set.
  ///
  /// ## Parameters
  /// * `tenant_name` - tenant's name
  /// * `platform` - target platform for the api
  ///
  /// ## Examples
  ///
  /// ```no_run
  /// # use dsh_api::dsh_api_tenant::DshApiTenant;
  /// # use dsh_api::platform::DshPlatform;
  /// # fn main() -> Result<(), String> {
  /// let tenant_name = String::from("greenbox-dev");
  /// let platform = DshPlatform::NpLz;
  /// let dsh_api_tenant =
  ///     DshApiTenant::from_tenant_and_platform(tenant_name, platform)?;
  /// println!("{}@{}", dsh_api_tenant.name(), dsh_api_tenant.platform());
  /// # Ok(())
  /// # }
  /// ```
  pub fn from_tenant_and_platform(tenant_name: String, platform: DshPlatform) -> Result<Self, String> {
    let user_env = user_environment_variable(tenant_name.as_str());
    let user = match env::var(&user_env) {
      Ok(value) => value,
      Err(_) => return Err(format!("environment variable {} not set", user_env)),
    };
    Ok(DshApiTenant::new(tenant_name, user, platform))
  }

  /// # Create new dsh api tenant from platform
  ///
  /// This factory function will attempt to create a `DshapiTenant` instance from the provided
  /// `platform`. The tenant's name and group and user ids will be read from the
  /// environment variables `DSH_API_TENANT` and
  /// `DSH_API_USER_[TENANT]`.
  /// The function will return an `Error<String>` if the environment variables are not set.
  ///
  /// ## Parameters
  /// * `platform` - target platform for the api
  ///
  /// ## Examples
  ///
  /// ```no_run
  /// # use dsh_api::dsh_api_tenant::DshApiTenant;
  /// # use dsh_api::platform::DshPlatform;
  /// # fn main() -> Result<(), String> {
  /// let tenant_name = String::from("greenbox-dev");
  /// let platform = DshPlatform::NpLz;
  /// let dsh_api_tenant =
  ///     DshApiTenant::from_tenant_and_platform(tenant_name, platform)?;
  /// println!("{}@{}", dsh_api_tenant.name(), dsh_api_tenant.platform());
  /// # Ok(())
  /// # }
  /// ```
  pub fn from_platform(platform: DshPlatform) -> Result<Self, String> {
    let tenant_name = match env::var(TENANT_ENVIRONMENT_VARIABLE) {
      Ok(name) => name,
      Err(_) => return Err(format!("environment variable {} not set", TENANT_ENVIRONMENT_VARIABLE)),
    };
    let user_env = user_environment_variable(tenant_name.as_str());
    let user = match env::var(&user_env) {
      Ok(value) => value,
      Err(_) => return Err(format!("environment variable {} not set", user_env)),
    };
    Ok(DshApiTenant::new(tenant_name, user, platform))
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
    let tenant_name = match get_default_tenant_name() {
      Ok(name) => name,
      Err(error) => panic!("{}", error),
    };
    let user = match get_user_from_tenant_name(tenant_name.as_str()) {
      Ok(user) => user,
      Err(error) => panic!("{}", error),
    };
    let platform = DshPlatform::default();
    DshApiTenant::new(tenant_name, user, platform)
  }
}

pub fn get_default_tenant_name() -> Result<String, String> {
  env::var(TENANT_ENVIRONMENT_VARIABLE).map_err(|_| format!("environment variable {} not set", TENANT_ENVIRONMENT_VARIABLE))
}

pub fn get_user_from_tenant_name(tenant_name: &str) -> Result<String, String> {
  let user_env = user_environment_variable(tenant_name);
  env::var(&user_env).map_err(|_| format!("environment variable {} not set", user_env))
}
