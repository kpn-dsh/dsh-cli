//! Defines DSH platforms and their properties

use std::env;
use std::fmt::{Display, Formatter};

use dsh_sdk::Platform as SdkPlatform;

#[derive(Clone, Debug)]
pub enum DshPlatform {
  NpLz,
  Poc,
  Prod,
  ProdAz,
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

  pub fn console_url(&self) -> Option<String> {
    match self {
      Self::NpLz => Some("https://console.dsh-dev.dsh.np.aws.kpn.com".to_string()),
      Self::Poc => Some("https://console.poc.kpn-dsh.com".to_string()),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some("https://console.dsh-prod.dsh.prod.aws.kpn.com".to_string()),
    }
  }

  pub fn monitoring_url(&self, tenant: &str) -> Option<String> {
    match self {
      Self::NpLz => Some(format!("https://monitoring-{}.dsh-dev.dsh.np.aws.kpn.com", tenant)),
      Self::Poc => Some(format!("https://monitoring-{}.poc.kpn-dsh.com", tenant)),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some(format!("https://monitoring-{}.dsh-prod.dsh.prod.aws.kpn.com", tenant)),
    }
  }

  pub fn public_vhosts_domain(&self) -> Option<String> {
    match self {
      Self::NpLz => Some("dsh-dev.dsh.np.aws.kpn.com".to_string()),
      Self::Poc => Some("poc.kpn-dsh.com".to_string()),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some("dsh-prod.dsh.prod.aws.kpn.com".to_string()),
    }
  }

  pub fn dsh_internal_domain(&self, tenant: &str) -> Option<String> {
    match self {
      Self::NpLz => Some(format!("{}.marathon.mesos", tenant)),
      Self::Poc => Some(format!("{}.marathon.mesos", tenant)),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some(format!("{}.marathon.mesos", tenant)),
    }
  }

  pub fn app_domain(&self, tenant: &str) -> Option<String> {
    match self {
      Self::NpLz => Some(format!("{}.dsh-dev.dsh.np.aws.kpn.com", tenant)),
      Self::Poc => Some(format!("{}.poc.kpn-dsh.com", tenant)),
      Self::Prod => None,
      Self::ProdAz => None,
      Self::ProdLz => Some(format!("{}.dsh-prod.dsh.prod.aws.kpn.com", tenant)),
    }
  }

  pub fn endpoint_rest_api(&self) -> String {
    SdkPlatform::from(self).endpoint_rest_api().to_string()
  }

  pub fn endpoint_rest_access_token(&self) -> String {
    SdkPlatform::from(self).endpoint_rest_access_token().to_string()
  }
}

impl Default for DshPlatform {
  fn default() -> Self {
    match env::var("TRIFONIUS_TARGET_PLATFORM") {
      Ok(platform_name) => match DshPlatform::try_from(platform_name.as_str()) {
        Ok(platform) => platform,
        Err(_) => panic!("invalid platform name {}", platform_name),
      },
      Err(_) => panic!("environment variable TRIFONIUS_TARGET_PLATFORM not set"),
    }
  }
}
