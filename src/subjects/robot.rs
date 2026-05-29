use crate::authentication::AuthenticationMethod;
#[cfg(feature = "robot")]
use crate::capability::UPDATE_COMMAND;
use crate::capability::{Capability, CommandExecutor, COPY_COMMAND, IMPORT_COMMAND, LIST_COMMAND, LIST_COMMAND_ALIAS, SET_COMMAND, UNSET_COMMAND};
use crate::capability_builder::CapabilityBuilder;
use crate::clients::create_client_access_token_from_platform_tenant;
use crate::context::Context;
#[cfg(feature = "robot")]
use crate::error::DshCliError;
use crate::formatters::list_formatter::ListFormatter;
use crate::formatters::{Label, SubjectFormatter, Value};
use crate::keyring::{get_secret_from_keyring, get_secrets_from_keyring, remove_secret_from_keyring, upsert_secret_to_keyring};
use crate::subject::{Requirements, Subject};
use crate::target_platform::{get_target_platform_from_command_line_argument, platform_name_argument};
use crate::target_tenant::{get_target_tenant_from_command_line_argument, tenant_name_argument};
use crate::{err, DshCliResult};
use arboard::Clipboard;
use async_trait::async_trait;
use clap::ArgMatches;
#[cfg(feature = "robot")]
use dsh_api::dsh_api_client_factory::DshApiClientFactory;
#[cfg(feature = "robot")]
use dsh_api::dsh_api_tenant::DshApiTenant;
#[cfg(feature = "robot")]
use dsh_api::error::DshApiError;
use dsh_api::platform::DshPlatform;
use dsh_api::secret::ROBOT_SECRET;
use itertools::Itertools;
use log::error;
use serde::Serialize;
use std::sync::LazyLock;

pub struct RobotSubject {}

const ROBOT_SUBJECT_TARGET: &str = "robot";

pub(crate) static ROBOT_SUBJECT: LazyLock<Box<dyn Subject + Send + Sync>> = LazyLock::new(|| Box::new(RobotSubject {}));

#[async_trait]
impl Subject for RobotSubject {
  fn subject(&self) -> &'static str {
    ROBOT_SUBJECT_TARGET
  }

  fn subject_command_about(&self) -> String {
    "Manage and store robot secrets.".to_string()
  }

  fn subject_command_long_about(&self) -> String {
    "Manage and store robot secrets.".to_string()
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      COPY_COMMAND => Some(ROBOT_COPY_CAPABILITY.as_ref()),
      IMPORT_COMMAND => Some(ROBOT_IMPORT_CAPABILITY.as_ref()),
      LIST_COMMAND => Some(ROBOT_LIST_CAPABILITY.as_ref()),
      SET_COMMAND => Some(ROBOT_SET_CAPABILITY.as_ref()),
      UNSET_COMMAND => Some(ROBOT_UNSET_CAPABILITY.as_ref()),
      #[cfg(feature = "robot")]
      UPDATE_COMMAND => Some(ROBOT_UPDATE_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &ROBOT_CAPABILITIES
  }
}

static ROBOT_COPY_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(COPY_COMMAND, None, &RobotCopy {}, "Copy robot secret from local keyring to clipboard")
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true)),
  )
});

static ROBOT_IMPORT_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(IMPORT_COMMAND, None, &RobotImport {}, "Import the robot secret from the platform secret store")
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true))
      .set_long_about(
        "Import the robot secret from the DSH platform secret store into your local keyring. This \
       command is only available with the single-sign-on authentication method.",
      ),
  )
});

static ROBOT_LIST_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(CapabilityBuilder::new(
    LIST_COMMAND,
    Some(LIST_COMMAND_ALIAS),
    &RobotList {},
    "List robot secrets in your local keyring",
  ))
});

static ROBOT_SET_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(SET_COMMAND, None, &RobotSet {}, "Store a robot secret into local keyring")
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true)),
  )
});

static ROBOT_UNSET_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(UNSET_COMMAND, None, &RobotUnset {}, "Remove robot secret from local keyring")
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true)),
  )
});

#[cfg(feature = "robot")]
static ROBOT_UPDATE_CAPABILITY: LazyLock<Box<(dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  Box::new(
    CapabilityBuilder::new(UPDATE_COMMAND, None, &RobotUpdate {}, "Request a new robot secret")
      .add_target_argument(platform_name_argument().required(true))
      .add_target_argument(tenant_name_argument().required(true))
      .set_long_about(
        "Triggers the generation of a new robot secret for the tenant’s robot account. \
         This automatically invalidates the existing client secret. The new secret will be stored \
         in DSH secret store and in your local keyring. This command is only available with the \
         robot authentication method and thus requires the original robot password.",
      ),
  )
});

static ROBOT_CAPABILITIES: LazyLock<Vec<&'static (dyn Capability + Send + Sync)>> = LazyLock::new(|| {
  vec![
    ROBOT_COPY_CAPABILITY.as_ref(),
    ROBOT_IMPORT_CAPABILITY.as_ref(),
    ROBOT_LIST_CAPABILITY.as_ref(),
    ROBOT_SET_CAPABILITY.as_ref(),
    ROBOT_UNSET_CAPABILITY.as_ref(),
    #[cfg(feature = "robot")]
    ROBOT_UPDATE_CAPABILITY.as_ref(),
  ]
});

fn get_target_platform_and_tenant_from_command_line_arguments(matches: &ArgMatches) -> DshCliResult<(DshPlatform, String)> {
  let platform = get_target_platform_from_command_line_argument(matches)?.unwrap_or_else(|| unreachable!());
  let tenant = get_target_tenant_from_command_line_argument(matches).unwrap_or_else(|| unreachable!());
  Ok((platform, tenant))
}

struct RobotCopy {}

#[async_trait]
impl CommandExecutor for RobotCopy {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    let (platform, tenant) = get_target_platform_and_tenant_from_command_line_arguments(matches)?;
    context.print_explanation(format!("copy robot secret for '{}@{}' to clipboard", platform, tenant));
    match get_secret_from_keyring(&platform, &tenant, ROBOT_SECRET)? {
      Some(robot_secret) => match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(robot_secret)) {
        Ok(_) => context.print_outcome(format!("robot secret for '{}@{}' copied to clipboard", platform, tenant)),
        Err(error) => {
          error!("clipboard error ({})", error);
          context.print_error(format!("robot secret for '{}@{}' could not be copied to clipboard", platform, tenant));
        }
      },
      None => context.print_error(format!("robot secret for '{}@{}' not found in keyring", platform, tenant)),
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct RobotImport {}

#[async_trait]
impl CommandExecutor for RobotImport {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    if context.stdin_is_terminal() {
      let (platform, tenant) = get_target_platform_and_tenant_from_command_line_arguments(matches)?;
      match create_client_access_token_from_platform_tenant(&platform, &tenant, context).await? {
        Some(client) => {
          let start_instant = context.now();
          let robot_secret = client.get_secret(ROBOT_SECRET).await?;
          context.print_execution_time(start_instant);
          context.print_explanation(format!("copy robot secret for '{}@{}' from platform secret store to keyring", platform, tenant));
          if get_secret_from_keyring(&platform, &tenant, ROBOT_SECRET)?.is_some()
            && !context.confirmed(format!("overwrite existing robot secret for '{}@{}' in keyring?", platform, tenant))?
          {
            context.print_outcome(format!("cancelled, robot secret for '{}@{}' not stored to keyring", platform, tenant));
            return Ok(());
          }
          if context.dry_run() {
            context.print_warning("dry-run mode, secret not stored in keyring");
          } else {
            match upsert_secret_to_keyring(&platform, &tenant, ROBOT_SECRET, &robot_secret) {
              Ok(_) => context.print_outcome("robot secret stored in keyring"),
              Err(_) => context.print_error("robot secret could not be stored in keyring"),
            }
          }
          Ok(())
        }
        None => Ok(()),
      }
    } else {
      err!("command is only available in interactive mode")
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::new(false, Some(AuthenticationMethod::SingleSignOn), false)
  }
}

static ROBOT_LABELS_LIST: [RobotLabel; 2] = [RobotLabel::PlatformName, RobotLabel::TenantName];

struct RobotList {}

#[async_trait]
impl CommandExecutor for RobotList {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, _: &ArgMatches, context: &Context) -> DshCliResult<()> {
    context.print_explanation("list all robot secret targets from keyring");
    let secret_targets = get_secrets_from_keyring()?;
    let targets: Vec<(&String, &String)> = secret_targets
      .iter()
      .flat_map(|(platform_name, tenants)| tenants.iter().map(|tenant| (platform_name, tenant)).collect_vec())
      .collect_vec();
    let mut formatter = ListFormatter::new(&ROBOT_LABELS_LIST, context);
    formatter.push_values(&targets);
    formatter.print(None)?;
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct RobotSet {}

#[async_trait]
impl CommandExecutor for RobotSet {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    if context.stdin_is_terminal() {
      let (platform, tenant) = get_target_platform_and_tenant_from_command_line_arguments(matches)?;
      context.print_explanation(format!("store robot secret for '{}@{}' in keyring", platform, tenant));
      let robot_secret = context.read_single_line_password("enter robot secret")?;
      if context.dry_run() {
        context.print_warning("dry-run mode, secret not stored to keyring");
      } else {
        match upsert_secret_to_keyring(&platform, &tenant, ROBOT_SECRET, &robot_secret) {
          Ok(_) => context.print_outcome("robot secret stored in keyring"),
          Err(_) => context.print_error("robot secret could not be stored in keyring"),
        }
      }
    } else {
      context.print_warning("keyring is only available in interactive mode");
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

struct RobotUnset {}

#[async_trait]
impl CommandExecutor for RobotUnset {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    if context.stdin_is_terminal() {
      let (platform, tenant) = get_target_platform_and_tenant_from_command_line_arguments(matches)?;
      context.print_explanation(format!("remove robot secret for '{}@{}' from keyring", platform, tenant));
      if context.confirmed(format!("remove robot secret for '{}@{}'?", platform, tenant))? {
        if context.dry_run() {
          context.print_warning("dry-run mode, secret not removed from keyring");
        } else {
          match remove_secret_from_keyring(&platform, &tenant, ROBOT_SECRET) {
            Ok(_) => context.print_outcome("robot secret removed from keyring"),
            Err(_) => context.print_error("robot secret could not be removed in keyring"),
          }
        }
      } else {
        context.print_outcome(format!("cancelled, robot secret for '{}@{}' not removed from keyring", platform, tenant));
      }
    } else {
      context.print_warning("keyring is only available in interactive mode");
    }
    Ok(())
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::standard_without_api()
  }
}

#[cfg(feature = "robot")]
struct RobotUpdate {}

#[cfg(feature = "robot")]
#[async_trait]
impl CommandExecutor for RobotUpdate {
  async fn execute_without_client(&self, _: Option<String>, _: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult<()> {
    if context.stdin_is_terminal() {
      let (robot_platform, robot_tenant) = get_target_platform_and_tenant_from_command_line_arguments(matches)?;
      context.print_explanation(format!("renew the robot secret for '{}@{}'", robot_platform, robot_tenant));
      context.print_warning("this will automatically invalidate the existing robot secret");
      if context.confirmed(format!("renew robot secret for '{}@{}'?", robot_platform, robot_tenant))? {
        let robot_secret = context.read_single_line_password("enter the current robot secret")?;
        if robot_secret.is_empty() {
          return err!("robot secret cannot be empty, update cancelled");
        }
        let dsh_api_tenant = DshApiTenant::new(robot_tenant.clone(), robot_platform.clone());
        let dsh_api_client_factory = DshApiClientFactory::create_with_token_fetcher(dsh_api_tenant, robot_secret);
        let client = dsh_api_client_factory.client().await?;
        let secret_dependants = match client.secret_dependants(ROBOT_SECRET).await {
          Ok(dependants) => dependants,
          Err(error) => match error {
            DshApiError::Unexpected { message, .. } if message == "statuscode 401 Unauthorized" => return err!("authorization error"),
            _ => return Err(DshCliError::from(error)),
          },
        };
        if context.dependencies_warning("robot secret", secret_dependants, ROBOT_SECRET) && !context.confirmed("do you want to continue?")? {
          context.print_outcome(format!("cancelled, robot secret '{}' not updated", ROBOT_SECRET));
          return Ok(());
        }
        if context.dry_run() {
          context.print_warning("dry-run mode, robot secret not renewed");
        } else {
          let new_robot_secret = client.post_robot_generate_secret().await?;
          context.print_outcome(format!("robot secret for '{}@{}' renewed", robot_platform, robot_tenant));
          if context.confirmed(format!("store new robot secret for '{}@{}' in keyring?", robot_platform, robot_tenant))? {
            upsert_secret_to_keyring(&robot_platform, &robot_tenant, ROBOT_SECRET, &new_robot_secret.value)?;
            context.print_outcome(format!("robot secret for '{}@{}' stored in keyring", robot_platform, robot_tenant));
          } else {
            context.print_outcome(format!("cancelled, robot secret for '{}@{}' not stored in keyring", robot_platform, robot_tenant));
          }
        }
      } else {
        context.print_outcome(format!("cancelled, robot secret for '{}@{}' not renewed", robot_platform, robot_tenant));
      }
      Ok(())
    } else {
      err!("command is only available in interactive mode")
    }
  }

  fn requirements(&self, _: &ArgMatches) -> Requirements {
    Requirements::new(false, Some(AuthenticationMethod::Robot), false)
  }
}

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum RobotLabel {
  PlatformName,
  TenantName,
}

impl Label for RobotLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::PlatformName => "platform",
      Self::TenantName => "tenant",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::PlatformName)
  }
}

impl SubjectFormatter<RobotLabel> for (&String, &String) {
  fn value(&self, label: &RobotLabel, _target_id: &str) -> Value {
    let (platform_name, tenant_name) = self;
    match label {
      RobotLabel::PlatformName => Value::plain(platform_name),
      RobotLabel::TenantName => Value::plain(tenant_name),
    }
  }
}
