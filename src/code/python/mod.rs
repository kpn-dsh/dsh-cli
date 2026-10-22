use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::code::{apply_template, example_directory, EXAMPLE_CONSUMER, EXAMPLE_LIST_TOPICS, EXAMPLE_PRODUCER, LANGUAGE_PYTHON};
use crate::context::Context;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

const README_MD_TEMPLATE: &str = include_str!("README.md.template");
const CONSUMER_PY_TEMPLATE: &str = include_str!("consumer.py.template");
const LIST_TOPICS_PY_TEMPLATE: &str = include_str!("list-topics.py.template");
const PRODUCER_PY_TEMPLATE: &str = include_str!("producer.py.template");

pub(crate) fn generate_python_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<Option<String>> {
  let example_directory = example_directory(LANGUAGE_PYTHON, bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let readme_md = apply_template(README_MD_TEMPLATE, "", bundle_configuration, bundle_directory, "    ")?;
  let readme_md_filename = format!("{}/README.md", example_directory);
  fs::write(&readme_md_filename, &readme_md)?;
  context.print_outcome(format!("created file '{}'", readme_md_filename));

  let python_consumer = apply_template(CONSUMER_PY_TEMPLATE, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "    ")?;
  let python_consumer_filename = format!("{}/consumer.py", example_directory);
  fs::write(&python_consumer_filename, &python_consumer)?;
  context.print_outcome(format!("created file '{}'", python_consumer_filename));

  let python_list_topics = apply_template(LIST_TOPICS_PY_TEMPLATE, EXAMPLE_LIST_TOPICS, bundle_configuration, bundle_directory, "    ")?;
  let python_list_topics_filename = format!("{}/list-topics.py", example_directory);
  fs::write(&python_list_topics_filename, &python_list_topics)?;
  context.print_outcome(format!("created file '{}'", python_list_topics_filename));

  let python_producer = apply_template(PRODUCER_PY_TEMPLATE, EXAMPLE_PRODUCER, bundle_configuration, bundle_directory, "    ")?;
  let python_producer_filename = format!("{}/producer.py", example_directory);
  fs::write(&python_producer_filename, &python_producer)?;
  context.print_outcome(format!("created file '{}'", python_producer_filename));

  Ok(Some(example_directory))
}

pub(crate) fn python_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(example_directory(LANGUAGE_PYTHON, bundle_configuration, context))?)
}

pub(crate) fn delete_python_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = example_directory(LANGUAGE_PYTHON, bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted python example directory '{}'", example_directory));
  Ok(())
}
