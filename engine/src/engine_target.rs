use std::collections::HashMap;

use lazy_static::lazy_static;
use log::info;
use rand::Rng;
use regex::Regex;
use uuid::Uuid;

use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::dsh_api_client_factory::DshApiClientFactory;
use dsh_api::dsh_api_tenant::DshApiTenant;
use dsh_api::platform::DshPlatform;

use crate::placeholder::PlaceHolder;

#[derive(Debug)]
pub struct EngineTarget {
  pub(crate) dsh_api_client_factory: DshApiClientFactory,
}

impl EngineTarget {
  pub fn new() -> EngineTarget {
    EngineTarget::default()
  }

  pub fn create(dsh_api_client_factory: DshApiClientFactory) -> Result<Self, String> {
    Ok(EngineTarget { dsh_api_client_factory })
  }

  pub fn platform(&self) -> &DshPlatform {
    self.dsh_api_client_factory.tenant().platform()
  }

  pub fn tenant(&self) -> &DshApiTenant {
    self.dsh_api_client_factory.tenant()
  }

  pub fn tenant_name(&self) -> &str {
    self.dsh_api_client_factory.tenant().name()
  }

  pub fn user(&self) -> &str {
    self.dsh_api_client_factory.tenant().user()
  }

  pub async fn dsh_api_client(&self) -> Result<DshApiClient, String> {
    self.dsh_api_client_factory.client().await
  }
}

impl Default for EngineTarget {
  fn default() -> Self {
    match EngineTarget::create(DshApiClientFactory::default()) {
      Ok(engine_target) => {
        info!("default engine target for {}@{} created", engine_target.tenant_name(), engine_target.platform());
        engine_target
      }
      Err(error) => panic!("{}", error),
    }
  }
}

lazy_static! {
  pub static ref DEFAULT_ENGINE_TARGET: EngineTarget = EngineTarget::default();
}

pub type TemplateMapping = HashMap<PlaceHolder, String>;

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
  mapping.insert(PlaceHolder::Tenant, tenant.name().clone());
  mapping.insert(PlaceHolder::User, tenant.user().clone());
  mapping
}

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
