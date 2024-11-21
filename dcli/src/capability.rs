use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};
use std::fmt::{Display, Formatter};

use crate::capability::CapabilityType::*;
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};

pub(crate) const CREATE: &str = "create";
pub(crate) const DELETE: &str = "delete";
pub(crate) const FIND: &str = "find";
pub(crate) const DIFF: &str = "diff";
pub(crate) const LIST: &str = "list";
pub(crate) const NEW: &str = "new";
pub(crate) const SHOW: &str = "show";
pub(crate) const START: &str = "start";
pub(crate) const STOP: &str = "stop";
pub(crate) const UPDATE: &str = "update";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CapabilityType {
  Create,
  Delete,
  Diff,
  Find,
  List,
  New,
  Show,
  Start,
  Stop,
  Update,
}

pub(crate) static ALL_CAPABILITY_TYPES: [CapabilityType; 10] = [Create, Delete, Diff, Find, List, New, Show, Start, Stop, Update];

impl TryFrom<&str> for CapabilityType {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      CREATE => Ok(Create),
      DELETE => Ok(Delete),
      DIFF => Ok(Diff),
      FIND => Ok(Find),
      LIST => Ok(List),
      NEW => Ok(New),
      SHOW => Ok(Show),
      START => Ok(Start),
      STOP => Ok(Stop),
      UPDATE => Ok(Update),
      _ => Err(format!("unrecognized capability type {}", value)),
    }
  }
}

impl CapabilityType {
  pub(crate) fn command_alias(&self) -> Option<&'static str> {
    match self {
      Create => None,
      Delete => None,
      Diff => None,
      Find => Some("f"),
      List => Some("l"),
      New => None,
      Show => Some("s"),
      Start => None,
      Stop => None,
      Update => None,
    }
  }
}

impl Display for CapabilityType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Create => CREATE,
        Delete => DELETE,
        Diff => DIFF,
        Find => FIND,
        List => LIST,
        New => NEW,
        Show => SHOW,
        Start => START,
        Stop => STOP,
        Update => UPDATE,
      }
    )
  }
}

#[async_trait]
pub trait Capability {
  fn clap_capability_command(&self, subject: &dyn Subject) -> Command;

  fn clap_flags(&self, subject: &str) -> Vec<Arg>;

  fn long_about(&self) -> Option<String>;

  fn command_target_argument_ids(&self) -> Vec<String>;

  async fn execute_capability(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult;

  // name: "delete",
  // about: Some(
  //     StyledStr(
  //         "Delete target configuration.",
  //     ),
  // ),
  // long_about: Some(
  //     StyledStr(
  //         "Delete a target configuration. You will be prompted for the target's platform and tenant, and you need to explicitly confirm the action.",
  //     ),
  // ),

  // long_flag: None,
  // short_flag: None,
  // display_name: None,
  // bin_name: None,
  // author: None,
  // version: None,
  // long_version: None,
  // before_help: None,
  // before_long_help: None,
  // after_help: None,
  // after_long_help: None,
  // aliases: [],
  // short_flag_aliases: [],
  // long_flag_aliases: [],
  // usage_str: None,
  // usage_name: None,
  // help_str: None,
  // disp_ord: Some(
  //     0,
  // ),
  // template: None,
  // settings: AppFlags(
  //     0,
  // ),
  // g_settings: AppFlags(
  //     0,
  // ),
  // args: MKeyMap {
  //     args: [],
  //     keys: [],
  // },
  // subcommands: [],
  // groups: [],
  // current_help_heading: None,
  // current_disp_ord: Some(
  //     0,
  // ),
  // subcommand_value_name: None,
  // subcommand_heading: None,
  // external_value_parser: None,
  // long_help_exists: false,
  // deferred: None,
  // app_ext: Extensions {
  //     extensions: FlatMap {
  //         keys: [],
  //         values: [],
  //     },
  // },
}

#[async_trait]
pub(crate) trait CommandExecutor {
  // fn get_flag(&self);
  // fn get_help(&self);
  // fn get_long_help(&self);

  async fn execute(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult;
}
