use crate::capability::{Capability, LIST_COMMAND};
use crate::context::Context;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{ArgMatches, Command};
use dsh_api::dsh_api_client::DshApiClient;

#[derive(Debug, PartialEq)]
pub struct Requirements {
  needs_dsh_api_client: bool,
}

impl Requirements {
  pub fn new(needs_dsh_api_client: bool) -> Self {
    Self { needs_dsh_api_client }
  }

  pub fn standard_with_api() -> Self {
    Self::new(true)
  }

  pub fn standard_without_api() -> Self {
    Self::new(false)
  }

  pub fn needs_dsh_api_client(&self) -> bool {
    self.needs_dsh_api_client
  }
}

// A subject represents something that the dsh tool can act upon, such as an Application,
// a Secret, a Target or the API itself.
// The subject is always selected by the first command on the command line.
#[async_trait]
pub trait Subject {
  fn subject(&self) -> &'static str;

  fn subject_command_about(&self) -> String;

  fn subject_command_long_about(&self) -> String {
    self.subject_command_about()
  }

  fn subject_command_alias(&self) -> Option<&str> {
    None
  }

  // Is called at most once and only if capability command is used
  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)>;

  // Returns list of capabilities that are supported for this Subject
  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)>;

  // Called once by main when building the clap command
  fn subject_command(&self) -> (String, Command) {
    let mut capability_subcommands: Vec<Command> = vec![];
    for capability in self.capabilities() {
      let capability_command = capability.clap_capability_command(self.subject());
      capability_subcommands.push(capability_command)
    }
    let mut subject_command = Command::new(self.subject().to_string())
      .about(self.subject_command_about())
      .long_about(self.subject_command_long_about())
      .arg_required_else_help(true)
      .subcommands(capability_subcommands);
    if let Some(alias) = self.subject_command_alias() {
      subject_command = subject_command.alias(alias.to_string())
    }
    (self.subject().to_string(), subject_command)
  }

  // Called once by main when building the clap command
  fn subject_list_shortcut_command(&self) -> Option<(String, Command)> {
    if let Some(list_capability) = self.capability(LIST_COMMAND) {
      let list_shortcut_name = format!("{}s", self.subject());
      let list_flags = list_capability.clap_flags(self.subject());
      let mut list_shortcut_command = Command::new(list_shortcut_name.to_string())
        .about(self.subject_command_about())
        .args(list_flags)
        .hide(true);
      if let Some(alias) = self.subject_command_alias() {
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

  fn requirements(&self, subject_matches: &ArgMatches) -> Requirements {
    let (capability_command_id, capability_matches) = subject_matches.subcommand().unwrap_or_else(|| unreachable!());
    let capability = self.capability(capability_command_id).unwrap_or_else(|| unreachable!());
    capability.requirements(capability_matches)
  }

  fn requirements_list_shortcut(&self, matches: &ArgMatches) -> Requirements {
    self.capability(LIST_COMMAND).unwrap_or_else(|| unreachable!()).requirements(matches)
  }

  async fn execute_subject_command_with_client<'a>(&self, subject_matches: &'a ArgMatches, dsh_api_client: &DshApiClient, context: &Context) -> DshCliResult {
    let (capability_command_id, capability_matches) = subject_matches.subcommand().unwrap_or_else(|| unreachable!());
    let capability = self.capability(capability_command_id).unwrap_or_else(|| unreachable!());
    let arguments = capability.command_target_argument_ids();
    let argument = arguments.first().and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    let sub_argument = arguments.get(1).and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    capability
      .execute_capability_with_client(argument, sub_argument, capability_matches, dsh_api_client, context)
      .await
  }

  async fn execute_subject_command_without_client<'a>(&self, subject_matches: &'a ArgMatches, context: &Context) -> DshCliResult {
    let (capability_command_id, capability_matches) = subject_matches.subcommand().unwrap_or_else(|| unreachable!());
    let capability = self.capability(capability_command_id).unwrap_or_else(|| unreachable!());
    let arguments = capability.command_target_argument_ids();
    let argument = arguments.first().and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    let sub_argument = arguments.get(1).and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    capability
      .execute_capability_without_client(argument, sub_argument, capability_matches, context)
      .await
  }

  async fn execute_subject_list_shortcut_with_client<'a>(&self, matches: &'a ArgMatches, dsh_api_client: &DshApiClient, context: &Context) -> DshCliResult {
    self
      .capability(LIST_COMMAND)
      .unwrap_or_else(|| unreachable!())
      .execute_capability_with_client(None, None, matches, dsh_api_client, context)
      .await
  }

  async fn execute_subject_list_shortcut_without_client<'a>(&self, matches: &'a ArgMatches, context: &Context) -> DshCliResult {
    self
      .capability(LIST_COMMAND)
      .unwrap_or_else(|| unreachable!())
      .execute_capability_without_client(None, None, matches, context)
      .await
  }
}
