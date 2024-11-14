use crate::APPLICATION_NAME;
use clap::builder::EnumValueParser;
use clap::{Arg, ArgAction, Command};
use clap_complete::{generate, shells};
use std::io;

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum AutocompleteShell {
  Bash,
  Elvish,
  Fish,
  PowerShell,
  Zsh,
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
          the application will write an autocomplete file for the selected shell to stdout. \
          See the documentation for your shell on how to install the autocomplete file. \
          The supported shells are 'bash', 'elvish', 'fish', 'powershell' and 'zsh'. \
          Any other provided commands or options will be ignored.",
    )
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
