use crate::formatters::{Label, SubjectFormatter, Value};
use async_trait::async_trait;
use clap::ArgMatches;
use dsh_api::platform::DshPlatform;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::arguments::{platform_name_argument, tenant_name_argument, PLATFORM_NAME_ARGUMENT, TENANT_NAME_ARGUMENT};
use crate::capability::{Capability, CommandExecutor, CREATE_COMMAND, CREATE_COMMAND_ALIAS, DELETE_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::directory::{get_settings, list_targets, read_target, upsert_target, write_settings};
use crate::formatters::list_formatter::ListFormatter;
use crate::settings::Settings;
use crate::subject::{Requirements, Subject};
use crate::targets::{delete_target, Target};
use crate::{err, read_single_line, DshCliResult};

struct TargetSubject {}

const TARGET_SUBJECT_TARGET: &str = "target";

lazy_static! {
  pub(crate) static ref TARGET_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(TargetSubject {});
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
    CapabilityBuilder::new(CREATE_COMMAND, Some(CREATE_COMMAND_ALIAS), &TargetCreate {}, "Create a new target configuration")
      .set_long_about(
        "Create a new target configuration. \
        You will be prompted for the target's platform, tenant and password. \
        The platform and tenant will be stored in an unencrypted configuration file. \
        The password will be stored in your computer's keyring, which is more secure.",
      )
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true))
  );
  static ref TARGET_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = Box::new(
    CapabilityBuilder::new(DELETE_COMMAND, None, &TargetDelete {}, "Delete target configuration")
      .set_long_about(
        "Delete a target configuration. \
        You will be prompted for the target's platform and tenant, \
        and you need to explicitly confirm the action.",
      )
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true))
  );
  static ref TARGET_LIST_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(LIST_COMMAND, Some(LIST_COMMAND_ALIAS), &TargetList {}, "List all target configurations").set_long_about("Lists all target configurations."));
  static ref TARGET_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![TARGET_CREATE_CAPABILITY.as_ref(), TARGET_DELETE_CAPABILITY.as_ref(), TARGET_LIST_CAPABILITY.as_ref()];
}

struct TargetCreate {}

#[async_trait]
impl CommandExecutor for TargetCreate {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    context.print_explanation("create new target configuration");
    let platform = get_platform_argument_or_prompt(matches)?;
    let tenant = get_tenant_argument_or_prompt(matches)?;
    if let Some(existing_target) = read_target(&platform, &tenant)? {
      return err!(
        "target configuration '{}' already exists (first delete the existing target configuration)",
        existing_target
      );
    };
    let password = context.read_single_line_password("enter password")?;
    let target = Target::new(platform, tenant, Some(password), vec![]);
    if context.dry_run() {
      context.print_warning(format!("dry-run mode, target '{}' not created", target));
    } else {
      upsert_target(&target)?;
      context.print_outcome(format!("target '{}' created", target));
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct TargetDelete {}

#[async_trait]
impl CommandExecutor for TargetDelete {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    context.print_explanation("delete existing target");
    let platform = get_platform_argument_or_prompt(matches)?;
    let tenant = get_tenant_argument_or_prompt(matches)?;
    match read_target(&platform, &tenant)? {
      Some(target) => {
        let prompt = if target.password.is_some() { format!("delete target '{}' and password from the keyring?", target) } else { format!("delete target '{}'?", target) };
        if context.confirmed(prompt)? {
          if context.dry_run() {
            context.print_warning(format!("dry-run mode, target '{}' not deleted", target));
          } else {
            delete_target(&platform, &tenant)?;
            if target.password.is_some() {
              context.print_outcome(format!("target '{}' and password deleted", target));
            } else {
              context.print_outcome(format!("target '{}' deleted", target));
            }
            let (settings, _) = get_settings()?;
            if let (Some(default_platform), Some(default_tenant)) = (settings.default_platform, settings.default_tenant) {
              if default_platform == target.platform.to_string() && default_tenant == target.tenant {
                let settings = Settings { default_platform: None, default_tenant: None, ..settings };
                write_settings(settings)?;
                context.print_outcome(format!("target '{}' unset as default", target));
              }
            }
          }
        } else {
          context.print_outcome("cancelled");
        }
      }
      None => return err!("target '{}@{}' does not exist", tenant, platform),
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct TargetList {}

#[async_trait]
impl CommandExecutor for TargetList {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all target configurations");
    let (settings, _) = get_settings()?;
    let targets: Vec<(DshPlatform, String)> = list_targets()?;
    let mut target_formatters = vec![];
    for (target_platform, target_tenant) in targets {
      let platform_is_default = settings
        .default_platform
        .clone()
        .is_some_and(|ref default_platform| default_platform == target_platform.name());
      let tenant_is_default = settings.default_tenant.clone().is_some_and(|ref default_tenant| default_tenant == &target_tenant);
      let target_formatter = TargetFormatter { platform: target_platform, tenant: target_tenant, is_default: platform_is_default && tenant_is_default };
      target_formatters.push(target_formatter);
    }
    if target_formatters.is_empty() {
      context.print_outcome("no targets configured");
    } else {
      let mut formatter = ListFormatter::new(&TARGET_LABELS, context);
      formatter.push_values(&target_formatters);
      formatter.print(None)?;
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

pub(crate) fn get_platform_argument_or_prompt(matches: &ArgMatches) -> DshCliResult<DshPlatform> {
  match matches.get_one::<String>(PLATFORM_NAME_ARGUMENT) {
    Some(dsh_platform) => Ok(DshPlatform::try_from(dsh_platform.as_str())?),
    None => Ok(DshPlatform::try_from(read_single_line("enter platform: ")?.as_str())?),
  }
}

pub(crate) fn get_tenant_argument_or_prompt(matches: &ArgMatches) -> DshCliResult<String> {
  match matches.get_one::<String>(TENANT_NAME_ARGUMENT) {
    Some(tenant_argument) => Ok(tenant_argument.to_string()),
    None => Ok(read_single_line("enter tenant: ")?),
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
enum TargetFormatterLabel {
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

#[derive(Serialize, Clone)]
struct TargetFormatter {
  platform: DshPlatform,
  tenant: String,
  is_default: bool,
}

impl SubjectFormatter<TargetFormatterLabel> for TargetFormatter {
  fn value(&self, label: &TargetFormatterLabel, _target_id: &str) -> Value {
    match label {
      TargetFormatterLabel::Default => {
        if self.is_default {
          Value::plain("*")
        } else {
          Value::empty()
        }
      }
      TargetFormatterLabel::Platform => Value::plain(format!("{} / {}", self.platform.name(), self.platform.alias())),
      TargetFormatterLabel::Tenant => Value::plain(&self.tenant),
    }
  }
}

static TARGET_LABELS: [TargetFormatterLabel; 3] = [TargetFormatterLabel::Platform, TargetFormatterLabel::Tenant, TargetFormatterLabel::Default];
