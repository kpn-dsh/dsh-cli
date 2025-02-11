use async_trait::async_trait;
use clap::{ArgMatches, Command};

use crate::capability::{Capability, LIST_COMMAND};
use crate::context::Context;
use crate::formatters::OutputFormat;
use crate::DshCliResult;

pub struct Requirements {
  pub needs_target: bool,
  pub needs_dsh_api_client: bool,
  pub default_output_format: Option<OutputFormat>,
}

impl Requirements {
  pub fn new(needs_target: bool, needs_dsh_api_client: bool, default_output_format: Option<OutputFormat>) -> Self {
    Self { needs_target, needs_dsh_api_client, default_output_format }
  }

  pub fn needs_platform(&self) -> bool {
    self.needs_target || self.needs_dsh_api_client
  }

  pub fn needs_tenant_name(&self) -> bool {
    self.needs_target || self.needs_dsh_api_client
  }
}

// A subject represents something that the tool can act upon, such as an Application,
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

  fn requirements(&self, sub_matches: &ArgMatches) -> Requirements;

  // Is called at most once and only if capability command is used
  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)>;

  // Returns list of capabilities that are supported for this Subject
  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)>;

  // Called once by main when building the clap command
  fn clap_subject_command(&self) -> (String, Command) {
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
  fn clap_list_shortcut_command(&self) -> Option<(String, Command)> {
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

  async fn execute_subject_command<'a>(&self, subject_matches: &'a ArgMatches, context: &Context) -> DshCliResult {
    let (capability_command_id, capability_matches) = subject_matches.subcommand().unwrap_or_else(|| unreachable!());
    let capability = self.capability(capability_command_id).unwrap_or_else(|| unreachable!());
    let arguments = capability.command_target_argument_ids();
    let argument = arguments.first().and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    let sub_argument = arguments.get(1).and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    capability.execute_capability(argument, sub_argument, capability_matches, context).await
  }

  async fn execute_subject_list_shortcut<'a>(&self, matches: &'a ArgMatches, context: &Context) -> DshCliResult {
    self
      .capability(LIST_COMMAND)
      .unwrap_or_else(|| unreachable!())
      .execute_capability(None, None, matches, context)
      .await
  }
}
