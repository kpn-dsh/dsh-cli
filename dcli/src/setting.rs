use async_trait::async_trait;
use clap::ArgMatches;
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::formatters::setting::SETTING_LABELS;
use crate::formatters::show_table::ShowTable;
use crate::settings::read_settings;
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};

pub(crate) struct SettingSubject {}

const SETTING_SUBJECT_TARGET: &str = "setting";

lazy_static! {
  pub static ref SETTING_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(SettingSubject {});
}

#[async_trait]
impl Subject for SettingSubject {
  fn subject(&self) -> &'static str {
    SETTING_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Show, manage and list dcli settings.".to_string()
  }

  fn requires_dsh_api_client(&self) -> bool {
    false
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::List, SETTING_LIST_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref SETTING_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List settings")
      .set_long_about("Lists all dcli settings.")
      .set_default_command_executor(&SettingList {})
  );
}

struct SettingList {}

#[async_trait]
impl CommandExecutor for SettingList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list default settings");
    }
    match read_settings(None)? {
      Some(settings) => ShowTable::new("current values", &settings, &SETTING_LABELS, context).print(),
      None => println!("no default settings found"),
    }
    Ok(false)
  }
}
