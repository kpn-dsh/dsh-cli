use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::code::apply_template;
use crate::context::Context;
use crate::DshCliResult;
use std::fs;
use std::fs::{exists, remove_file};

const CONFIGURATION_JSON_TEMPLATE: &str = include_str!("configuration.json.template");
const CONFIGURATION_WITH_SCHEMA_JSON_TEMPLATE: &str = include_str!("configuration-with-schema.json.template");

pub(crate) fn generate_json_configuration(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<Option<String>> {
  let json = if bundle_configuration.enable_schema_store {
    apply_template(CONFIGURATION_WITH_SCHEMA_JSON_TEMPLATE, "", bundle_configuration, bundle_directory, "        ")?
  } else {
    apply_template(CONFIGURATION_JSON_TEMPLATE, "", bundle_configuration, bundle_directory, "        ")?
  };
  let json_filename = json_configuration_filename(bundle_configuration, context);
  fs::write(&json_filename, &json)?;
  context.print_outcome(format!("created json configuration file '{}'", json_filename));
  Ok(None)
}

pub(crate) fn json_configuration_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(json_configuration_filename(bundle_configuration, context))?)
}

pub(crate) fn delete_json_configuration(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let json_filename = json_configuration_filename(bundle_configuration, context);
  remove_file(&json_filename)?;
  context.print_outcome(format!("deleted json configuration file '{}'", json_filename));
  Ok(())
}

fn json_configuration_filename(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> String {
  match context.output_directory() {
    Some(output_directory) => format!("{}/{}-configuration.json", output_directory.display(), bundle_configuration.proxy_name),
    None => format!("{}-configuration.json", bundle_configuration.proxy_name),
  }
}
