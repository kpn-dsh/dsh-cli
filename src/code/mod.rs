use crate::code::python::{delete_python_example_code, generate_python_example_code, python_example_code_exists};
use crate::code::rust::{delete_rust_example_code, generate_rust_example_code, rust_example_code_exists};
use crate::context::Context;
use crate::proxy_bundles::{Language, ProxyCertificateBundleConfig};
use crate::{err, DshCliResult};

pub(crate) mod python;
pub(crate) mod rust;

pub(crate) fn generate_example_code(bundle_configuration: &ProxyCertificateBundleConfig, language: &Language, bundle_directory: &str, context: &Context) -> DshCliResult<String> {
  match language {
    Language::Python => generate_python_example_code(bundle_configuration, bundle_directory, context),
    Language::Rust => generate_rust_example_code(bundle_configuration, bundle_directory, context),
    Language::Golang | Language::Java | Language::Scala => err!("code generation for {} not yet implemented", language),
  }
}

pub(crate) fn example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, language: &Language, context: &Context) -> DshCliResult<bool> {
  match language {
    Language::Python => python_example_code_exists(bundle_configuration, context),
    Language::Rust => rust_example_code_exists(bundle_configuration, context),
    Language::Golang | Language::Java | Language::Scala => err!("code generation for {} not yet implemented", language),
  }
}

pub(crate) fn delete_example_code(bundle_configuration: &ProxyCertificateBundleConfig, language: &Language, context: &Context) -> DshCliResult<()> {
  match language {
    Language::Python => delete_python_example_code(bundle_configuration, context),
    Language::Rust => delete_rust_example_code(bundle_configuration, context),
    Language::Golang | Language::Java | Language::Scala => err!("code generation for {} not yet implemented", language),
  }
}

const BROKERS_PLACEHOLDER: &str = "{{brokers}}";
const BUNDLE_DIRECTORY_PLACEHOLDER: &str = "{{bundle-directory}}";
const CLIENT_ID_PLACEHOLDER: &str = "{{client-id}}";
const GROUP_ID_PLACEHOLDER: &str = "{{group-id}}";
const PROXY_NAME_PLACEHOLDER: &str = "{{proxy-name}}";
const TENANT_PLACEHOLDER: &str = "{{tenant}}";

fn apply_template(template: &str, bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str) -> DshCliResult<String> {
  let brokers = bundle_configuration
    .platform
    .tenant_proxy_bootstrap_servers(
      &bundle_configuration.proxy_name,
      &bundle_configuration.tenant,
      bundle_configuration.vhost_zone.clone(),
      3,
    )?
    .join(",");
  let rust_rs = template.replace(BROKERS_PLACEHOLDER, &brokers);
  let rust_rs = rust_rs.replace(BUNDLE_DIRECTORY_PLACEHOLDER, bundle_directory);
  let rust_rs = rust_rs.replace(CLIENT_ID_PLACEHOLDER, &bundle_configuration.client_id());
  let rust_rs = rust_rs.replace(GROUP_ID_PLACEHOLDER, &bundle_configuration.group_id(0));
  let rust_rs = rust_rs.replace(PROXY_NAME_PLACEHOLDER, &bundle_configuration.proxy_name);
  let rust_rs = rust_rs.replace(TENANT_PLACEHOLDER, &bundle_configuration.tenant);
  Ok(rust_rs)
}
