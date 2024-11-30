use crate::arguments::{Verbosity, HIDE_BORDER_ARGUMENT, SET_VERBOSITY_ARGUMENT, SHOW_EXECUTION_TIME_ARGUMENT};
use crate::formatters::formatter::OutputFormat;
use crate::settings::read_settings;
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;

pub(crate) struct DcliContext<'a> {
  pub(crate) output_format: OutputFormat,
  pub(crate) verbosity: Verbosity,
  pub(crate) hide_border: bool,
  pub(crate) show_execution_time: bool,
  pub(crate) dsh_api_client: Option<DshApiClient<'a>>,
}

impl DcliContext<'_> {
  pub(crate) fn print_capability_explanation<T: AsRef<str>>(&self, explanation: T) {
    match self.verbosity {
      Verbosity::Low => (),
      Verbosity::Medium => eprintln!("{}", explanation.as_ref()),
      Verbosity::High => {
        if let Some(ref client) = self.dsh_api_client {
          eprintln!("target {}", client.tenant());
        }
        eprintln!("{}", explanation.as_ref())
      }
    }
  }

  pub(crate) fn print_execution_time(&self, millis: u128) {
    if self.show_execution_time || self.verbosity == Verbosity::High {
      eprintln!("execution took {} milliseconds", millis);
    }
  }

  pub(crate) fn _show_settings(&self) -> bool {
    match self.verbosity {
      Verbosity::Low => false,
      Verbosity::Medium | Verbosity::High => true,
    }
  }
}

pub(crate) fn get_dcli_context<'a>(matches: &'a ArgMatches, dsh_api_client: Option<DshApiClient<'a>>) -> Result<DcliContext<'a>, String> {
  if let Some(settings) = read_settings(None)? {
    let hide_border = if matches.get_flag(HIDE_BORDER_ARGUMENT) { true } else { settings.hide_border.unwrap_or(false) };
    let verbosity: Verbosity = match matches.get_one::<Verbosity>(SET_VERBOSITY_ARGUMENT) {
      Some(verbosity_argument) => verbosity_argument.to_owned(),
      None => settings.verbosity.unwrap_or(Verbosity::Low).to_owned(),
    };
    let show_execution_time = if matches.get_flag(SHOW_EXECUTION_TIME_ARGUMENT) { true } else { settings.show_execution_time.unwrap_or(false) };
    Ok(DcliContext { output_format: OutputFormat::Table, verbosity, hide_border, show_execution_time, dsh_api_client })
  } else {
    let hide_border = matches.get_flag(HIDE_BORDER_ARGUMENT);
    let verbosity: Verbosity = matches.get_one::<Verbosity>(SET_VERBOSITY_ARGUMENT).unwrap_or(&Verbosity::Low).to_owned();
    let show_execution_time = matches.get_flag(SHOW_EXECUTION_TIME_ARGUMENT);
    Ok(DcliContext { output_format: OutputFormat::Table, verbosity, hide_border, show_execution_time, dsh_api_client })
  }
}
