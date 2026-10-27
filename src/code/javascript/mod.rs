use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::code::{apply_template, example_directory, EXAMPLE_CONSUMER, EXAMPLE_LIST_TOPICS, EXAMPLE_PRODUCER, LANGUAGE_JAVASCRIPT};
use crate::context::Context;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

const README_MD_TEMPLATE: &str = include_str!("README.md.template");
const PACKAGE_JSON_TEMPLATE: &str = include_str!("package.json.template");
const CONSUMER_JS_TEMPLATE: &str = include_str!("consumer.js.template");
const LIST_TOPICS_JS_TEMPLATE: &str = include_str!("list-topics.js.template");
const PRODUCER_JS_TEMPLATE: &str = include_str!("producer.js.template");

pub(crate) fn generate_javascript_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<Option<String>> {
  let example_directory = example_directory(LANGUAGE_JAVASCRIPT, bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let readme_md = apply_template(README_MD_TEMPLATE, "", bundle_configuration, bundle_directory, "")?;
  let readme_md_filename = format!("{}/README.md", example_directory);
  fs::write(&readme_md_filename, &readme_md)?;
  context.print_outcome(format!("created file '{}'", readme_md_filename));

  let package_json = apply_template(PACKAGE_JSON_TEMPLATE, "", bundle_configuration, bundle_directory, "")?;
  let package_json_filename = format!("{}/package.json", example_directory);
  fs::write(&package_json_filename, &package_json)?;
  context.print_outcome(format!("created file '{}'", package_json_filename));

  let javascript_consumer = apply_template(CONSUMER_JS_TEMPLATE, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "  ")?;
  let javascript_consumer_filename = format!("{}/{}.js", example_directory, EXAMPLE_CONSUMER);
  fs::write(&javascript_consumer_filename, &javascript_consumer)?;
  context.print_outcome(format!("created file '{}'", javascript_consumer_filename));

  let javascript_list_topics = apply_template(LIST_TOPICS_JS_TEMPLATE, EXAMPLE_LIST_TOPICS, bundle_configuration, bundle_directory, "  ")?;
  let javascript_list_topics_filename = format!("{}/{}.js", example_directory, EXAMPLE_LIST_TOPICS);
  fs::write(&javascript_list_topics_filename, &javascript_list_topics)?;
  context.print_outcome(format!("created file '{}'", javascript_list_topics_filename));

  let javascript_producer = apply_template(PRODUCER_JS_TEMPLATE, EXAMPLE_PRODUCER, bundle_configuration, bundle_directory, "  ")?;
  let javascript_producer_filename = format!("{}/{}.js", example_directory, EXAMPLE_PRODUCER);
  fs::write(&javascript_producer_filename, &javascript_producer)?;
  context.print_outcome(format!("created file '{}'", javascript_producer_filename));

  Ok(Some(example_directory))
}

pub(crate) fn javascript_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(example_directory(LANGUAGE_JAVASCRIPT, bundle_configuration, context))?)
}

pub(crate) fn delete_javascript_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = example_directory(LANGUAGE_JAVASCRIPT, bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted javascript example directory '{}'", example_directory));
  Ok(())
}
