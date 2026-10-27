use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::code::javascript::{delete_javascript_example_code, generate_javascript_example_code, javascript_example_code_exists};
use crate::code::json::{delete_json_configuration, generate_json_configuration, json_configuration_exists};
use crate::code::python::{delete_python_example_code, generate_python_example_code, python_example_code_exists};
use crate::code::rust::{delete_rust_example_code, generate_rust_example_code, rust_example_code_exists};
use crate::code::scala3::{delete_scala3_example_code, generate_scala3_example_code, scala3_example_code_exists};
use crate::context::Context;
use crate::error::DshCliError;
use crate::{err, DshCliResult};

pub(crate) mod javascript;
pub(crate) mod json;
pub(crate) mod python;
pub(crate) mod rust;
pub(crate) mod scala3;

pub(crate) const LANGUAGE_JAVASCRIPT: &str = "javascript";
pub(crate) const LANGUAGE_JSON: &str = "json";
pub(crate) const LANGUAGE_PYTHON: &str = "python";
pub(crate) const LANGUAGE_RUST: &str = "rust";
pub(crate) const LANGUAGE_SCALA3: &str = "scala3";

pub(crate) const EXAMPLE_CONSUMER: &str = "consumer";
pub(crate) const EXAMPLE_PRODUCER: &str = "producer";
pub(crate) const EXAMPLE_LIST_TOPICS: &str = "list-topics";

pub(crate) fn generate_example_code(
  language: &str,
  bundle_configuration: &ProxyCertificateBundleConfig,
  bundle_directory: &str,
  context: &Context,
) -> DshCliResult<Option<String>> {
  match language {
    LANGUAGE_JAVASCRIPT => generate_javascript_example_code(bundle_configuration, bundle_directory, context),
    LANGUAGE_JSON => generate_json_configuration(bundle_configuration, bundle_directory, context),
    LANGUAGE_PYTHON => generate_python_example_code(bundle_configuration, bundle_directory, context),
    LANGUAGE_RUST => generate_rust_example_code(bundle_configuration, bundle_directory, context),
    LANGUAGE_SCALA3 => generate_scala3_example_code(bundle_configuration, bundle_directory, context),
    _ => err!("unrecognized language '{}'", language),
  }
}

pub(crate) fn example_code_exists(language: &str, bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  match language {
    LANGUAGE_JAVASCRIPT => javascript_example_code_exists(bundle_configuration, context),
    LANGUAGE_JSON => json_configuration_exists(bundle_configuration, context),
    LANGUAGE_PYTHON => python_example_code_exists(bundle_configuration, context),
    LANGUAGE_RUST => rust_example_code_exists(bundle_configuration, context),
    LANGUAGE_SCALA3 => scala3_example_code_exists(bundle_configuration, context),
    _ => err!("unrecognized language '{}'", language),
  }
}

pub(crate) fn delete_example_code(language: &str, bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  match language {
    LANGUAGE_JAVASCRIPT => delete_javascript_example_code(bundle_configuration, context),
    LANGUAGE_JSON => delete_json_configuration(bundle_configuration, context),
    LANGUAGE_PYTHON => delete_python_example_code(bundle_configuration, context),
    LANGUAGE_RUST => delete_rust_example_code(bundle_configuration, context),
    LANGUAGE_SCALA3 => delete_scala3_example_code(bundle_configuration, context),
    _ => err!("unrecognized language '{}'", language),
  }
}

const BROKERS_PLACEHOLDER: &str = "{{brokers}}";
const BUNDLE_DIRECTORY_PLACEHOLDER: &str = "{{bundle-directory}}";
const CLIENT_ID_PLACEHOLDER: &str = "{{client-id}}";
const EXAMPLE_PLACEHOLDER: &str = "{{example}}";
const GROUP_ID_PLACEHOLDER: &str = "{{group-id}}";
const PROXY_NAME_PLACEHOLDER: &str = "{{proxy-name}}";
const PROXY_NAME_UNDERSCORE_PLACEHOLDER: &str = "{{proxy-name-underscore}}";
const SCHEMA_STORE_ENDPOINT_PLACEHOLDER: &str = "{{schema-store-endpoint}}";
const TENANT_PLACEHOLDER: &str = "{{tenant}}";

/// Apply template
///
/// * `{{brokers}}`
/// * `{{bundle-directory}}`
/// * `{{client-id}}`
/// * `{{example}}`
/// * `{{group-id}}`
/// * `{{proxy-name}}`
/// * `{{proxy-name-underscore}}`
/// * `{{schema-store-endpoint}}`
/// * `{{tenant}}`
fn apply_template(template: &str, example: &str, bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, prefix: &str) -> DshCliResult<String> {
  let brokers = match &bundle_configuration.bootstrap_servers {
    Some(brokers) => brokers
      .split(",")
      .collect::<Vec<_>>()
      .into_iter()
      .map(|broker| format!("{}\"{}\"", prefix, broker))
      .collect::<Vec<_>>()
      .join(",\n"),
    None => {
      return Err(DshCliError::Configuration(format!(
        "bundle configuration '{}' does not contain bootstrap servers",
        bundle_configuration.proxy_name
      )))
    }
  };
  let group_id = bundle_configuration.group_id.clone().ok_or(DshCliError::Configuration(format!(
    "bundle configuration '{}' does not contain group id",
    bundle_configuration.proxy_name
  )))?;
  let applied = template.replace(BROKERS_PLACEHOLDER, &brokers);
  let applied = applied.replace(BUNDLE_DIRECTORY_PLACEHOLDER, bundle_directory);
  let applied = applied.replace(CLIENT_ID_PLACEHOLDER, &bundle_configuration.client_id());
  let applied = applied.replace(EXAMPLE_PLACEHOLDER, example);
  let applied = applied.replace(GROUP_ID_PLACEHOLDER, &group_id);
  let applied = applied.replace(PROXY_NAME_PLACEHOLDER, &bundle_configuration.proxy_name);
  let applied = applied.replace(PROXY_NAME_UNDERSCORE_PLACEHOLDER, &bundle_configuration.proxy_name.replace("-", "_"));
  let applied = applied.replace(
    SCHEMA_STORE_ENDPOINT_PLACEHOLDER,
    &bundle_configuration.platform.proxy_schema_store_vhost(
      &bundle_configuration.tenant,
      &bundle_configuration.proxy_name,
      bundle_configuration.vhost_zone.clone(),
    )?,
  );
  let applied = applied.replace(TENANT_PLACEHOLDER, &bundle_configuration.tenant);
  Ok(applied)
}

/// Get example directory name
///
/// `[OUTPUT_DIR]/[PROXY_NAME]-example-[LANGUAGE]`
///
/// ## Example
/// `/Users/username/Workspaces/dsh/dcli/output/my-proxy-example-rust`
///
/// ## Parameters
/// * `language` - Programming language: `python` or `rust`.
/// * `bundle_configuration` - Contains the bundle/proxy configuration.
/// * `context` - DSH tool context.
fn example_directory(language: &str, bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> String {
  let directory_name = format!("{}-example-{}", bundle_configuration.proxy_name, language);
  match context.output_directory() {
    Some(output_directory) => format!("{}/{}", output_directory.display(), directory_name),
    None => directory_name,
  }
}
