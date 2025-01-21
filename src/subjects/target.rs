use crate::formatters::formatter::{Label, SubjectFormatter};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::dsh_api_tenant::parse_and_validate_guid;
use dsh_api::platform::DshPlatform;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::arguments::{guid_argument, platform_argument, tenant_argument, PlatformArgument, GUID_ARGUMENT, PLATFORM_ARGUMENT, TENANT_ARGUMENT};
use crate::capability::{
  Capability, CommandExecutor, DEFAULT_COMMAND, DEFAULT_COMMAND_PAIR, DELETE_COMMAND, DELETE_COMMAND_PAIR, LIST_COMMAND, LIST_COMMAND_PAIR, NEW_COMMAND, NEW_COMMAND_PAIR,
};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::list_formatter::ListFormatter;
use crate::settings::{all_targets, delete_target, read_settings, read_target, upsert_target, write_settings, Settings, Target};
use crate::subject::Subject;
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
    "Show, manage and list dsh target configurations. \
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

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DEFAULT_COMMAND => Some(TARGET_DEFAULT_CAPABILITY.as_ref()),
      DELETE_COMMAND => Some(TARGET_DELETE_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(TARGET_LIST_CAPABILITY.as_ref()),
      NEW_COMMAND => Some(TARGET_NEW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &TARGET_CAPABILITIES
  }
}

lazy_static! {
  static ref TARGET_DEFAULT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DEFAULT_COMMAND_PAIR, "Set default target.")
      .set_long_about(
        "Set the default target. If you set a default target, \
        you won't be prompted for the platform and tenant name."
      )
      .set_default_command_executor(&TargetDefault {})
      .add_extra_arguments(vec![platform_argument(), tenant_argument()])
  );
  static ref TARGET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND_PAIR, "Delete target configuration.")
      .set_long_about(
        "Delete a target configuration. \
        You will be prompted for the target's platform and tenant, \
        and you need to explicitly confirm the action.",
      )
      .set_default_command_executor(&TargetDelete {})
      .add_extra_arguments(vec![platform_argument(), tenant_argument()])
  );
  static ref TARGET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(LIST_COMMAND_PAIR, "List all target configurations.")
      .set_long_about("Lists all target configurations.")
      .set_default_command_executor(&TargetList {})
  );
  static ref TARGET_NEW_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(NEW_COMMAND_PAIR, "Create a new target configuration.")
      .set_long_about(
        "Create a new target configuration. \
        You will be prompted for the target's platform, tenant, group/user id and password. \
        The platform, tenant and id will be stored in an unencrypted configuration file. \
        The password will be stored in your computers keyring application, which is more secure.",
      )
      .set_default_command_executor(&TargetNew {})
      .add_extra_arguments(vec![guid_argument(), platform_argument(), tenant_argument()])
  );
  static ref TARGET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![TARGET_DEFAULT_CAPABILITY.as_ref(), TARGET_DELETE_CAPABILITY.as_ref(), TARGET_LIST_CAPABILITY.as_ref(), TARGET_NEW_CAPABILITY.as_ref()];
}

struct TargetDefault {}

#[async_trait]
impl CommandExecutor for TargetDefault {
  async fn execute(&self, _target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("set default target");
    let platform = get_platform_argument_or_prompt(matches)?;
    let tenant = get_tenant_argument_or_prompt(matches)?;
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
        context.print_outcome(format!("target {} has been set as default", target));
      }
      None => {
        return Err(format!("target {}@{} does not exist", tenant, platform));
      }
    }
    Ok(())
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
        let prompt = if target.password.is_some() {
          format!("type 'yes' to delete target '{}' and password from the keyring: ", target)
        } else {
          format!("type 'yes' to delete target '{}': ", target)
        };
        if context.confirmed(prompt)? {
          if context.dry_run {
            context.print_warning(format!("dry-run mode, target {} not deleted", target));
          } else {
            delete_target(&platform, &tenant)?;
            if target.password.is_some() {
              context.print_outcome(format!("target '{}' and password deleted", target));
            } else {
              context.print_outcome(format!("target '{}' deleted", target));
            }
            if let Some(settings) = read_settings(None)? {
              if let (Some(default_platform), Some(default_tenant)) = (settings.default_platform, settings.default_tenant) {
                if default_platform == target.platform.to_string() && default_tenant == target.tenant {
                  let settings = Settings { default_platform: None, default_tenant: None, ..settings };
                  write_settings(None, settings)?;
                  context.print_outcome(format!("target '{}' unset as default", target));
                }
              }
            }
          }
        } else {
          context.print_outcome("cancelled");
        }
      }
      None => return Err(format!("target {}@{} does not exist", tenant, platform)),
    }
    Ok(())
  }
}

struct TargetList {}

#[async_trait]
impl CommandExecutor for TargetList {
  async fn execute(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("list all target configurations");
    let settings = read_settings(None)?;
    let (default_platform, default_tenant) = match settings {
      Some(settings) => (settings.default_platform, settings.default_tenant),
      None => (None, None),
    };
    let targets = all_targets()?;
    let mut target_formatters = vec![];
    for target in targets {
      let is_default =
        default_platform.clone().is_some_and(|ref platform| target.platform.to_string() == *platform) && default_tenant.clone().is_some_and(|ref tenant| target.tenant == *tenant);
      let target_formatter = TargetFormatter { platform: target.platform.to_string(), tenant: target.tenant, group_user_id: target.group_user_id, is_default };
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
}

struct TargetNew {}

#[async_trait]
impl CommandExecutor for TargetNew {
  async fn execute(&self, _target: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("create new target configuration");
    let platform = get_platform_argument_or_prompt(matches)?;
    let tenant = get_tenant_argument_or_prompt(matches)?;
    if let Some(existing_target) = read_target(&platform, &tenant)? {
      return Err(format!(
        "target configuration {} already exists (first delete the existing target configuration)",
        existing_target
      ));
    };
    let guid = get_guid_argument_or_prompt(matches)?;
    let password = context.read_single_line_password("enter password: ")?;
    let target = Target::new(platform, tenant, guid, Some(password))?;
    if context.dry_run {
      context.print_warning(format!("dry-run mode, target {} not created", target));
    } else {
      upsert_target(&target)?;
      context.print_outcome(format!("target {} created", target));
    }
    Ok(())
  }
}

fn get_guid_argument_or_prompt(matches: &ArgMatches) -> Result<u16, String> {
  match matches.get_one::<String>(GUID_ARGUMENT) {
    Some(tenant_argument) => Ok(parse_and_validate_guid(tenant_argument.to_string())?),
    None => Ok(parse_and_validate_guid(read_single_line("enter group/user id: ")?)?),
  }
}

fn get_platform_argument_or_prompt(matches: &ArgMatches) -> Result<DshPlatform, String> {
  match matches.get_one::<PlatformArgument>(PLATFORM_ARGUMENT) {
    Some(platform_argument) => Ok(DshPlatform::try_from(platform_argument.to_string().as_str())?),
    None => Ok(DshPlatform::try_from(read_single_line("enter platform: ")?.as_str())?),
  }
}

fn get_tenant_argument_or_prompt(matches: &ArgMatches) -> Result<String, String> {
  match matches.get_one::<String>(TENANT_ARGUMENT) {
    Some(tenant_argument) => Ok(tenant_argument.to_string()),
    None => Ok(read_single_line("enter tenant: ")?),
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum TargetFormatterLabel {
  Default,
  GroupUserId,
  Platform,
  Tenant,
}

impl Label for TargetFormatterLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::Default => "default",
      Self::GroupUserId => "id",
      Self::Platform => "platform",
      Self::Tenant => "tenant",
    }
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

#[derive(Serialize)]
pub struct TargetFormatter {
  pub(crate) platform: String,
  pub(crate) tenant: String,
  pub(crate) group_user_id: u16,
  pub(crate) is_default: bool,
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
      TargetFormatterLabel::GroupUserId => format!("{}:{}", self.group_user_id, self.group_user_id),
      TargetFormatterLabel::Platform => self.platform.to_string(),
      TargetFormatterLabel::Tenant => self.tenant.clone(),
    }
  }

  fn target_label(&self) -> Option<TargetFormatterLabel> {
    None
  }
}

pub static TARGET_LABELS: [TargetFormatterLabel; 4] =
  [TargetFormatterLabel::Tenant, TargetFormatterLabel::Platform, TargetFormatterLabel::GroupUserId, TargetFormatterLabel::Default];
