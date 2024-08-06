use std::collections::HashMap;
use std::env;

use dsh_sdk::{RestTokenFetcher, RestTokenFetcherBuilder};
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use uuid::Uuid;

use trifonius_dsh_api::Client;
use trifonius_dsh_api::DshApiTenant;

use crate::placeholder::PlaceHolder;
use crate::platform::DshPlatform;

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

pub fn from_tenant_to_template_mapping(tenant: &DshApiTenant) -> TemplateMapping {
  let mut mapping = TemplateMapping::new();
  if let Some(app_domain) = tenant.app_domain() {
    mapping.insert(PlaceHolder::AppDomain, app_domain);
  }
  if let Some(console_url) = tenant.console_url() {
    mapping.insert(PlaceHolder::ConsoleUrl, console_url);
  }
  if let Some(dsh_internal_domain) = tenant.dsh_internal_domain() {
    mapping.insert(PlaceHolder::DshInternalDomain, dsh_internal_domain);
  }
  if let Some(monitoring_url) = tenant.monitoring_url() {
    mapping.insert(PlaceHolder::MonitoringUrl, monitoring_url);
  }
  mapping.insert(PlaceHolder::Platform, tenant.platform().to_string());
  if let Some(public_vhosts_domain) = tenant.public_vhosts_domain() {
    mapping.insert(PlaceHolder::PublicVhostsDomain, public_vhosts_domain);
  }
  mapping.insert(PlaceHolder::Random, format!("{:x}", rand::thread_rng().gen_range(0x10000000_u64..=0xffffffff_u64)));
  mapping.insert(PlaceHolder::RandomUuid, Uuid::new_v4().to_string());
  mapping.insert(PlaceHolder::Realm, tenant.realm());
  mapping.insert(PlaceHolder::RestAccessTokenUrl, tenant.endpoint_rest_access_token());
  mapping.insert(PlaceHolder::RestApiUrl, tenant.endpoint_rest_api());
  mapping.insert(PlaceHolder::Tenant, tenant.tenant().clone());
  mapping.insert(PlaceHolder::User, tenant.user().clone());
  mapping
}

fn get_env(name: &str) -> String {
  match env::var(name) {
    Ok(value) => value,
    Err(_) => panic!("environment variable {} not set", name),
  }
}

pub type TemplateMapping = HashMap<PlaceHolder, String>;

lazy_static! {
  static ref TEMPLATE_REGEX: Regex = Regex::new("\\$\\{([A-Z][A-Z0-9_]*)\\}").unwrap();
}

pub(crate) fn template_resolver(template: &str, template_mapping: &TemplateMapping) -> Result<String, String> {
  let mut new = String::with_capacity(template.len());
  let mut last_match = 0;
  for caps in TEMPLATE_REGEX.captures_iter(template) {
    let m = caps.get(0).unwrap();
    new.push_str(&template[last_match..m.start()]);
    let place_holder = PlaceHolder::try_from(caps.get(1).unwrap().as_str())?;
    match template_mapping.get(&place_holder) {
      Some(value) => {
        new.push_str(value);
      }
      None => return Err(format!("template resolution failed because placeholder '{}' has no value", place_holder)),
    }
    last_match = m.end();
  }
  new.push_str(&template[last_match..]);
  Ok(new)
}

pub(crate) fn validate_template(template: &str, template_mapping: &[PlaceHolder]) -> Result<(), String> {
  for caps in TEMPLATE_REGEX.captures_iter(template) {
    let place_holder = PlaceHolder::try_from(caps.get(1).unwrap().as_str())?;
    if !template_mapping.contains(&place_holder) {
      return Err(format!("invalid template because placeholder '{}' is not allowed", place_holder));
    }
  }
  Ok(())
}

#[test]
fn resolve_template_successfully() {
  let template = "abcd${TENANT}def${USER}ghi";
  let tenant = "tenant";
  let user = "user";
  let template_mapping: TemplateMapping = HashMap::from([(PlaceHolder::Tenant, tenant.to_string()), (PlaceHolder::User, user.to_string())]);
  assert_eq!(template_resolver(template, &template_mapping).unwrap(), "abcdtenantdefuserghi");
}

#[test]
fn validate_template_succesfully() {
  assert!(validate_template("abcd${TENANT}def${USER}ghi", &[PlaceHolder::Tenant, PlaceHolder::User]).is_ok());
  assert!(validate_template("abcd${TENANT}def${USER}ghi", &[PlaceHolder::Tenant]).is_err());
  assert!(validate_template("abcd{TENANT}def{USER}ghi", &[PlaceHolder::Tenant]).is_ok());
  assert!(validate_template("abcdefghijkl", &[PlaceHolder::Tenant]).is_ok());
  assert!(validate_template("", &[PlaceHolder::Tenant]).is_ok());
}
