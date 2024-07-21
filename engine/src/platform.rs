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

impl DshPlatform {
  pub fn sdk_platform(&self) -> SdkPlatform {
    match self {
      Self::NpLz => SdkPlatform::NpLz,
      Self::Poc => SdkPlatform::Poc,
      Self::Prod => SdkPlatform::Prod,
      Self::ProdAz => SdkPlatform::ProdAz,
      Self::ProdLz => SdkPlatform::ProdLz,
    }
  }

  pub fn realm(&self) -> String {
    self.sdk_platform().realm().to_string()
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
    self.sdk_platform().endpoint_rest_api().to_string()
  }

  pub fn endpoint_rest_access_token(&self) -> String {
    self.sdk_platform().endpoint_rest_access_token().to_string()
  }
}
