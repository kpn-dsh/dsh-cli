use crate::context::Context;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};

pub(crate) const DELETE_COMMAND: &str = "delete";
pub(crate) const EXPORT_COMMAND: &str = "export";
pub(crate) const FETCH_COMMAND: &str = "fetch";
pub(crate) const FIND_COMMAND: &str = "find";
pub(crate) const LIST_COMMAND: &str = "list";
pub(crate) const NEW_COMMAND: &str = "new";
pub(crate) const OPEN_COMMAND: &str = "open";
pub(crate) const SET_COMMAND: &str = "set";
pub(crate) const SHOW_COMMAND: &str = "show";
pub(crate) const UNSET_COMMAND: &str = "unset";
pub(crate) const UPDATE_COMMAND: &str = "update";

pub(crate) const DELETE_COMMAND_PAIR: (&str, &str) = (DELETE_COMMAND, "");
pub(crate) const EXPORT_COMMAND_PAIR: (&str, &str) = (EXPORT_COMMAND, "");
pub(crate) const FETCH_COMMAND_PAIR: (&str, &str) = (FETCH_COMMAND, "");
pub(crate) const FIND_COMMAND_PAIR: (&str, &str) = (FIND_COMMAND, "f");
pub(crate) const LIST_COMMAND_PAIR: (&str, &str) = (LIST_COMMAND, "l");
pub(crate) const NEW_COMMAND_PAIR: (&str, &str) = (NEW_COMMAND, "");
pub(crate) const OPEN_COMMAND_PAIR: (&str, &str) = (OPEN_COMMAND, "o");
pub(crate) const SET_COMMAND_PAIR: (&str, &str) = (SET_COMMAND, "");
pub(crate) const SHOW_COMMAND_PAIR: (&str, &str) = (SHOW_COMMAND, "s");
pub(crate) const UNSET_COMMAND_PAIR: (&str, &str) = (UNSET_COMMAND, "");
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
