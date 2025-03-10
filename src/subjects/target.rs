use crate::formatters::formatter::{Label, SubjectFormatter};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::platform::DshPlatform;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::arguments::{platform_name_argument, tenant_name_argument, PLATFORM_NAME_ARGUMENT, TENANT_NAME_ARGUMENT};
use crate::capability::{Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::list_formatter::ListFormatter;
use crate::settings::{get_settings, write_settings, Settings};
use crate::subject::{Requirements, Subject};
use crate::targets::{all_targets, delete_target, read_target, upsert_target, Target};
use crate::{read_single_line, DshCliResult};

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
    "Show, manage and list dsh target configurations.".to_string()
  }

  /// Help text printed for --help flag
  fn subject_command_long_about(&self) -> String {
    "Create, list and show dsh target configurations. \
    A target configuration consists of a platform name, a tenant name \
    and the tenant's api password for the platform. \
    The target command can be used to create, list and delete target configurations. \
    The target configurations will be stored in the dsh tool's home directory, \
    except for the password, which will be stored in the more secure \
    keyring of your computer."
      .to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      CREATE_COMMAND => Some(TARGET_CREATE_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(TARGET_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(TARGET_LIST_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TARGET_CAPABILITIES
  }
}

lazy_static! {
  static ref TARGET_CREATE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), "Create a new target configuration")
      .set_long_about(
        "Create a new target configuration. \
        You will be prompted for the target's platform, tenant and password. \
        The platform and tenant will be stored in an unencrypted configuration file. \
        The password will be stored in your computer's keyring, which is more secure.",
      )
      .set_default_command_executor(&TargetCreate {})
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true))
  );
  static ref TARGET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, "Delete target configuration")
      .set_long_about(
        "Delete a target configuration. \
        You will be prompted for the target's platform and tenant, \
        and you need to explicitly confirm the action.",
      )
      .set_default_command_executor(&TargetDelete {})
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true))
  );
  static ref TARGET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), "List all target configurations")
      .set_long_about("Lists all target configurations.")
      .set_default_command_executor(&TargetList {})
  );
  static ref TARGET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![TARGET_CREATE_CAPABILITY.as_ref(), TARGET_DELETE_CAPABILITY.as_ref(), TARGET_LIST_CAPABILITY.as_ref()];
}

struct TargetCreate {}

#[async_trait]
impl CommandExecutor for TargetCreate {
  async fn execute(&self, _target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("create new target configuration");
    let platform = get_platform_argument_or_prompt(matches)?;
    let tenant = get_tenant_argument_or_prompt(matches)?;
    if let Some(existing_target) = read_target(&platform, &tenant)? {
      return Err(format!(
        "target configuration '{}' already exists (first delete the existing target configuration)",
        existing_target
      ));
    };
    let password = context.read_single_line_password("enter password: ")?;
    let target = Target::new(platform, tenant, Some(password))?;
    if context.dry_run {
      context.print_warning(format!("dry-run mode, target '{}' not created", target));
    } else {
      upsert_target(&target)?;
      context.print_outcome(format!("target '{}' created", target));
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_without_api(None)
  }
}

struct TargetDelete {}

#[async_trait]
impl CommandExecutor for TargetDelete {
  async fn execute(&self, _target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("delete existing target");
    let platform = get_platform_argument_or_prompt(matches)?;
    let tenant = get_tenant_argument_or_prompt(matches)?;
    match read_target(&platform, &tenant)? {
      Some(target) => {
        let prompt = if target.password.is_some() { format!("delete target '{}' and password from the keyring?", target) } else { format!("delete target '{}'?", target) };
        if context.confirmed(prompt)? {
          if context.dry_run {
            context.print_warning(format!("dry-run mode, target '{}' not deleted", target));
          } else {
            delete_target(&platform, &tenant)?;
            if target.password.is_some() {
              context.print_outcome(format!("target '{}' and password deleted", target));
            } else {
              context.print_outcome(format!("target '{}' deleted", target));
            }
            let (settings, _) = get_settings(None)?;
            if let (Some(default_platform), Some(default_tenant)) = (settings.default_platform, settings.default_tenant) {
              if default_platform == target.platform.to_string() && default_tenant == target.tenant {
                let settings = Settings { default_platform: None, default_tenant: None, ..settings };
                write_settings(None, settings)?;
                context.print_outcome(format!("target '{}' unset as default", target));
              }
            }
          }
        } else {
          context.print_outcome("cancelled");
        }
      }
      None => return Err(format!("target '{}@{}' does not exist", tenant, platform)),
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_without_api(None)
  }
}

struct TargetList {}

#[async_trait]
impl CommandExecutor for TargetList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all target configurations");
    let (settings, _) = get_settings(None)?;
    let targets = all_targets()?;
    let mut target_formatters = vec![];
    for target in targets {
      let platform_is_default = settings
        .default_platform
        .clone()
        .is_some_and(|ref platform| target.platform.to_string() == *platform);
      let tenant_is_default = settings.default_tenant.clone().is_some_and(|ref tenant| target.tenant == *tenant);
      let target_formatter = TargetFormatter { platform: target.platform, tenant: target.tenant, is_default: platform_is_default && tenant_is_default };
      target_formatters.push(target_formatter);
    }
    if target_formatters.is_empty() {
      context.print_outcome("no targets configured");
    } else {
      let mut formatter = ListFormatter::new(&TARGET_LABELS, None, context);
      formatter.push_values(&target_formatters);
      formatter.print()?;
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_without_api(None)
  }
}

pub(crate) fn get_platform_argument_or_prompt(matches: &ArgMatches) -> Result<DshPlatform, String> {
  match matches.get_one::<String>(PLATFORM_NAME_ARGUMENT) {
    Some(dsh_platform) => Ok(DshPlatform::try_from(dsh_platform.as_str())?),
    None => Ok(DshPlatform::try_from(read_single_line("enter platform: ")?.as_str())?),
  }
}

pub(crate) fn get_tenant_argument_or_prompt(matches: &ArgMatches) -> Result<String, String> {
  match matches.get_one::<String>(TENANT_NAME_ARGUMENT) {
    Some(tenant_argument) => Ok(tenant_argument.to_string()),
    None => Ok(read_single_line("enter tenant: ")?),
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum TargetFormatterLabel {
  Default,
  Platform,
  Tenant,
}

impl Label for TargetFormatterLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Default => "default",
      Self::Platform => "platform",
      Self::Tenant => "tenant",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Platform)
  }
}

#[derive(Serialize)]
struct TargetFormatter {
  platform: DshPlatform,
  tenant: String,
  is_default: bool,
}

impl SubjectFormatter<TargetFormatterLabel> for TargetFormatter {
  fn value(&self, label: &TargetFormatterLabel, _target_id: &str) -> String {
    match label {
      TargetFormatterLabel::Default => {
        if self.is_default {
          "*".to_string()
        } else {
          "".to_string()
        }
      }
      TargetFormatterLabel::Platform => format!("{} / {}", self.platform.name(), self.platform.alias()),
      TargetFormatterLabel::Tenant => self.tenant.clone(),
    }
  }
}

pub static TARGET_LABELS: [TargetFormatterLabel; 3] = [TargetFormatterLabel::Platform, TargetFormatterLabel::Tenant, TargetFormatterLabel::Default];
