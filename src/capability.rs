use crate::context::Context;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};

pub(crate) const CREATE_COMMAND: &str = "create";
pub(crate) const DEFAULT_COMMAND: &str = "default";
pub(crate) const DELETE_COMMAND: &str = "delete";
// pub(crate) const DIFF_COMMAND: &str = "diff";
pub(crate) const FIND_COMMAND: &str = "find";
pub(crate) const LIST_COMMAND: &str = "list";
pub(crate) const NEW_COMMAND: &str = "new";
pub(crate) const OPEN_COMMAND: &str = "open";
pub(crate) const SHOW_COMMAND: &str = "show";
// pub(crate) const START_COMMAND: &str = "start";
// pub(crate) const STOP_COMMAND: &str = "stop";
pub(crate) const UPDATE_COMMAND: &str = "update";

pub(crate) const CREATE_COMMAND_PAIR: (&str, &str) = (CREATE_COMMAND, "");
pub(crate) const DEFAULT_COMMAND_PAIR: (&str, &str) = (DEFAULT_COMMAND, "");
pub(crate) const DELETE_COMMAND_PAIR: (&str, &str) = (DELETE_COMMAND, "");
// pub(crate) const DIFF_COMMAND_PAIR: (&str, &str) = (DIFF_COMMAND, "");
pub(crate) const FIND_COMMAND_PAIR: (&str, &str) = (FIND_COMMAND, "f");
pub(crate) const LIST_COMMAND_PAIR: (&str, &str) = (LIST_COMMAND, "l");
pub(crate) const NEW_COMMAND_PAIR: (&str, &str) = (NEW_COMMAND, "");
pub(crate) const OPEN_COMMAND_PAIR: (&str, &str) = (OPEN_COMMAND, "o");
pub(crate) const SHOW_COMMAND_PAIR: (&str, &str) = (SHOW_COMMAND, "s");
// pub(crate) const START_COMMAND_PAIR: (&str, &str) = (START_COMMAND, "");
// pub(crate) const STOP_COMMAND_PAIR: (&str, &str) = (STOP_COMMAND, "");
pub(crate) const UPDATE_COMMAND_PAIR: (&str, &str) = (UPDATE_COMMAND, "");

#[async_trait]
pub trait Capability {
  fn clap_capability_command(&self, subject_command: &str) -> Command;

  fn clap_flags(&self, subject: &str) -> Vec<Arg>;

  fn long_about(&self) -> Option<String>;

  fn command_target_argument_ids(&self) -> Vec<String>;

  async fn execute_capability(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult;
}

#[async_trait]
pub(crate) trait CommandExecutor {
  async fn execute(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult;
}
