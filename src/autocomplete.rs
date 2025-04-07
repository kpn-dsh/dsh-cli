use crate::APPLICATION_NAME;
use clap::builder::EnumValueParser;
use clap::{Arg, ArgAction, Command};
use clap_complete::{generate, shells};
use std::fmt::{Display, Formatter};
use std::io;

#[derive(clap::ValueEnum, Clone, Debug)]
#[clap(rename_all = "lower")]
pub(crate) enum AutocompleteShell {
  /// Bourne-again shell
  Bash,
  /// Elvish shell
  Elvish,
  /// Fish shell
  Fish,
  /// Microsoft Powershell
  PowerShell,
  /// Z shell
  Zsh,
}

impl Display for AutocompleteShell {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      AutocompleteShell::Bash => write!(f, "bash"),
      AutocompleteShell::Elvish => write!(f, "elvish"),
      AutocompleteShell::Fish => write!(f, "fish"),
      AutocompleteShell::PowerShell => write!(f, "powershell"),
      AutocompleteShell::Zsh => write!(f, "zsh"),
    }
  }
}

pub(crate) const AUTOCOMPLETE_ARGUMENT: &str = "autocomplete-argument";

pub(crate) fn generate_autocomplete_file_argument() -> Arg {
  Arg::new(AUTOCOMPLETE_ARGUMENT)
    .long("generate-autocomplete-file")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<AutocompleteShell>::new())
    .value_name("SHELL")
    .help("Generate autocomplete file")
    .long_help(
      "If this option is provided, \
          the dsh tool will write an autocomplete file for the selected shell to stdout. \
          See the documentation for your shell on how to install the autocomplete file. \
          When this option is used, all other provided commands or options will be ignored.",
    )
    .exclusive(true)
    .hide_short_help(true)
}

pub(crate) fn generate_autocomplete_file(command: &mut Command, shell: &AutocompleteShell) {
  match shell {
    AutocompleteShell::Bash => generate(shells::Bash, command, APPLICATION_NAME, &mut io::stdout()),
    AutocompleteShell::Elvish => generate(shells::Elvish, command, APPLICATION_NAME, &mut io::stdout()),
    AutocompleteShell::Fish => generate(shells::Fish, command, APPLICATION_NAME, &mut io::stdout()),
    AutocompleteShell::PowerShell => generate(shells::PowerShell, command, APPLICATION_NAME, &mut io::stdout()),
    AutocompleteShell::Zsh => generate(shells::Zsh, command, APPLICATION_NAME, &mut io::stdout()),
  }
}
