// Template for new commands
use async_trait::async_trait;
use clap::ArgMatches;

use trifonius_dsh_api::DshApiClient;

use crate::arguments::Flag;
use crate::command::SubjectCommand;
use crate::CommandResult;

pub(crate) struct DefImplCommand {}

impl Default for DefImplCommand {
  fn default() -> Self {
    DefImplCommand {}
  }
}

#[allow(unused)]
#[async_trait]
impl SubjectCommand for DefImplCommand {
  fn subject(&self) -> &'static str {
    todo!()
  }

  fn subject_first_upper(&self) -> &'static str {
    todo!()
  }

  fn about(&self) -> String {
    todo!()
  }

  fn long_about(&self) -> String {
    todo!()
  }

  fn alias(&self) -> Option<&str> {
    todo!()
  }

  fn list_flags(&self) -> &'static [Flag] {
    todo!()
  }

  fn show_flags(&self) -> &'static [Flag] {
    todo!()
  }

  async fn list_all(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_allocation_status(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_configuration(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_default(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_ids(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_tasks(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_usages(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn list_values(&self, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_all(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_allocation_status(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_configuration(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_default(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_tasks(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_usage(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }

  async fn show_value(&self, target_id: &str, matches: &ArgMatches, dsh_api_client: &DshApiClient<'_>) -> CommandResult {
    todo!()
  }
}
