use crate::capability::{Capability, CapabilityType, CommandExecutor};
use crate::filter_flags::{create_filter_flag, FilterFlagType};
use crate::flags::{create_flag, FlagType};
use crate::modifier_flags::{create_modifier_flag, ModifierFlagType};
use crate::subject::Subject;
use crate::{DcliContext, DcliResult};
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};

pub struct CapabilityBuilder<'a> {
  capability_type: CapabilityType,
  about: String,
  long_about: Option<String>,
  executors: Vec<(FlagType, &'a (dyn CommandExecutor + Send + Sync), Option<String>)>,
  default_executor: Option<&'a (dyn CommandExecutor + Send + Sync)>,
  run_all_executors: bool,
  target_arguments: Vec<Arg>,
  extra_arguments: Vec<Arg>,
  filter_flags: Vec<(FilterFlagType, Option<String>)>,
  modifier_flags: Vec<(ModifierFlagType, Option<String>)>,
}

impl<'a> CapabilityBuilder<'a> {
  pub fn new(capability_type: CapabilityType, about: impl Into<String>) -> Self {
    Self {
      capability_type,
      about: about.into(),
      long_about: None,
      executors: vec![],
      default_executor: None,
      run_all_executors: false,
      target_arguments: vec![],
      extra_arguments: vec![],
      filter_flags: vec![],
      modifier_flags: vec![],
    }
  }

  pub fn set_long_about(mut self, long_about: impl Into<String>) -> Self {
    self.long_about = Some(long_about.into());
    self
  }

  pub fn add_command_executor(mut self, flag_type: FlagType, executor: &'a (dyn CommandExecutor + Send + Sync), long_help: Option<String>) -> Self {
    self.executors.push((flag_type, executor, long_help.map(|long_help| long_help.into())));
    self
  }

  pub fn add_command_executors(mut self, executors: Vec<(FlagType, &'a (dyn CommandExecutor + Send + Sync), Option<String>)>) -> Self {
    for (flag_type, executor, long_help) in executors {
      self.executors.push((flag_type, executor, long_help.map(|s| s.into())))
    }
    self
  }

  pub fn set_default_command_executor(mut self, executor: &'a (dyn CommandExecutor + Send + Sync)) -> Self {
    self.default_executor = Some(executor);
    self
  }

  pub fn set_run_all_executors(mut self, value: bool) -> Self {
    self.run_all_executors = value;
    self
  }

  pub fn add_target_argument(mut self, argument: Arg) -> Self {
    self.target_arguments.push(argument);
    self
  }

  pub fn add_target_arguments(mut self, mut arguments: Vec<Arg>) -> Self {
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
    self.filter_flags.push((flag_type, long_help.map(|long_help| long_help.into())));
    self
  }

  pub fn add_filter_flags(mut self, flags: Vec<(FilterFlagType, Option<String>)>) -> Self {
    for (flag_type, long_help) in flags {
      self.filter_flags.push((flag_type, long_help.map(|s| s.into())))
    }
    self
  }

  pub fn add_modifier_flag(mut self, flag_type: ModifierFlagType, long_help: Option<String>) -> Self {
    self.modifier_flags.push((flag_type, long_help));
    self
  }

  pub fn add_modifier_flags(mut self, flags: Vec<(ModifierFlagType, Option<String>)>) -> Self {
    for (flag_type, long_help) in flags {
      self.modifier_flags.push((flag_type, long_help.map(|s| s.into())))
    }
    self
  }
}

#[async_trait]
impl<'a> Capability for CapabilityBuilder<'a> {
  fn clap_capability_command(&self, subject: &dyn Subject) -> Command {
    let mut capability_command = Command::new(self.capability_type.to_string())
      .name(self.capability_type.to_string())
      .about(&self.about)
      .args(&self.target_arguments)
      .args(self.clap_flags(subject.subject()))
      .args(&self.extra_arguments);
    if let Some(alias) = self.capability_type.command_alias() {
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
        .collect::<Vec<Arg>>(),
      self
        .filter_flags
        .iter()
        .map(|(flag_type, long_help)| create_filter_flag(flag_type, subject, long_help.as_deref()))
        .collect::<Vec<_>>(),
      self
        .modifier_flags
        .iter()
        .map(|(flag_type, long_help)| create_modifier_flag(flag_type, subject, long_help.as_deref()))
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

  async fn execute_capability(&self, argument: Option<String>, sub_argument: Option<String>, matches: &ArgMatches, context: &DcliContext) -> DcliResult {
    let mut last_dcli_result: Option<DcliResult> = None;
    let mut number_of_executed_capabilities = 0;
    if self.run_all_executors {
      for (flag_type, executor, _) in &self.executors {
        if matches.get_flag(flag_type.id()) {
          last_dcli_result = Some(executor.execute(argument.clone(), sub_argument.clone(), matches, context).await);
          number_of_executed_capabilities += 1;
        }
      }
    } else {
      for (flag_type, executor, _) in &self.executors {
        if matches.get_flag(flag_type.id()) && last_dcli_result.is_none() {
          last_dcli_result = Some(executor.execute(argument.clone(), sub_argument.clone(), matches, context).await);
          number_of_executed_capabilities += 1;
        }
      }
    }
    match last_dcli_result {
      Some(dcli_result) => {
        if number_of_executed_capabilities > 1 {
          Ok(true)
        } else {
          dcli_result
        }
      }
      None => {
        if let Some(default_executor) = self.default_executor {
          default_executor.execute(argument.clone(), sub_argument.clone(), matches, context).await
        } else {
          Ok(true)
        }
      }
    }
  }
}
