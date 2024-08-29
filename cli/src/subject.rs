use std::collections::HashMap;

use async_trait::async_trait;
use clap::{ArgMatches, Command};

use trifonius_dsh_api::DshApiClient;

use crate::capability::{Capability, CapabilityType, ALL_CAPABILITY_TYPES};
use crate::CommandResult;

#[async_trait]
pub trait Subject {
  fn subject(&self) -> &'static str;

  fn subject_first_upper(&self) -> &'static str;

  fn subject_command_about(&self) -> String;

  fn subject_command_long_about(&self) -> String;

  fn subject_command_name(&self) -> &str;

  fn subject_command_alias(&self) -> Option<&str> {
    None
  }

  fn capabilities(&self) -> HashMap<CapabilityType, &Box<(dyn Capability + Send + Sync)>>;

  async fn execute_subject_command<'a>(&self, matches: &'a ArgMatches, dsh_api_client: &'a DshApiClient<'_>) -> CommandResult {
    match matches.subcommand() {
      Some((capability_command_id, matches)) => match CapabilityType::try_from(capability_command_id) {
        Ok(ref capability_type) => match self.capabilities().get(capability_type) {
          Some(capability) => {
            let arguments = capability_type.command_target_argument_ids();
            let argument = arguments.first().and_then(|argument| matches.get_one::<String>(argument)).cloned();
            let sub_argument = arguments.get(1).and_then(|argument| matches.get_one::<String>(argument)).cloned();
            capability.execute_capability(argument, sub_argument, matches, dsh_api_client).await
          }
          None => unreachable!(),
        },
        Err(_) => unreachable!(),
      },
      None => unreachable!(),
    }
  }

  async fn execute_subject_list_shortcut<'a>(&self, matches: &'a ArgMatches, dsh_api_client: &'a DshApiClient<'_>) -> CommandResult {
    match self.capabilities().get(&CapabilityType::List) {
      Some(capability) => capability.execute_capability(None, None, matches, dsh_api_client).await,
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
  let subject_command_name = subject.subject_command_name();
  let mut subject_command = Command::new(subject_command_name.to_string())
    .about(subject.subject_command_about())
    .long_about(subject.subject_command_long_about())
    .arg_required_else_help(true)
    .subcommands(capability_subcommands);
  if let Some(alias) = subject.subject_command_alias() {
    subject_command = subject_command.alias(alias.to_string())
  }
  (subject_command_name.to_string(), subject_command)
}

pub(crate) fn clap_subject_list_shortcut(subject: &dyn Subject) -> Option<(String, Command)> {
  if let Some(list_capability) = subject.capabilities().get(&CapabilityType::List) {
    let list_flags = list_capability.clap_flags(subject);
    let subject_list_shortcut_name = format!("{}s", subject.subject_command_name());
    let mut subject_list_shortcut = Command::new(subject_list_shortcut_name.to_string())
      .about(subject.subject_command_about())
      .args(list_flags)
      .hide(true);
    if let Some(alias) = subject.subject_command_alias() {
      subject_list_shortcut = subject_list_shortcut.alias(format!("{}s", alias))
    }
    if let Some(long_about) = list_capability.long_about() {
      subject_list_shortcut = subject_list_shortcut.long_about(long_about)
    }
    Some((subject_list_shortcut_name.to_string(), subject_list_shortcut))
  } else {
    None
  }
}
