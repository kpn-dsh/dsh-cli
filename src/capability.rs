use crate::context::Context;
use crate::subject::Requirements;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};
use dsh_api::dsh_api_client::DshApiClient;

pub(crate) const COPY_COMMAND: &str = "copy";
pub(crate) const CREATE_COMMAND: &str = "create";
pub(crate) const CREATE_COMMAND_ALIAS: &str = "c";
pub(crate) const DEFAULT_COMMAND: &str = "default";
pub(crate) const DEFAULT_COMMAND_ALIAS: &str = "d";
pub(crate) const DELETE_COMMAND: &str = "delete";
pub(crate) const DUPLICATE_COMMAND: &str = "duplicate";
pub(crate) const EDIT_COMMAND: &str = "edit";
pub(crate) const EXPORT_COMMAND: &str = "export";
pub(crate) const EXPORT_COMMAND_ALIAS: &str = "e";
pub(crate) const FETCH_COMMAND: &str = "fetch";
pub(crate) const FIND_COMMAND: &str = "find";
pub(crate) const FIND_COMMAND_ALIAS: &str = "f";
#[cfg(feature = "manage")]
pub(crate) const GRANT_COMMAND: &str = "grant";
pub(crate) const LIST_COMMAND: &str = "list";
pub(crate) const LIST_COMMAND_ALIAS: &str = "l";
pub(crate) const OPEN_COMMAND: &str = "open";
pub(crate) const OPEN_COMMAND_ALIAS: &str = "o";
pub(crate) const RESTART_COMMAND: &str = "restart";
#[cfg(feature = "manage")]
pub(crate) const REVOKE_COMMAND: &str = "revoke";
pub(crate) const SET_COMMAND: &str = "set";
pub(crate) const SHOW_COMMAND: &str = "show";
pub(crate) const SHOW_COMMAND_ALIAS: &str = "s";
pub(crate) const START_COMMAND: &str = "start";
pub(crate) const STOP_COMMAND: &str = "stop";
pub(crate) const UNSET_COMMAND: &str = "unset";
pub(crate) const UPDATE_COMMAND: &str = "update";

#[async_trait]
pub trait Capability {
  fn clap_capability_command(&self, subject_command: &str) -> Command;

  fn clap_flags(&self, subject: &str) -> Vec<Arg>;

  fn long_about(&self) -> Option<String>;

  fn command_target_argument_ids(&self) -> Vec<String>;

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements;

  async fn execute_capability_with_client(
    &self,
    argument: Option<String>,
    sub_argument: Option<String>,
    matches: &ArgMatches,
    dsh_api_client: &DshApiClient,
    context: &Context,
  ) -> DshCliResult;

  async fn execute_capability_without_client(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult;
}

#[async_trait]
pub(crate) trait CommandExecutor {
  #[allow(unused_variables)]
  async fn execute_with_client(
    &self,
    argument: Option<String>,
    sub_argument: Option<String>,
    matches: &ArgMatches,
    dsh_api_client: &DshApiClient,
    context: &Context,
  ) -> DshCliResult {
    unreachable!()
  }

  #[allow(unused_variables)]
  async fn execute_without_client(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    unreachable!()
  }

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements;
}
