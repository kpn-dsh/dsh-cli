use crate::bundle::proxy::ProxyCertificateBundleConfig;
use crate::code::{apply_template, example_directory, EXAMPLE_CONSUMER, LANGUAGE_SCALA3};
use crate::context::Context;
use crate::DshCliResult;
use std::fs;
use std::fs::{create_dir_all, exists, remove_dir_all};

const README_MD_TEMPLATE: &str = include_str!("README.md.template");
const BUILD_SBT_TEMPLATE: &str = include_str!("build.sbt.template");
const PACKAGE_SCALA_TEMPLATE: &str = include_str!("package.scala.template");
const CONSUMER_SCALA_TEMPLATE: &str = include_str!("Consumer.scala.template");
const PRODUCER_SCALA_TEMPLATE: &str = include_str!("Producer.scala.template");
const LIST_TOPICS_SCALA_TEMPLATE: &str = include_str!("ListTopics.scala.template");
const DSH_PROXY_PROPERTIES_SCALA_TEMPLATE: &str = include_str!("DshProxyProperties.scala.template");

pub(crate) fn generate_scala3_example_code(bundle_configuration: &ProxyCertificateBundleConfig, bundle_directory: &str, context: &Context) -> DshCliResult<Option<String>> {
  let example_directory = example_directory(LANGUAGE_SCALA3, bundle_configuration, context);
  create_dir_all(&example_directory)?;
  context.print_outcome(format!("created directory '{}'", example_directory));

  let package_directory = format!(
    "{}/src/main/scala/org/kpn/dsh/proxy/{}",
    example_directory,
    bundle_configuration.proxy_name.replace("-", "_")
  );
  create_dir_all(&package_directory)?;
  context.print_outcome(format!("created directory '{}'", package_directory));

  let readme_md = apply_template(README_MD_TEMPLATE, "", bundle_configuration, bundle_directory, "")?;
  let readme_md_filename = format!("{}/README.md", example_directory);
  fs::write(&readme_md_filename, &readme_md)?;
  context.print_outcome(format!("created file '{}'", readme_md_filename));

  let build_sbt = apply_template(BUILD_SBT_TEMPLATE, "", bundle_configuration, bundle_directory, "")?;
  let build_sbt_filename = format!("{}/build.sbt", example_directory);
  fs::write(&build_sbt_filename, &build_sbt)?;
  context.print_outcome(format!("created file '{}'", build_sbt_filename));

  let package_scala = apply_template(PACKAGE_SCALA_TEMPLATE, "", bundle_configuration, bundle_directory, "    ")?;
  let package_scala_filename = format!("{}/package.scala", package_directory);
  fs::write(&package_scala_filename, &package_scala)?;
  context.print_outcome(format!("created file '{}'", package_scala_filename));

  let consumer_scala = apply_template(CONSUMER_SCALA_TEMPLATE, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "    ")?;
  let consumer_scala_filename = format!("{}/Consumer.scala", package_directory);
  fs::write(&consumer_scala_filename, &consumer_scala)?;
  context.print_outcome(format!("created file '{}'", consumer_scala_filename));

  let producer_scala = apply_template(PRODUCER_SCALA_TEMPLATE, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "    ")?;
  let producer_scala_filename = format!("{}/Producer.scala", package_directory);
  fs::write(&producer_scala_filename, &producer_scala)?;
  context.print_outcome(format!("created file '{}'", producer_scala_filename));

  let list_topics_scala = apply_template(LIST_TOPICS_SCALA_TEMPLATE, EXAMPLE_CONSUMER, bundle_configuration, bundle_directory, "    ")?;
  let list_topics_scala_filename = format!("{}/ListTopics.scala", package_directory);
  fs::write(&list_topics_scala_filename, &list_topics_scala)?;
  context.print_outcome(format!("created file '{}'", list_topics_scala_filename));

  let dsh_proxy_properties_scala = apply_template(
    DSH_PROXY_PROPERTIES_SCALA_TEMPLATE,
    EXAMPLE_CONSUMER,
    bundle_configuration,
    bundle_directory,
    "    ",
  )?;
  let dsh_proxy_properties_scala_filename = format!("{}/DshProxyProperties.scala", package_directory);
  fs::write(&dsh_proxy_properties_scala_filename, &dsh_proxy_properties_scala)?;
  context.print_outcome(format!("created file '{}'", dsh_proxy_properties_scala_filename));

  Ok(Some(example_directory))
}

pub(crate) fn scala3_example_code_exists(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<bool> {
  Ok(exists(example_directory(LANGUAGE_SCALA3, bundle_configuration, context))?)
}

pub(crate) fn delete_scala3_example_code(bundle_configuration: &ProxyCertificateBundleConfig, context: &Context) -> DshCliResult<()> {
  let example_directory = example_directory(LANGUAGE_SCALA3, bundle_configuration, context);
  remove_dir_all(&example_directory)?;
  context.print_outcome(format!("deleted rust example directory '{}'", example_directory));
  Ok(())
}
