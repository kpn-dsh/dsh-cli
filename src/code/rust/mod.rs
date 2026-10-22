use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::code::{apply_template, example_directory, EXAMPLE_CONSUMER, EXAMPLE_LIST_TOPICS, EXAMPLE_PRODUCER, LANGUAGE_RUST};
use crate::context::Context;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

const README_MD_TEMPLATE: &str = include_str!("README.md.template");
const CARGO_TOML_TEMPLATE: &str = include_str!("Cargo.toml.template");
const MAIN_RS_TEMPLATE: &str = include_str!("main.rs.template");
const CONSUMER_RS_TEMPLATE: &str = include_str!("consumer.rs.template");
const PRODUCER_RS_TEMPLATE: &str = include_str!("producer.rs.template");

pub(crate) fn generate_rust_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<Option<String>> {
  let example_directory = example_directory(LANGUAGE_RUST, bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let src_directory = format!("{}/src", example_directory);
  create_dir_all(&src_directory)?;
  context.print_outcome(format!("created directory '{}'", src_directory));

  let bin_directory = format!("{}/src/bin", example_directory);
  create_dir_all(&bin_directory)?;
  context.print_outcome(format!("created directory '{}'", bin_directory));

  let readme_md = apply_template(README_MD_TEMPLATE, "", bundle_configuration, bundle_directory, "")?;
  let readme_md_filename = format!("{}/README.md", example_directory);
  fs::write(&readme_md_filename, &readme_md)?;
  context.print_outcome(format!("created file '{}'", readme_md_filename));

  let cargo_toml = apply_template(CARGO_TOML_TEMPLATE, EXAMPLE_LIST_TOPICS, bundle_configuration, bundle_directory, "")?;
  let cargo_toml_filename = format!("{}/Cargo.toml", example_directory);
  fs::write(&cargo_toml_filename, &cargo_toml)?;
  context.print_outcome(format!("created file '{}'", cargo_toml_filename));

  let main_rs = apply_template(MAIN_RS_TEMPLATE, EXAMPLE_LIST_TOPICS, bundle_configuration, bundle_directory, "  ")?;
  let main_rs_filename = format!("{}/src/main.rs", example_directory);
  fs::write(&main_rs_filename, &main_rs)?;
  context.print_outcome(format!("created file '{}'", main_rs_filename));

  let consumer_rs = apply_template(CONSUMER_RS_TEMPLATE, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "  ")?;
  let consumer_rs_filename = format!("{}/src/bin/consumer.rs", example_directory);
  fs::write(&consumer_rs_filename, &consumer_rs)?;
  context.print_outcome(format!("created file '{}'", consumer_rs_filename));

  let producer_rs = apply_template(PRODUCER_RS_TEMPLATE, EXAMPLE_PRODUCER, bundle_configuration, bundle_directory, "  ")?;
  let producer_rs_filename = format!("{}/src/bin/producer.rs", example_directory);
  fs::write(&producer_rs_filename, &producer_rs)?;
  context.print_outcome(format!("created file '{}'", producer_rs_filename));

  Ok(Some(example_directory))
}

pub(crate) fn rust_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(example_directory(LANGUAGE_RUST, bundle_configuration, context))?)
}

pub(crate) fn delete_rust_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = example_directory(LANGUAGE_RUST, bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted rust example directory '{}'", example_directory));
  Ok(())
}
