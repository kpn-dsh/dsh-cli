use crate::capability::{Capability, CommandExecutor};
use crate::context::Context;
use crate::filter_flags::{create_filter_flag, FilterFlagType};
use crate::flags::{create_flag, FlagType};
use crate::modifier_flags::{create_modifier_flag, ModifierFlagType};
use crate::subject::Requirements;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};
use dsh_api::dsh_api_client::DshApiClient;

pub struct CapabilityBuilder<'a> {
  capability_command_name: String,
  capability_command_alias: Option<String>,
  about: String,
  long_about: Option<String>,
  subcommands: Vec<Command>,
  executors: Vec<(FlagType, &'a (dyn CommandExecutor + Send + Sync), Option<String>)>,
  default_executor: &'a (dyn CommandExecutor + Send + Sync),
  target_arguments: Vec<Arg>,
  extra_arguments: Vec<Arg>,
  filter_flags: Vec<(FilterFlagType, Option<String>)>,
  modifier_flags: Vec<(ModifierFlagType, Option<String>)>,
}

impl<'a> CapabilityBuilder<'a> {
  /// # Create a new `CapabilityBuilder`
  ///
  /// ## Parameters
  /// * `capability_type` -
  /// * `about` - help text printed when -h flag is provided
  pub fn new(command: &str, alias: Option<&str>, default_executor: &'a (dyn CommandExecutor + Send + Sync), about: impl Into<String>) -> Self {
    Self {
      capability_command_name: command.to_string(),
      capability_command_alias: alias.map(|alias| alias.to_string()),
      about: about.into(),
      long_about: None,
      subcommands: vec![],
      executors: vec![],
      default_executor,
      target_arguments: vec![],
      extra_arguments: vec![],
      filter_flags: vec![],
      modifier_flags: vec![],
    }
  }

  /// # Set long help text
  ///
  /// Set the long help text, which is shown when the `--help` flag was provided.
  ///
  /// ## Parameters
  /// * `long_about` - long help text
  pub fn set_long_about(mut self, long_about: impl Into<String>) -> Self {
    self.long_about = Some(long_about.into());
    self
  }

  pub fn _add_subcommand(mut self, subcommand: Command) -> Self {
    self.subcommands.push(subcommand);
    self
  }

  pub fn add_subcommands(mut self, subcommands: Vec<Command>) -> Self {
    for subcommand in subcommands {
      self.subcommands.push(subcommand);
    }
    self
  }

  pub fn add_command_executor(mut self, flag_type: FlagType, executor: &'a (dyn CommandExecutor + Send + Sync), long_help: Option<String>) -> Self {
    self.executors.push((flag_type, executor, long_help));
    self
  }

  pub fn add_command_executors(mut self, executors: Vec<(FlagType, &'a (dyn CommandExecutor + Send + Sync), Option<String>)>) -> Self {
    for (flag_type, executor, long_help) in executors {
      self.executors.push((flag_type, executor, long_help))
    }
    self
  }

  pub fn add_target_argument(mut self, argument: Arg) -> Self {
    self.target_arguments.push(argument);
    self
  }

  pub fn _add_target_arguments(mut self, mut arguments: Vec<Arg>) -> Self {
    self.target_arguments.append(&mut arguments);
    self
  }

  pub fn add_extra_argument(mut self, argument: Arg) -> Self {
    self.extra_arguments.push(argument);
    self
  }

  pub fn add_extra_arguments(mut self, mut arguments: Vec<Arg>) -> Self {
    self.extra_arguments.append(&mut arguments);
    self
  }

  pub fn add_filter_flag(mut self, flag_type: FilterFlagType, long_help: Option<String>) -> Self {
    self.filter_flags.push((flag_type, long_help));
    self
  }

  pub fn add_filter_flags(mut self, flags: Vec<(FilterFlagType, Option<String>)>) -> Self {
    for (flag_type, long_help) in flags {
      self.filter_flags.push((flag_type, long_help))
    }
    self
  }

  pub fn add_modifier_flag(mut self, flag_type: ModifierFlagType, long_help: Option<String>) -> Self {
    self.modifier_flags.push((flag_type, long_help));
    self
  }

  pub fn _add_modifier_flags(mut self, flags: Vec<(ModifierFlagType, Option<String>)>) -> Self {
    for (flag_type, long_help) in flags {
      self.modifier_flags.push((flag_type, long_help))
    }
    self
  }
}

#[async_trait]
impl Capability for CapabilityBuilder<'_> {
  fn clap_capability_command(&self, subject_command: &str) -> Command {
    let mut capability_command = Command::new(self.capability_command_name.clone())
      .name(self.capability_command_name.clone())
      .about(&self.about)
      .subcommands(&self.subcommands)
      .args(&self.target_arguments)
      .args(self.clap_flags(subject_command))
      .args(&self.extra_arguments)
      .arg_required_else_help(!self.subcommands.is_empty());
    if let Some(ref alias) = self.capability_command_alias {
      capability_command = capability_command.alias(alias)
    }
    if let Some(ref long_about) = self.long_about {
      capability_command = capability_command.long_about(long_about)
    }
    capability_command
  }

  fn clap_flags(&self, subject: &str) -> Vec<Arg> {
    [
      self
        .executors
        .iter()
        .map(|(flag_type, _, long_help)| create_flag(flag_type, subject, long_help.as_deref()))
        .collect::<Vec<_>>(),
      self
        .filter_flags
        .iter()
        .map(|(flag_type, long_help)| create_filter_flag(flag_type, long_help.as_deref()))
        .collect::<Vec<_>>(),
      self
        .modifier_flags
        .iter()
        .map(|(flag_type, _)| create_modifier_flag(flag_type, subject))
        .collect::<Vec<_>>(),
    ]
    .concat()
  }

  fn long_about(&self) -> Option<String> {
    self.long_about.clone()
  }

  fn command_target_argument_ids(&self) -> Vec<String> {
    self.target_arguments.clone().iter().map(|arg| arg.get_id().to_string()).collect::<Vec<_>>()
  }

  fn requirements(&self, matches: &ArgMatches) -> Requirements {
    for (flag_type, executor, _) in &self.executors {
      if matches.get_flag(flag_type.id()) {
        return executor.requirements(matches);
      }
    }
    self.default_executor.requirements(matches)
  }

  async fn execute_capability_with_client(
    &self,
    argument: Option<String>,
    sub_argument: Option<String>,
    matches: &ArgMatches,
    dsh_api_client: &DshApiClient,
    context: &Context,
  ) -> DshCliResult {
    for (flag_type, executor, _) in &self.executors {
      if matches.get_flag(flag_type.id()) {
        return executor
          .execute_with_client(argument.clone(), sub_argument.clone(), matches, dsh_api_client, context)
          .await;
      }
    }
    self
      .default_executor
      .execute_with_client(argument.clone(), sub_argument.clone(), matches, dsh_api_client, context)
      .await
  }

  async fn execute_capability_without_client(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    for (flag_type, executor, _) in &self.executors {
      if matches.get_flag(flag_type.id()) {
        return executor.execute_without_client(argument.clone(), sub_argument.clone(), matches, context).await;
      }
    }
    self
      .default_executor
      .execute_without_client(argument.clone(), sub_argument.clone(), matches, context)
      .await
  }
}
