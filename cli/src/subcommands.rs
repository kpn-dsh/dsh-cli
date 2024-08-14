use clap::{builder, Arg, ArgAction, Command};

pub(crate) const CREATE_SUBCOMMAND: &str = "create";
pub(crate) const DELETE_SUBCOMMAND: &str = "delete";
pub(crate) const LIST_SUBCOMMAND: &str = "list";
pub(crate) const SHOW_SUBCOMMAND: &str = "show";
pub(crate) const STATUS_SUBCOMMAND: &str = "status";
pub(crate) const USAGE_SUBCOMMAND: &str = "usage";

pub(crate) const TARGET_ARGUMENT: &str = "target-argument";

pub(crate) struct CommandDescriptor {
  what: &'static str,
  upper_what: &'static str,
}

impl CommandDescriptor {
  pub fn new(what: &'static str, upper_what: &'static str) -> Self {
    Self { what, upper_what }
  }
}

pub(crate) fn create_subcommand(command_descriptor: &CommandDescriptor, arguments: Vec<Arg>) -> Command {
  Command::new(CREATE_SUBCOMMAND)
    .about(format!("Create {}", command_descriptor.what))
    .after_help(format!("Create {}", command_descriptor.what))
    .after_long_help(format!("Create {}", command_descriptor.what))
    .args(arguments)
    .args(vec![target_argument(command_descriptor)])
}

pub(crate) fn delete_subcommand(command_descriptor: &CommandDescriptor, arguments: Vec<Arg>) -> Command {
  Command::new(DELETE_SUBCOMMAND)
    .about(format!("Delete {}", command_descriptor.what))
    .after_help(format!("Delete {}", command_descriptor.what))
    .after_long_help(format!("Delete {}", command_descriptor.what))
    .args(arguments)
    .args(vec![target_argument(command_descriptor)])
}

pub(crate) fn list_subcommand(command_descriptor: &CommandDescriptor, arguments: Vec<Arg>) -> Command {
  Command::new(LIST_SUBCOMMAND)
    .about(format!("List {}s", command_descriptor.what))
    .args(arguments)
    .after_help(format!("List {}s", command_descriptor.what))
    .after_long_help(format!("List all available {}s", command_descriptor.what))
}

pub(crate) fn show_subcommand(command_descriptor: &CommandDescriptor, arguments: Vec<Arg>) -> Command {
  Command::new(SHOW_SUBCOMMAND)
    .about(format!("Show {} details", command_descriptor.what))
    .after_help(format!("Show {} details", command_descriptor.what))
    .after_long_help(format!("Show {} details", command_descriptor.what))
    .args(arguments)
    .args(vec![target_argument(command_descriptor)])
}

pub(crate) fn status_subcommand(command_descriptor: &CommandDescriptor, _arguments: Vec<Arg>) -> Command {
  Command::new(STATUS_SUBCOMMAND)
    .about(format!("Show {} status", command_descriptor.what))
    .after_help(format!("Show {} status", command_descriptor.what))
    .after_long_help(format!("Show {} status", command_descriptor.what))
    .args(vec![target_argument(command_descriptor)])
}

pub(crate) fn usage_subcommand(command_descriptor: &CommandDescriptor, _arguments: Vec<Arg>) -> Command {
  Command::new(USAGE_SUBCOMMAND)
    .about(format!("Show {} usage", command_descriptor.what))
    .after_help(format!("Show {} usage", command_descriptor.what))
    .after_long_help(format!("Show {} usage", command_descriptor.what))
    .args(vec![target_argument(command_descriptor)])
}

pub(crate) fn target_argument(command_descriptor: &CommandDescriptor) -> Arg {
  let first_upper_what = some_kind_of_uppercase_first_letter(command_descriptor.what);
  Arg::new(TARGET_ARGUMENT)
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name(command_descriptor.upper_what)
    .help(format!("{} name", first_upper_what))
    .long_help(format!("{} name", first_upper_what))
}

fn some_kind_of_uppercase_first_letter(s: &str) -> String {
  let mut c = s.chars();
  match c.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
  }
}
