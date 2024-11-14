use clap::builder::EnumValueParser;
use clap::{Arg, ArgAction, Command};
use clap_complete::{generate, shells, Generator};
use std::io;
use crate::APPLICATION_NAME;

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
    .help("Generate autocompletion file for the provided SHELL")
    .long_help(
      "If this option is provided, \
          the application will write an autocomplete file for the selected shell to stdout. \
          See the documentation for your shell on how to install the autocomplete file. \
          The supported shells are 'bash', 'elvish', 'fish', `powershell' and 'zsh'. \
          Any other provided commands or options will be ignored.",
    )
}

pub(crate) fn generate_autocomplete_file(command: &mut Command, shell: &AutocompleteShell) {
  let generator: impl Generator = match shell {
    AutocompleteShell::Bash => shells::Bash,
    AutocompleteShell::Elvish => shells::Elvish,
    AutocompleteShell::Fish => shells::Fish,
    AutocompleteShell::PowerShell => shells::PowerShell,
    AutocompleteShell::Zsh => shells::Zsh
  };
  generate(generator, command, APPLICATION_NAME, &mut io::stdout())
}
