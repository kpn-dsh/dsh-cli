use async_trait::async_trait;
use clap::{builder, Arg, ArgAction, ArgMatches, Command};

use trifonius_dsh_api::{DshApiClient, DshApiError};

use crate::arguments::Flag;
use crate::CommandResult;

pub(crate) const CREATE_SUBCOMMAND: &str = "create";
pub(crate) const DELETE_SUBCOMMAND: &str = "delete";
pub(crate) const LIST_SUBCOMMAND: &str = "list";
pub(crate) const SHOW_SUBCOMMAND: &str = "show";
pub(crate) const UPDATE_SUBCOMMAND: &str = "update";

pub(crate) const TARGET_ARGUMENT: &str = "target-argument";

#[async_trait]
pub trait SubjectCommand {
  fn subject(&self) -> &'static str;

  fn subject_first_upper(&self) -> &'static str;

  fn about(&self) -> String;

  fn long_about(&self) -> String;

  fn alias(&self) -> Option<&str>;

  // Default implementation
  fn supports_create(&self) -> bool {
    false
  }

  // Default implementation
  fn supports_delete(&self) -> bool {
    false
  }

  // Default implementation
  fn supports_list(&self) -> bool {
    true
  }

  // Default implementation
  fn supports_list_shortcut(&self) -> bool {
    true
  }

  // Default implementation
  fn supports_show(&self) -> bool {
    true
  }

  // Default implementation
  fn supports_update(&self) -> bool {
    false
  }

  // Default implementation
  fn create_flags(&self) -> &'static [Flag] {
    &[]
  }

  // Default implementation
  fn delete_flags(&self) -> &'static [Flag] {
    &[]
  }

  // Default implementation
  fn list_flags(&self) -> &'static [Flag] {
    &[]
  }

  // Default implementation
  fn show_flags(&self) -> &'static [Flag] {
    &[]
  }

  // Default implementation
  fn update_flags(&self) -> &'static [Flag] {
    &[]
  }

  async fn list_all(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::All.option_not_available()
  }

  // Default implementation
  async fn list_allocation_status(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::AllocationStatus.option_not_available()
  }

  // Default implementation
  async fn list_configuration(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Configuration.option_not_available()
  }

  // Default implementation
  async fn list_default(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.list_all(matches, dsh_api_client).await
  }

  // Default implementation
  async fn list_ids(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Ids.option_not_available()
  }

  // Default implementation
  async fn list_tasks(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Tasks.option_not_available()
  }

  // Default implementation
  async fn list_usages(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Usage.option_not_available()
  }

  // Default implementation
  async fn list_values(&self, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Value.option_not_available()
  }

  // Default implementation
  async fn show_all(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::All.option_not_available()
  }

  // Default implementation
  async fn show_allocation_status(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::AllocationStatus.option_not_available()
  }

  // Default implementation
  async fn show_configuration(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Configuration.option_not_available()
  }

  // Default implementation
  async fn show_ids(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Ids.option_not_available()
  }

  // Default implementation
  async fn show_default(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Err("no default option available".to_string())
  }

  // Default implementation
  async fn show_tasks(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Tasks.option_not_available()
  }

  // Default implementation
  async fn show_usage(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Usage.option_not_available()
  }

  // Default implementation
  async fn show_value(&self, _target_id: &str, _matches: &ArgMatches, _dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    Flag::Value.option_not_available()
  }

  // Final
  fn create_command(&self) -> (String, Command) {
    let mut subcommands = vec![];
    if self.supports_create() {
      subcommands.push(self.create_subcommand())
    }
    if self.supports_delete() {
      subcommands.push(self.delete_subcommand())
    }
    if self.supports_list() {
      subcommands.push(self.list_subcommand())
    }
    if self.supports_show() {
      subcommands.push(self.show_subcommand())
    }
    if self.supports_update() {
      subcommands.push(self.update_subcommand())
    }
    let command_name = format!("{}", self.subject());
    let mut command = Command::new(&command_name)
      .about(self.about())
      .long_about(self.long_about())
      .arg_required_else_help(true)
      .subcommands(subcommands);
    if let Some(alias) = self.alias() {
      command = command.alias(format!("{}", alias))
    }
    (command_name, command)
  }

  // Final
  fn create_list_shortcut_command(&self) -> Option<(String, Command)> {
    if self.supports_list_shortcut() {
      let list_shortcut_command_name = format!("{}s", self.subject());
      let mut list_shortcut_command = Command::new(&list_shortcut_command_name)
        .about(self.about())
        .long_about(self.long_about())
        .args(self.flag_arguments(&self.list_flags()))
        .hide(true);
      if let Some(alias) = self.alias() {
        list_shortcut_command = list_shortcut_command.alias(format!("{}s", alias))
      }
      Some((list_shortcut_command_name, list_shortcut_command))
    } else {
      None
    }
  }

  // Final
  async fn run_command(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    match matches.subcommand() {
      // Some((CREATE_SUBCOMMAND, sub_matches)) => self.run_create_subcommand(sub_matches, dsh_api_client).await,
      // Some((DELETE_SUBCOMMAND, sub_matches)) => self.run_delete_subcommand(sub_matches, dsh_api_client).await,
      Some((LIST_SUBCOMMAND, sub_matches)) => self.run_list(sub_matches, dsh_api_client).await,
      Some((SHOW_SUBCOMMAND, sub_matches)) => self.run_show(sub_matches, dsh_api_client).await,
      // Some((UPDATE_SUBCOMMAND, sub_matches)) => self.run_update_subcommand(sub_matches, dsh_api_client).await,
      _ => unreachable!(),
    }
  }

  // Final
  async fn run_list_shortcut(&self, sub_matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    self.run_list(sub_matches, dsh_api_client).await
  }

  // Final
  async fn run_list(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    let mut has_run = false;
    for flag in self.list_flags() {
      if matches.get_flag(flag.id()) {
        match flag {
          Flag::All => self.list_all(matches, dsh_api_client).await?,
          Flag::AllocationStatus => self.list_allocation_status(matches, dsh_api_client).await?,
          Flag::Configuration => self.list_configuration(matches, dsh_api_client).await?,
          Flag::Ids => self.list_ids(matches, dsh_api_client).await?,
          Flag::Tasks => self.list_tasks(matches, dsh_api_client).await?,
          Flag::Usage => self.list_usages(matches, dsh_api_client).await?,
          Flag::Value => self.list_values(matches, dsh_api_client).await?,
        }
        has_run = true;
        break;
      }
    }
    if !has_run {
      self.list_default(matches, dsh_api_client).await?
    }
    Ok(())
  }

  // Final
  async fn run_show(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    match matches.get_one::<String>(TARGET_ARGUMENT) {
      Some(target_id) => {
        let mut has_run = false;
        for flag in self.show_flags() {
          if matches.get_flag(flag.id()) {
            match flag {
              Flag::All => self.show_all(target_id, matches, dsh_api_client).await?,
              Flag::AllocationStatus => self.show_allocation_status(target_id, matches, dsh_api_client).await?,
              Flag::Configuration => self.show_configuration(target_id, matches, dsh_api_client).await?,
              Flag::Ids => self.show_ids(target_id, matches, dsh_api_client).await?,
              Flag::Tasks => self.show_tasks(target_id, matches, dsh_api_client).await?,
              Flag::Usage => self.show_usage(target_id, matches, dsh_api_client).await?,
              Flag::Value => self.show_value(target_id, matches, dsh_api_client).await?,
            }
            has_run = true;
          }
        }
        if !has_run {
          self.show_default(target_id, matches, dsh_api_client).await?
        }
        Ok(())
      }
      None => self.to_command_error_missing_id(),
    }
  }

  // Final
  fn flag_arguments(&self, flags: &[Flag]) -> Vec<Arg> {
    flags
      .iter()
      .map(|flag| match flag {
        Flag::All => self.all_flag(),
        Flag::AllocationStatus => self.allocation_status_flag(),
        Flag::Configuration => self.configuration_flag(),
        Flag::Ids => self.ids_flag(),
        Flag::Tasks => self.tasks_flag(),
        Flag::Usage => self.usage_flag(),
        Flag::Value => self.value_flag(),
      })
      .collect()
  }

  // Final
  fn all_flag(&self) -> Arg {
    Arg::new(Flag::All.id())
      .long(Flag::All.option())
      .short('a')
      .action(ArgAction::SetTrue)
      .help(format!("Show actual {}", self.subject()))
      .long_help(format!("Include actual deployed {}s.", self.subject()))
  }

  // Final
  fn allocation_status_flag(&self) -> Arg {
    Arg::new(Flag::AllocationStatus.id())
      .long(Flag::AllocationStatus.option())
      .short('s')
      .action(ArgAction::SetTrue)
      .help(format!("Show {}'s allocation status", self.subject()))
      .long_help(format!("Show {}'s allocation status information.", self.subject()))
  }

  // Final
  fn configuration_flag(&self) -> Arg {
    Arg::new(Flag::Configuration.id())
      .long(Flag::Configuration.option())
      .short('c')
      .action(ArgAction::SetTrue)
      .help(format!("Show {}'s configuration", self.subject()))
      .long_help(format!("Show {}'s configuration parameters.", self.subject()))
  }

  // Final
  fn ids_flag(&self) -> Arg {
    Arg::new(Flag::Ids.id())
      .long(Flag::Ids.option())
      .short('i')
      .action(ArgAction::SetTrue)
      .help(format!("Show {}'s identifiers", self.subject()))
      .long_help(format!("Show {}'s identifiers.", self.subject()))
  }

  // Final
  fn tasks_flag(&self) -> Arg {
    Arg::new(Flag::Tasks.id())
      .long(Flag::Tasks.option())
      .action(ArgAction::SetTrue)
      .help(format!("Show {}'s tasks", self.subject()))
      .long_help(format!("Show {}'s tasks information.", self.subject()))
  }

  // Final
  fn usage_flag(&self) -> Arg {
    Arg::new(Flag::Usage.id())
      .long(Flag::Usage.option())
      .short('u')
      .action(ArgAction::SetTrue)
      .help(format!("Show {}'s usage", self.subject()))
      .long_help(format!("Show where this {} is used.", self.subject()))
  }

  // Final
  fn value_flag(&self) -> Arg {
    Arg::new(Flag::Value.id())
      .long(Flag::Value.option())
      .short('v')
      .action(ArgAction::SetTrue)
      .help(format!("Show {}'s value", self.subject()))
      .long_help(format!("Show the value of this {}.", self.subject()))
  }

  // Final
  fn create_subcommand(&self) -> Command {
    Command::new(CREATE_SUBCOMMAND)
      .about(format!("Create a new {}", self.subject()))
      .after_help(format!("Create a new {}", self.subject()))
      .after_long_help(format!("Create a new {}", self.subject()))
      .arg(self.target_argument())
      .args(self.flag_arguments(&self.create_flags()))
  }

  // Final
  fn delete_subcommand(&self) -> Command {
    Command::new(DELETE_SUBCOMMAND)
      .about(format!("Delete {}", self.subject()))
      .after_help(format!("Delete {}", self.subject()))
      .after_long_help(format!("Delete {}", self.subject()))
      .arg(self.target_argument())
      .args(self.flag_arguments(&self.delete_flags()))
  }

  // Final
  fn list_subcommand(&self) -> Command {
    Command::new(LIST_SUBCOMMAND)
      .about(format!("List {}s", self.subject()))
      .alias("l")
      .args(self.flag_arguments(&self.list_flags()))
      .after_help(format!("List {}s", self.subject()))
      .after_long_help(format!("List all available {}s", self.subject()))
  }

  // Final
  fn show_subcommand(&self) -> Command {
    Command::new(SHOW_SUBCOMMAND)
      .about(format!("Show {} details", self.subject()))
      .alias("s")
      .after_help(format!("Show {} details", self.subject()))
      .after_long_help(format!("Show {} details", self.subject()))
      .arg(self.target_argument())
      .args(self.flag_arguments(&self.show_flags()))
  }

  // Final
  fn update_subcommand(&self) -> Command {
    Command::new(UPDATE_SUBCOMMAND)
      .about(format!("Update an existing {}", self.subject()))
      .after_help(format!("Update an existing {}", self.subject()))
      .after_long_help(format!("Update an existing {}", self.subject()))
      .arg(self.target_argument())
      .args(self.flag_arguments(&self.update_flags()))
  }

  // Final
  fn target_argument(&self) -> Arg {
    Arg::new(TARGET_ARGUMENT)
      .action(ArgAction::Append)
      .value_parser(builder::NonEmptyStringValueParser::new())
      .value_name(self.subject_first_upper())
      .help(format!("{} name", self.subject_first_upper()))
      .long_help(format!("{} name", self.subject_first_upper()))
  }

  // Final
  fn to_command_error(&self, error: DshApiError) -> CommandResult {
    match error {
      DshApiError::NotAuthorized => Err("not authorized".to_string()),
      DshApiError::NotFound => Err(format!("{} not found", &self.subject())),
      DshApiError::Unexpected(error) => Err(format!("unexpected error {}", error)),
    }
  }

  // Final
  fn to_command_error_with_id(&self, error: DshApiError, which: &str) -> CommandResult {
    match error {
      DshApiError::NotAuthorized => Err("not authorized".to_string()),
      DshApiError::NotFound => Err(format!("{} {} not found", &self.subject(), which)),
      DshApiError::Unexpected(error) => Err(format!("unexpected error {}", error)),
    }
  }

  // Final
  fn to_command_error_missing_id(&self) -> CommandResult {
    Err(format!("missing {} id", &self.subject()))
  }
}
