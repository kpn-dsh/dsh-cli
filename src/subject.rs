use crate::authentication::AuthenticationMethod;
use crate::capability::{Capability, LIST_COMMAND};
use crate::context::Context;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{ArgMatches, Command};
use dsh_api::dsh_api_client::DshApiClient;
use std::fmt::{Display, Formatter};

/// Defines the requirements for a command/capability
#[derive(PartialEq)]
pub(crate) struct Requirements {
  /// If `true` the command allows the `--all-tenants` flag to be used.
  all_tenants_allowed: bool,
  /// If provided specifies the mandatory authentication method.
  mandatory_authentication_method: Option<AuthenticationMethod>,
  /// If `true` the command requires an api client.
  needs_dsh_api_client: bool,
}

impl Requirements {
  /// Create `Requirements` struct
  ///
  /// ## Parameters
  /// * `all_tenants_allowed` - If `true` the command allows the `--all-tenants` flag to be used.
  /// * `mandatory_authentication_method` - If provided this parameter specifies the mandatory
  ///   authentication method.
  /// * `needs_dsh_api_client` - If `true` the command requires an api client.
  pub(crate) fn new(all_tenants_allowed: bool, mandatory_authentication_method: Option<AuthenticationMethod>, needs_dsh_api_client: bool) -> Self {
    Self { all_tenants_allowed, mandatory_authentication_method, needs_dsh_api_client }
  }

  /// Standard `Requirements` with api client
  ///
  /// * The command allows the `--all-tenants` flag to be used.
  /// * All authentication methods allowed.
  /// * Command requires an api client.
  pub(crate) fn standard_with_api() -> Self {
    Self::new(true, None, true)
  }

  /// Standard `Requirements` without api client
  ///
  /// * The command allows the `--all-tenants` flag to be used.
  /// * Although not really relevant, all authentication methods are allowed.
  /// * Command does not require an api client.
  pub(crate) fn standard_without_api() -> Self {
    Self::new(true, None, false)
  }

  /// Checks whether the `--all-tenants` flag is allowed.
  pub(crate) fn all_tenants_allowed(&self) -> bool {
    self.all_tenants_allowed
  }

  /// Checks whether the command requires an api client.
  pub(crate) fn needs_dsh_api_client(&self) -> bool {
    self.needs_dsh_api_client
  }

  /// Checks whether an authentication method is allowed
  ///
  /// ## Parameters
  /// * `authentication_method` - The authentication method to check.
  pub(crate) fn authentication_method_allowed(&self, authentication_method: &AuthenticationMethod) -> bool {
    self
      .mandatory_authentication_method
      .as_ref()
      .is_none_or(|mandatory_authentication_method| mandatory_authentication_method == authentication_method)
  }
}

impl Display for Requirements {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "needs_dsh_api_client: {}", self.needs_dsh_api_client)
  }
}

// A subject represents something that the dsh tool can act upon, such as an Application,
// a Secret, a Target or the API itself.
// The subject is always selected by the first command on the command line.
#[async_trait]
pub(crate) trait Subject {
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

  fn support_list_shortcut(&self) -> bool {
    true
  }

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
      if self.support_list_shortcut() {
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
    } else {
      None
    }
  }

  fn requirements(&self, subject_matches: &ArgMatches) -> Requirements {
    let (capability_command_id, capability_matches) = subject_matches.subcommand().unwrap_or_else(|| unreachable!("no subcommand specified"));
    let capability = self
      .capability(capability_command_id)
      .unwrap_or_else(|| unreachable!("capability '{}' not recognized", capability_command_id));
    capability.requirements(capability_matches)
  }

  fn requirements_list_shortcut(&self, matches: &ArgMatches) -> Requirements {
    self.capability(LIST_COMMAND).unwrap_or_else(|| unreachable!()).requirements(matches)
  }

  async fn execute_subject_command_with_client<'a>(&self, subject_matches: &'a ArgMatches, dsh_api_client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    let (capability_command_id, capability_matches) = subject_matches.subcommand().unwrap_or_else(|| unreachable!());
    let capability = self.capability(capability_command_id).unwrap_or_else(|| unreachable!());
    let arguments = capability.command_target_argument_ids();
    let argument = arguments.first().and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    let sub_argument = arguments.get(1).and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    capability
      .execute_capability_with_client(argument, sub_argument, capability_matches, dsh_api_client, context)
      .await
  }

  async fn execute_subject_command_without_client<'a>(&self, subject_matches: &'a ArgMatches, context: &Context) -> DshCliResult<()> {
    let (capability_command_id, capability_matches) = subject_matches.subcommand().unwrap_or_else(|| unreachable!());
    let capability = self.capability(capability_command_id).unwrap_or_else(|| unreachable!());
    let arguments = capability.command_target_argument_ids();
    let argument = arguments.first().and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    let sub_argument = arguments.get(1).and_then(|argument| capability_matches.get_one::<String>(argument)).cloned();
    capability
      .execute_capability_without_client(argument, sub_argument, capability_matches, context)
      .await
  }

  async fn execute_subject_list_shortcut_with_client<'a>(&self, matches: &'a ArgMatches, dsh_api_client: &DshApiClient, context: &Context) -> DshCliResult<()> {
    self
      .capability(LIST_COMMAND)
      .unwrap_or_else(|| unreachable!())
      .execute_capability_with_client(None, None, matches, dsh_api_client, context)
      .await
  }

  async fn execute_subject_list_shortcut_without_client<'a>(&self, matches: &'a ArgMatches, context: &Context) -> DshCliResult<()> {
    self
      .capability(LIST_COMMAND)
      .unwrap_or_else(|| unreachable!())
      .execute_capability_without_client(None, None, matches, context)
      .await
  }
}
