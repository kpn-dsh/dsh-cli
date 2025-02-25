use crate::context::Context;
use crate::subject::Requirements;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};

pub(crate) const DEFAULT_COMMAND: &str = "default";
pub(crate) const DEFAULT_COMMAND_ALIAS: &str = "d";
pub(crate) const DELETE_COMMAND: &str = "delete";
pub(crate) const EXPORT_COMMAND: &str = "export";
pub(crate) const FETCH_COMMAND: &str = "fetch";
pub(crate) const FIND_COMMAND: &str = "find";
pub(crate) const FIND_COMMAND_ALIAS: &str = "f";
pub(crate) const LIST_COMMAND: &str = "list";
pub(crate) const LIST_COMMAND_ALIAS: &str = "l";
pub(crate) const NEW_COMMAND: &str = "new";
pub(crate) const OPEN_COMMAND: &str = "open";
pub(crate) const OPEN_COMMAND_ALIAS: &str = "o";
pub(crate) const SET_COMMAND: &str = "set";
pub(crate) const SHOW_COMMAND: &str = "show";
pub(crate) const SHOW_COMMAND_ALIAS: &str = "s";
pub(crate) const UNSET_COMMAND: &str = "unset";
pub(crate) const UPDATE_COMMAND: &str = "update";

#[async_trait]
pub trait Capability {
  fn clap_capability_command(&self, subject_command: &str) -> Command;

  fn clap_flags(&self, subject: &str) -> Vec<Arg>;

  fn long_about(&self) -> Option<String>;

  fn command_target_argument_ids(&self) -> Vec<String>;

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements;

  async fn execute_capability(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult;
}

#[async_trait]
pub(crate) trait CommandExecutor {
  async fn execute(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult;

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements;
}
