use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;

use crate::arguments::{query_argument, target_argument, QUERY_ARGUMENT};
use crate::capability::CapabilityType::*;
use crate::command::TARGET_ARGUMENT;
use crate::flags::{create_flag, FlagType};
use crate::subject::Subject;
use crate::CommandResult;

pub(crate) const CREATE: &str = "create";
pub(crate) const DELETE: &str = "delete";
pub(crate) const FIND: &str = "find";
pub(crate) const LIST: &str = "list";
pub(crate) const SHOW: &str = "show";
pub(crate) const START: &str = "start";
pub(crate) const STOP: &str = "stop";
pub(crate) const UPDATE: &str = "update";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CapabilityType {
  Create,
  Delete,
  Find,
  List,
  Show,
  Start,
  Stop,
  Update,
}

pub(crate) static ALL_CAPABILITY_TYPES: [CapabilityType; 8] = [Create, Delete, Find, List, Show, Start, Stop, Update];

impl TryFrom<&String> for CapabilityType {
  type Error = String;

  fn try_from(value: &String) -> Result<Self, Self::Error> {
    match value.as_str() {
      CREATE => Ok(Create),
      DELETE => Ok(Delete),
      FIND => Ok(Find),
      LIST => Ok(List),
      SHOW => Ok(Show),
      START => Ok(Start),
      STOP => Ok(Stop),
      UPDATE => Ok(Update),
      _ => Err(format!("unrecognized capability type {}", value)),
    }
  }
}

impl From<&str> for CapabilityType {
  fn from(value: &str) -> Self {
    match value {
      CREATE => Create,
      DELETE => Delete,
      FIND => Find,
      LIST => List,
      SHOW => Show,
      START => Start,
      STOP => Stop,
      UPDATE => Update,
      _ => panic!("unrecognized capability type {}", value),
    }
  }
}

impl CapabilityType {
  fn command_name(&self) -> &'static str {
    match self {
      Create => CREATE,
      Delete => DELETE,
      Find => FIND,
      List => LIST,
      Show => SHOW,
      Start => START,
      Stop => STOP,
      Update => UPDATE,
    }
  }

  fn command_id(&self) -> &'static str {
    self.command_name()
  }

  fn command_alias(&self) -> Option<&'static str> {
    match self {
      Create => None,
      Delete => None,
      Find => Some("f"),
      List => Some("l"),
      Show => Some("s"),
      Start => None,
      Stop => None,
      Update => None,
    }
  }

  pub(crate) fn command_target_arguments(&self, subject: &dyn Subject) -> Vec<Arg> {
    match self {
      Create => vec![target_argument(subject, None)],
      Delete => vec![target_argument(subject, None)],
      Find => vec![query_argument(None)],
      List => vec![],
      Show => vec![target_argument(subject, None)],
      Start => vec![target_argument(subject, None)],
      Stop => vec![target_argument(subject, None)],
      Update => vec![target_argument(subject, None)],
    }
  }

  pub(crate) fn command_target_argument_ids(&self) -> &[&str] {
    match self {
      Create => &[TARGET_ARGUMENT],
      Delete => &[TARGET_ARGUMENT],
      Find => &[QUERY_ARGUMENT],
      List => &[],
      Show => &[TARGET_ARGUMENT],
      Start => &[TARGET_ARGUMENT],
      Stop => &[TARGET_ARGUMENT],
      Update => &[TARGET_ARGUMENT],
    }
  }
}

#[async_trait]
pub trait Capability {
  fn capability_type(&self) -> CapabilityType;

  fn clap_capability_command(&self, subject: &dyn Subject) -> Command;

  fn clap_flags(&self, subject: &dyn Subject) -> Vec<Arg>;

  fn long_about(&self) -> Option<String>;

  async fn execute_capability(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult;
}

#[async_trait]
pub(crate) trait CommandExecutor {
  async fn execute(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult;
}

pub(crate) struct DeclarativeCapability<'a> {
  pub(crate) capability_type: CapabilityType,
  pub(crate) command_about: String,
  pub(crate) command_long_about: Option<String>,
  pub(crate) command_after_help: Option<String>,
  pub(crate) command_after_long_help: Option<String>,
  pub(crate) command_executors: Vec<(FlagType, Box<&'a (dyn CommandExecutor + Send + Sync)>, Option<&'a str>)>,
  pub(crate) default_command_executor: Option<Box<&'a (dyn CommandExecutor + Send + Sync)>>,
  pub(crate) run_all_executors: bool,
  pub(crate) extra_arguments: Vec<Arg>,
  pub(crate) extra_flags: Vec<Arg>,
}

#[async_trait]
impl Capability for DeclarativeCapability<'_> {
  fn capability_type(&self) -> CapabilityType {
    self.capability_type.clone()
  }

  fn clap_capability_command(&self, subject: &dyn Subject) -> Command {
    let mut capability_command = Command::new(self.capability_type().command_id())
      .name(self.capability_type.command_name())
      .about(&self.command_about)
      .args(self.capability_type.command_target_arguments(subject))
      .args(self.clap_flags(subject))
      .args(&self.extra_arguments)
      .args(&self.extra_flags);
    if let Some(alias) = self.capability_type.command_alias() {
      capability_command = capability_command.alias(alias)
    }
    if let Some(ref long_about) = self.command_long_about {
      capability_command = capability_command.long_about(long_about)
    }
    if let Some(ref after_help) = self.command_after_help {
      capability_command = capability_command.after_help(after_help)
    }
    if let Some(ref after_long_help) = self.command_after_long_help {
      capability_command = capability_command.after_long_help(after_long_help)
    }
    capability_command
  }

  fn clap_flags(&self, subject: &dyn Subject) -> Vec<Arg> {
    self
      .command_executors
      .iter()
      .map(|(flag_type, _, long_help)| create_flag(flag_type, subject, long_help))
      .collect::<Vec<Arg>>()
  }

  fn long_about(&self) -> Option<String> {
    self.command_long_about.clone()
  }

  async fn execute_capability(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let mut matching_flag_found = false;
    for (flag_type, executor, _) in &self.command_executors {
      if matches.get_flag(flag_type.id()) {
        matching_flag_found = true;
        if !self.run_all_executors {
          break;
        }
        executor.execute(argument.clone(), sub_argument.clone(), matches, dsh_api_client).await?;
      }
    }
    if !matching_flag_found {
      if let Some(ref default_executor) = self.default_command_executor {
        default_executor.execute(argument.clone(), sub_argument.clone(), matches, dsh_api_client).await
      } else {
        Ok(())
      }
    } else {
      Ok(())
    }
  }
}

// impl DeclarativeCapability<'_> {
//   pub(crate) fn new(capability_type: CapabilityType, about: String) -> Self {
//     Self {
//       capability_type,
//       command_about: about,
//       command_long_about: None,
//       command_after_help: None,
//       command_after_long_help: None,
//       command_executors: vec![],
//       default_command_executor: None,
//       run_all_executors: false,
//       extra_arguments: vec![],
//       extra_flags: vec![],
//     }
//   }
//
//   pub(crate) fn set_long_about(&mut self, long_about: String) -> &Self {
//     self.command_long_about = Some(long_about);
//     self
//   }
//
//   pub(crate) fn set_after_help(&mut self, after_help: String) -> &Self {
//     self.command_after_help = Some(after_help);
//     self
//   }
//
//   pub(crate) fn set_after_long_help(&mut self, after_long_help: String) -> &Self {
//     self.command_after_long_help = Some(after_long_help);
//     self
//   }
//
//   pub(crate) fn add_extra_argument(&mut self, extra_argument: Arg) -> &Self {
//     self.extra_arguments.push(extra_argument);
//     self
//   }
//
//   pub(crate) fn add_extra_flag(&mut self, flag: Arg) -> &Self {
//     self.extra_flags.push(flag);
//     self
//   }
// }
