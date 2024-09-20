//! Defines DSH platforms and their properties

use std::env;
use std::fmt::{Display, Formatter};

use dsh_sdk::Platform as SdkPlatform;

use crate::PLATFORM_ENVIRONMENT_VARIABLE;

#[derive(Clone, Debug)]
pub enum DshPlatform {
  /// Test and development landing zone, KPN internal (non-production).
  NpLz,
  /// Proof of concept platform (non-production).
  Poc,
  /// Production platform.
  Prod,
  /// Azure production platform.
  ProdAz,
  /// Production landing zone, KPN internal
  ProdLz,
}

impl Display for DshPlatform {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      DshPlatform::NpLz => write!(f, "nplz"),
      DshPlatform::Poc => write!(f, "poc"),
      DshPlatform::Prod => write!(f, "prod"),
      DshPlatform::ProdAz => write!(f, "prodaz"),
      DshPlatform::ProdLz => write!(f, "prodlz"),
    }
  }
}

impl TryFrom<&str> for DshPlatform {
  type Error = String;

  fn try_from(platform_name: &str) -> Result<Self, Self::Error> {
    match platform_name {
      "nplz" => Ok(DshPlatform::NpLz),
      "poc" => Ok(DshPlatform::Poc),
      "prod" => Ok(DshPlatform::Prod),
      "prodaz" => Ok(DshPlatform::ProdAz),
      "prodlz" => Ok(DshPlatform::ProdLz),
      _ => Err(format!("invalid platform name {}", platform_name)),
    }
  }
}

/// Converts to [`dsh_sdk::Platform`] in DSH rust SDK
impl From<&DshPlatform> for SdkPlatform {
  fn from(dsh_platform: &DshPlatform) -> Self {
    match dsh_platform {
      DshPlatform::NpLz => SdkPlatform::NpLz,
      DshPlatform::Poc => SdkPlatform::Poc,
      DshPlatform::Prod => SdkPlatform::Prod,
      DshPlatform::ProdAz => SdkPlatform::ProdAz,
      DshPlatform::ProdLz => SdkPlatform::ProdLz,
    }
  }
}

impl DshPlatform {
  pub fn realm(&self) -> String {
    SdkPlatform::from(self).realm().to_string()
  }

  /// Returns the url of the platform console
  ///
  /// ## Example
  /// ```rust
  /// use dsh_api::platform::DshPlatform;
  ///
  /// let nplz_platform = DshPlatform::NpLz;
  /// if let Some(console) = nplz_platform.console_url() {
  ///   println!("console url for {} is {}", nplz_platform, console);
  /// }
  /// ```
  pub fn console_url(&self) -> Option<String> {
    match self {
      Self::NpLz => Some("https://console.dsh-dev.dsh.np.aws.kpn.com".to_string()),
      Self::Poc => Some("https://console.poc.kpn-dsh.com".to_string()),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some("https://console.dsh-prod.dsh.prod.aws.kpn.com".to_string()),
    }
  }

  /// Returns the url of the platform/tenant Grafana page
  ///
  /// Returns the url of the tenant's Grafana page for this `DshPlatform`.
  ///
  /// ## Parameters
  /// `tenant` - tenant's name
  ///
  /// ## Example
  /// ```rust
  /// use dsh_api::platform::DshPlatform;
  ///
  /// let tenant = "greenbox-dev";
  /// if let Some(url) = DshPlatform::NpLz.monitoring_url(tenant) {
  ///   println!("monitoring page for {}@nplz is {}", tenant, url);
  /// }
  /// ```
  pub fn monitoring_url(&self, tenant: &str) -> Option<String> {
    match self {
      Self::NpLz => Some(format!("https://monitoring-{}.dsh-dev.dsh.np.aws.kpn.com", tenant)),
      Self::Poc => Some(format!("https://monitoring-{}.poc.kpn-dsh.com", tenant)),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some(format!("https://monitoring-{}.dsh-prod.dsh.prod.aws.kpn.com", tenant)),
    }
  }

  /// Returns the domain used to expose public vhosts
  ///
  /// ## Example
  /// ```rust
  /// use dsh_api::platform::DshPlatform;
  ///
  /// if let Some(vhosts_domain) = DshPlatform::NpLz.public_vhosts_domain() {
  ///   println!("for the eavesdropper, click https://eavesdropper.{}", vhosts_domain);
  /// }
  /// ```
  pub fn public_vhosts_domain(&self) -> Option<String> {
    match self {
      Self::NpLz => Some("dsh-dev.dsh.np.aws.kpn.com".to_string()),
      Self::Poc => Some("poc.kpn-dsh.com".to_string()),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some("dsh-prod.dsh.prod.aws.kpn.com".to_string()),
    }
  }

  /// Returns the domain used to expose private vhosts
  ///
  /// Returns the domain used for the tenant's private vhosts for this `DshPlatform`.
  ///
  /// ## Parameters
  /// `tenant` - tenant's name
  pub fn dsh_internal_domain(&self, tenant: &str) -> Option<String> {
    match self {
      Self::NpLz => Some(format!("{}.marathon.mesos", tenant)),
      Self::Poc => Some(format!("{}.marathon.mesos", tenant)),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some(format!("{}.marathon.mesos", tenant)),
    }
  }

  /// Returns the domain used to expose tenant's apps
  ///
  /// Returns the domain used for the tenant's apps for this `DshPlatform`.
  ///
  /// ## Parameters
  /// `tenant` - tenant's name
  pub fn app_domain(&self, tenant: &str) -> Option<String> {
    match self {
      Self::NpLz => Some(format!("{}.dsh-dev.dsh.np.aws.kpn.com", tenant)),
      Self::Poc => Some(format!("{}.poc.kpn-dsh.com", tenant)),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some(format!("{}.dsh-prod.dsh.prod.aws.kpn.com", tenant)),
    }
  }

  /// Returns the rest api endpoint for this platform
  pub fn endpoint_rest_api(&self) -> String {
    SdkPlatform::from(self).endpoint_rest_api().to_string()
  }

  /// Returns the access token endpoint for this platform
  pub fn endpoint_rest_access_token(&self) -> String {
    SdkPlatform::from(self).endpoint_rest_access_token().to_string()
  }
}

impl TryFrom<()> for DshPlatform {
  type Error = String;

  /// Returns the default platform
  ///
  /// This method will read the value of the environment variable `DSH_API_PLATFORM` and
  /// will select the platform from this value.
  /// It will return an `Error<String>` when the environment variable is not set or
  /// contains an undefined value.
  ///
  /// ## Returns
  /// * `Ok<DshPlatform>` - when the environment variable contains a valid platform name
  /// * `Error<String>` - when the environment variable is not set or
  ///                     contains an undefined value
  fn try_from(_: ()) -> Result<Self, Self::Error> {
    match env::var(PLATFORM_ENVIRONMENT_VARIABLE) {
      Ok(platform_name) => match DshPlatform::try_from(platform_name.as_str()) {
        Ok(platform) => Ok(platform),
        Err(_) => Err(format!(
          "environment variable {} contains invalid platform name {}",
          PLATFORM_ENVIRONMENT_VARIABLE, platform_name
        )),
      },
      Err(_) => Err(format!("environment variable {} not set", PLATFORM_ENVIRONMENT_VARIABLE)),
    }
  }
}

impl Default for DshPlatform {
  /// Returns the default platform
  ///
  /// This method will read the value of the environment variable
  /// `DSH_API_PLATFORM` and
  /// will select the platform from this value.
  ///
  /// ## Panics
  /// This method will panic if the environment variable is not set or
  /// if it contains an invalid platform name.
  fn default() -> Self {
    match Self::try_from(()) {
      Ok(dsh_platform) => dsh_platform,
      Err(error) => panic!("{}", error),
    }
  }
}
