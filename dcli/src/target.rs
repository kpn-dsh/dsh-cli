use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_tenant::parse_and_validate_guid;
use dsh_api::platform::DshPlatform;
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::capability_builder::CapabilityBuilder;
use crate::formatters::list_table::ListTable;
use crate::formatters::target::{TargetFormatter, TARGET_LABELS};
use crate::settings::{all_targets, delete_target, read_settings, read_target, upsert_target, write_settings, Settings, Target};
use crate::subject::Subject;
use crate::{confirmed, read_single_line, read_single_line_password, DcliContext, DcliResult};

pub(crate) struct TargetSubject {}

const TARGET_SUBJECT_TARGET: &str = "target";

lazy_static! {
  pub static ref TARGET_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TargetSubject {});
}

#[async_trait]
impl Subject for TargetSubject {
  fn subject(&self) -> &'static str {
    TARGET_SUBJECT_TARGET
  }

  /// Help text printed for -h flag
  fn subject_command_about(&self) -> String {
    "Show, manage and list dcli target configurations.".to_string()
  }

  /// Help text printed for --help flag
  fn subject_command_long_about(&self) -> String {
    "Show, manage and list dcli target configurations. \
    A target configuration consists of a platform name, a tenant name, \
    the tenant's group/user id and the tenant's api password for the platform. \
    The target command can be used to create, list and delete target configurations. \
    The target configurations will be stored in the application's home directory, \
    except for the password, which will be stored in the more secure \
    keyring application of your computer."
      .to_string()
  }

  fn requires_dsh_api_client(&self) -> bool {
    false
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)> {
    let mut capabilities: HashMap<CapabilityType, &(dyn Capability + Send + Sync)> = HashMap::new();
    capabilities.insert(CapabilityType::Default, TARGET_DEFAULT_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::Delete, TARGET_DELETE_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::List, TARGET_LIST_CAPABILITY.as_ref());
    capabilities.insert(CapabilityType::New, TARGET_NEW_CAPABILITY.as_ref());
    capabilities
  }
}

lazy_static! {
  pub static ref TARGET_DEFAULT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Default, "Set default target.")
      .set_long_about(
        "Set the default target. If you set a default target, \
        you won't be prompted for the platform and tenant name."
      )
      .set_default_command_executor(&TargetDefault {})
  );
  pub static ref TARGET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::Delete, "Delete target configuration.")
      .set_long_about(
        "Delete a target configuration. \
        You will be prompted for the target's platform and tenant, \
        and you need to explicitly confirm the action.",
      )
      .set_default_command_executor(&TargetDelete {})
  );
  pub static ref TARGET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::List, "List all target configurations.")
      .set_long_about("Lists all target configurations.")
      .set_default_command_executor(&TargetList {})
  );
  pub static ref TARGET_NEW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CapabilityType::New, "Create a new target configuration.")
      .set_long_about(
        "Create a new target configuration. \
        You will be prompted for the target's platform, tenant, group/user id and password. \
        The platform, tenant and id will be stored in an unencrypted configuration file. \
        The password will be stored in your computers keyring application, which is more secure.",
      )
      .set_default_command_executor(&TargetNew {})
  );
}

struct TargetDefault {}

#[async_trait]
impl CommandExecutor for TargetDefault {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("set default target");
    }
    let platform = read_single_line("enter platform: ")?;
    let platform = DshPlatform::try_from(platform.as_str())?;
    let tenant = read_single_line("enter tenant: ")?;
    match read_target(&platform, &tenant)? {
      Some(ref target) => {
        match read_settings(None)? {
          Some(settings) => {
            let settings = Settings { default_platform: Some(target.platform.to_string()), default_tenant: Some(target.tenant.clone()), ..settings };
            write_settings(None, settings)?;
          }
          None => {
            let settings = Settings { default_platform: Some(target.platform.to_string()), default_tenant: Some(target.tenant.clone()), ..Settings::default() };
            write_settings(None, settings)?;
          }
        }
        println!("target {} has been set as default", target);
      }
      None => {
        return Err(format!("target {}@{} does not exist", tenant, platform));
      }
    }
    Ok(false)
  }
}

struct TargetDelete {}

#[async_trait]
impl CommandExecutor for TargetDelete {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("delete existing target");
    }
    let platform = read_single_line("enter platform: ")?;
    let platform = DshPlatform::try_from(platform.as_str())?;
    let tenant = read_single_line("enter tenant: ")?;
    match read_target(&platform, &tenant)? {
      Some(target) => {
        let prompt = if target.password.is_some() {
          format!("type 'yes' to delete target '{}' and password from the keyring: ", target)
        } else {
          format!("type 'yes' to delete target '{}': ", target)
        };
        if confirmed(prompt)? {
          delete_target(&platform, &tenant)?;
          if target.password.is_some() {
            println!("target '{}' and password deleted", target);
          } else {
            println!("target '{}' deleted", target);
          }
        } else {
          println!("cancelled");
        }
      }
      None => {
        return Err(format!("target {}@{} does not exist", tenant, platform));
      }
    }

    Ok(false)
  }
}

struct TargetList {}

#[async_trait]
impl CommandExecutor for TargetList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("list all target configurations");
    }
    let mut table = ListTable::new(&TARGET_LABELS, context);
    let settings = read_settings(None)?;
    let (default_platform, default_tenant) = match settings {
      Some(settings) => (settings.default_platform, settings.default_tenant),
      None => (None, None),
    };
    let targets = all_targets()?;
    for target in targets {
      let is_default =
        default_platform.clone().is_some_and(|ref platform| target.platform.to_string() == *platform) && default_tenant.clone().is_some_and(|ref tenant| target.tenant == *tenant);
      let target_formatter = TargetFormatter { platform: target.platform.to_string(), tenant: target.tenant, group_user_id: target.group_user_id, is_default };
      table.value("", &target_formatter);
    }
    if table.is_empty() {
      println!("no targets configured");
    } else {
      table.print();
    }
    Ok(false)
  }
}

struct TargetNew {}

#[async_trait]
impl CommandExecutor for TargetNew {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &DcliContext) -> DcliResult {
    if context.show_capability_explanation() {
      println!("create new target configuration");
    }
    let platform = read_single_line("enter platform: ")?;
    let platform = DshPlatform::try_from(platform.as_str())?;
    let tenant = read_single_line("enter tenant: ")?;
    if let Some(existing_target) = read_target(&platform, &tenant)? {
      return Err(format!(
        "target configuration {} already exists (first delete the existing target configuration)",
        existing_target
      ));
    }
    let guid = parse_and_validate_guid(read_single_line("enter group/user id: ")?)?;
    let password = read_single_line_password("enter password: ")?;
    let target = Target::new(platform, tenant, guid, Some(password))?;
    upsert_target(&target)?;
    println!("target {} created", target);
    Ok(false)
  }
}
