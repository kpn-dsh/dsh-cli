use std::collections::HashMap;

use async_trait::async_trait;
use clap::{ArgMatches, Command};

use crate::capability::{Capability, CapabilityType, ALL_CAPABILITY_TYPES};
use crate::{DcliContext, DcliResult};

#[async_trait]
pub trait Subject {
  fn subject(&self) -> &'static str;

  fn subject_first_upper(&self) -> &'static str;

  fn subject_command_about(&self) -> String;

  fn subject_command_long_about(&self) -> String {
    self.subject_command_about()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    None
  }

  fn requires_dsh_api_client(&self) -> bool;

  // Map of capabilities that are supported for this Subject
  fn capabilities(&self) -> HashMap<CapabilityType, &(dyn Capability + Send + Sync)>;

  async fn execute_subject_command<'a>(&self, matches: &'a ArgMatches, context: &DcliContext) -> DcliResult {
    match matches.subcommand() {
      Some((capability_command_id, matches)) => match CapabilityType::try_from(capability_command_id) {
        Ok(ref capability_type) => match self.capabilities().get(capability_type) {
          Some(capability) => {
            let arguments = capability_type.command_target_argument_ids();
            let argument = arguments.first().and_then(|argument| matches.get_one::<String>(argument)).cloned();
            let sub_argument = arguments.get(1).and_then(|argument| matches.get_one::<String>(argument)).cloned();
            capability.execute_capability(argument, sub_argument, matches, context).await
          }
          None => unreachable!(),
        },
        Err(_) => unreachable!(),
      },
      None => unreachable!(),
    }
  }

  async fn execute_subject_list_shortcut<'a>(&self, matches: &'a ArgMatches, context: &DcliContext) -> DcliResult {
    match self.capabilities().get(&CapabilityType::List) {
      Some(capability) => capability.execute_capability(None, None, matches, context).await,
      None => unreachable!(),
    }
  }
}

pub(crate) fn clap_subject_command(subject: &dyn Subject) -> (String, Command) {
  let mut capability_subcommands: Vec<Command> = vec![];
  for capability_type in &ALL_CAPABILITY_TYPES {
    if let Some(capability) = subject.capabilities().get(capability_type) {
      capability_subcommands.push(capability.clap_capability_command(subject))
    }
  }
  let mut subject_command = Command::new(subject.subject().to_string())
    .about(subject.subject_command_about())
    .long_about(subject.subject_command_long_about())
    .arg_required_else_help(true)
    .subcommands(capability_subcommands);
  if let Some(alias) = subject.subject_command_alias() {
    subject_command = subject_command.alias(alias.to_string())
  }
  (subject.subject().to_string(), subject_command)
}

pub(crate) fn clap_list_shortcut_command(subject: &dyn Subject) -> Option<(String, Command)> {
  if let Some(list_capability) = subject.capabilities().get(&CapabilityType::List) {
    let list_shortcut_name = format!("{}s", subject.subject());
    let list_flags = list_capability.clap_flags(subject.subject());
    let mut list_shortcut_command = Command::new(list_shortcut_name.to_string())
      .about(subject.subject_command_about())
      .args(list_flags)
      .hide(true);
    if let Some(alias) = subject.subject_command_alias() {
      list_shortcut_command = list_shortcut_command.alias(format!("{}s", alias))
    }
    if let Some(long_about) = list_capability.long_about() {
      list_shortcut_command = list_shortcut_command.long_about(long_about)
    }
    Some((list_shortcut_name, list_shortcut_command))
  } else {
    None
  }
}
