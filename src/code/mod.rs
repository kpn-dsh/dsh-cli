use crate::code::python::{delete_python_example_code, generate_python_example_code, python_example_code_exists};
use crate::code::rust::{delete_rust_example_code, generate_rust_example_code, rust_example_code_exists};
use crate::context::Context;
use crate::proxy_bundles::ProxyCertificateBundleConfig;
use crate::{err, DshCliResult};
use itertools::Itertools;

pub(crate) mod python;
pub(crate) mod rust;

pub(crate) const LANGUAGE_PYTHON: &str = "python";
pub(crate) const LANGUAGE_RUST: &str = "rust";

pub(crate) const EXAMPLE_CONSUMER: &str = "consumer";
pub(crate) const EXAMPLE_PRODUCER: &str = "producer";
pub(crate) const EXAMPLE_LIST_TOPICS: &str = "list-topics";

pub(crate) fn generate_example_code(language: &str, bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<String> {
  match language {
    LANGUAGE_PYTHON => generate_python_example_code(bundle_configuration, bundle_directory, context),
    LANGUAGE_RUST => generate_rust_example_code(bundle_configuration, bundle_directory, context),
    _ => err!("unrecognized language '{}'", language),
  }
}

pub(crate) fn example_code_exists(language: &str, bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  match language {
    LANGUAGE_PYTHON => python_example_code_exists(bundle_configuration, context),
    LANGUAGE_RUST => rust_example_code_exists(bundle_configuration, context),
    _ => err!("unrecognized language '{}'", language),
  }
}

pub(crate) fn delete_example_code(language: &str, bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  match language {
    LANGUAGE_PYTHON => delete_python_example_code(bundle_configuration, context),
    LANGUAGE_RUST => delete_rust_example_code(bundle_configuration, context),
    _ => err!("unrecognized language '{}'", language),
  }
}

const BROKERS_PLACEHOLDER: &str = "{{brokers}}";
const BUNDLE_DIRECTORY_PLACEHOLDER: &str = "{{bundle-directory}}";
const CLIENT_ID_PLACEHOLDER: &str = "{{client-id}}";
const EXAMPLE_PLACEHOLDER: &str = "{{example}}";
const GROUP_ID_PLACEHOLDER: &str = "{{group-id}}";
const PROXY_NAME_PLACEHOLDER: &str = "{{proxy-name}}";
const TENANT_PLACEHOLDER: &str = "{{tenant}}";

fn apply_template(template: &str, example: &str, bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str) -> DshCliResult<String> {
  let brokers = bundle_configuration
    .platform
    .tenant_proxy_bootstrap_servers(
      &bundle_configuration.tenant,
      &bundle_configuration.proxy_name,
      bundle_configuration.vhost_zone.clone(),
      3,
    )?
    .iter()
    .map(|broker| format!("  \"{}\",\n", broker))
    .collect_vec()
    .join("");
  let rust_rs = template.replace(BROKERS_PLACEHOLDER, &brokers);
  let rust_rs = rust_rs.replace(BUNDLE_DIRECTORY_PLACEHOLDER, bundle_directory);
  let rust_rs = rust_rs.replace(CLIENT_ID_PLACEHOLDER, &bundle_configuration.client_id());
  let rust_rs = rust_rs.replace(EXAMPLE_PLACEHOLDER, example);
  let rust_rs = rust_rs.replace(GROUP_ID_PLACEHOLDER, &bundle_configuration.group_id(1));
  let rust_rs = rust_rs.replace(PROXY_NAME_PLACEHOLDER, &bundle_configuration.proxy_name);
  let rust_rs = rust_rs.replace(TENANT_PLACEHOLDER, &bundle_configuration.tenant);
  Ok(rust_rs)
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
