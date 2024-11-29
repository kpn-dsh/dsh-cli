use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};
use std::fmt::{Display, Formatter};

use crate::capability::CapabilityType::*;
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};

pub(crate) const CREATE: &str = "create";
pub(crate) const DEFAULT: &str = "default";
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
  Default,
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

pub(crate) static ALL_CAPABILITY_TYPES: [CapabilityType; 11] = [Create, Default, Delete, Diff, Find, List, New, Show, Start, Stop, Update];

impl TryFrom<&str> for CapabilityType {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      CREATE => Ok(Create),
      DEFAULT => Ok(Default),
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
      Default => None,
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
    match self {
      Create => write!(f, "{}", CREATE),
      Default => write!(f, "{}", DEFAULT),
      Delete => write!(f, "{}", DELETE),
      Diff => write!(f, "{}", DIFF),
      Find => write!(f, "{}", FIND),
      List => write!(f, "{}", LIST),
      New => write!(f, "{}", NEW),
      Show => write!(f, "{}", SHOW),
      Start => write!(f, "{}", START),
      Stop => write!(f, "{}", STOP),
      Update => write!(f, "{}", UPDATE),
    }
  }
}

#[async_trait]
pub trait Capability {
  fn clap_capability_command(&self, subject: &dyn Subject) -> Command;

  fn clap_flags(&self, subject: &str) -> Vec<Arg>;

  fn long_about(&self) -> Option<String>;

  fn command_target_argument_ids(&self) -> Vec<String>;

  async fn execute_capability(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult;
}

#[async_trait]
pub(crate) trait CommandExecutor {
  async fn execute(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult;
}
