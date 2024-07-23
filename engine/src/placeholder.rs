use std::fmt::{Display, Formatter};

const APP_DOMAIN: &str = "APP_DOMAIN";
const CONSOLE_URL: &str = "CONSOLE_URL";
const DSH_INTERNAL_DOMAIN: &str = "DSH_INTERNAL_DOMAIN";
const MONITORING_URL: &str = "MONITORING_URL";
const PLATFORM: &str = "PLATFORM";
const PROCESSOR_ID: &str = "PROCESSOR_ID";
const PUBLIC_VHOSTS_DOMAIN: &str = "PUBLIC_VHOSTS_DOMAIN";
const RANDOM: &str = "RANDOM";
const RANDOM_UUID: &str = "RANDOM_UUID";
const REALM: &str = "REALM";
const _REGISTRY: &str = "REGISTRY"; // TODO
const REST_ACCESS_TOKEN_URL: &str = "REST_ACCESS_TOKEN_URL";
const REST_API_URL: &str = "REST_API_URL";
const SERVICE_ID: &str = "SERVICE_ID";
const SERVICE_NAME: &str = "SERVICE_NAME";
const TENANT: &str = "TENANT";
const USER: &str = "USER";

#[derive(Eq, Hash, PartialEq)]
pub enum PlaceHolder {
  AppDomain,
  ConsoleUrl,
  DshInternalDomain,
  MonitoringUrl,
  Platform,
  ProcessorId,
  PublicVhostsDomain,
  Random,
  RandomUuid,
  Realm,
  RestAccessTokenUrl,
  RestApiUrl,
  ServiceId,
  ServiceName,
  Tenant,
  User,
}

impl Display for PlaceHolder {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      PlaceHolder::AppDomain => write!(f, "{}", APP_DOMAIN),
      PlaceHolder::ConsoleUrl => write!(f, "{}", CONSOLE_URL),
      PlaceHolder::DshInternalDomain => write!(f, "{}", DSH_INTERNAL_DOMAIN),
      PlaceHolder::MonitoringUrl => write!(f, "{}", MONITORING_URL),
      PlaceHolder::Platform => write!(f, "{}", PLATFORM),
      PlaceHolder::ProcessorId => write!(f, "{}", PROCESSOR_ID),
      PlaceHolder::PublicVhostsDomain => write!(f, "{}", PUBLIC_VHOSTS_DOMAIN),
      PlaceHolder::Random => write!(f, "{}", RANDOM),
      PlaceHolder::RandomUuid => write!(f, "{}", RANDOM_UUID),
      PlaceHolder::Realm => write!(f, "{}", REALM),
      PlaceHolder::RestAccessTokenUrl => write!(f, "{}", REST_ACCESS_TOKEN_URL),
      PlaceHolder::RestApiUrl => write!(f, "{}", REST_API_URL),
      PlaceHolder::ServiceId => write!(f, "{}", SERVICE_ID),
      PlaceHolder::ServiceName => write!(f, "{}", SERVICE_NAME),
      PlaceHolder::Tenant => write!(f, "{}", TENANT),
      PlaceHolder::User => write!(f, "{}", USER),
    }
  }
}

impl TryFrom<&str> for PlaceHolder {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      APP_DOMAIN => Ok(PlaceHolder::AppDomain),
      CONSOLE_URL => Ok(PlaceHolder::ConsoleUrl),
      DSH_INTERNAL_DOMAIN => Ok(PlaceHolder::DshInternalDomain),
      MONITORING_URL => Ok(PlaceHolder::MonitoringUrl),
      PLATFORM => Ok(PlaceHolder::Platform),
      PROCESSOR_ID => Ok(PlaceHolder::ProcessorId),
      PUBLIC_VHOSTS_DOMAIN => Ok(PlaceHolder::PublicVhostsDomain),
      RANDOM => Ok(PlaceHolder::Random),
      RANDOM_UUID => Ok(PlaceHolder::RandomUuid),
      REALM => Ok(PlaceHolder::Realm),
      REST_ACCESS_TOKEN_URL => Ok(PlaceHolder::RestAccessTokenUrl),
      REST_API_URL => Ok(PlaceHolder::RestApiUrl),
      SERVICE_ID => Ok(PlaceHolder::ServiceId),
      SERVICE_NAME => Ok(PlaceHolder::ServiceName),
      TENANT => Ok(PlaceHolder::Tenant),
      USER => Ok(PlaceHolder::User),
      unrecognized => Err(format!("unrecognized placeholder '{}'", unrecognized)),
    }
  }
}
