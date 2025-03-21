use crate::context::{MatchingColor, MatchingStyle};
use crate::formatters::OutputFormat;
use crate::verbosity::Verbosity;
use builder::EnumValueParser;
use clap::builder::{PossibleValue, ValueParser};
use clap::{builder, Arg, ArgAction};
use dsh_api::platform::DshPlatform;

pub(crate) const DRY_RUN_ARGUMENT: &str = "dry-run-argument";
pub(crate) const FORCE_ARGUMENT: &str = "force-argument";
// pub(crate) const FROM_CLIPBOARD_ARGUMENT: &str = "from-clipboard-argument";
pub(crate) const MATCHING_COLOR_ARGUMENT: &str = "matching-color-argument";
pub(crate) const MATCHING_STYLE_ARGUMENT: &str = "matching-style-argument";
pub(crate) const NO_ESCAPE_ARGUMENT: &str = "no-escape-argument";
pub(crate) const NO_HEADERS_ARGUMENT: &str = "no-headers-argument";
pub(crate) const OUTPUT_FORMAT_ARGUMENT: &str = "output-format-argument";
pub(crate) const QUIET_ARGUMENT: &str = "quiet-argument";
pub(crate) const SHOW_EXECUTION_TIME_ARGUMENT: &str = "show-execution-time-argument";
pub(crate) const TARGET_PASSWORD_FILE_ARGUMENT: &str = "target-password-file-argument";
pub(crate) const TARGET_PLATFORM_ARGUMENT: &str = "target-platform-argument";
pub(crate) const TARGET_TENANT_ARGUMENT: &str = "target-tenant-argument";
pub(crate) const TERMINAL_WIDTH_ARGUMENT: &str = "terminal-width-argument";
// pub(crate) const TO_CLIPBOARD_ARGUMENT: &str = "to-clipboard-argument";
pub(crate) const VERBOSITY_ARGUMENT: &str = "set-verbosity-argument";

pub(crate) const OUTPUT_OPTIONS_HEADING: &str = "Output options";

pub(crate) fn dry_run_argument() -> Arg {
  Arg::new(DRY_RUN_ARGUMENT)
    .long("dry-run")
    .action(ArgAction::SetTrue)
    .help("Execute in dry-run mode")
    .long_help(
      "When this option is provided the dsh tool will run in dry-run mode, \
          meaning that no changes will be made to the \
          resources and services on the DSH. Dry-run mode can also be set by the \
          environment variable DSH_CLI_DRY_RUN or in the settings file. \
          Dry-run mode will take precedence over the --force flag.",
    )
    .global(true)
}

pub(crate) fn force_argument() -> Arg {
  Arg::new(FORCE_ARGUMENT)
    .long("force")
    .action(ArgAction::SetTrue)
    .help("Force changes without confirmation")
    .long_help(
      "When this option is provided all change, update and delete actions \
          will be executed without asking for confirmation. \
          Note that dry-run mode will take precedence over the --force flag.",
    )
    .global(true)
}

// pub(crate) fn from_clipboard_argument() -> Arg {
//   Arg::new(FROM_CLIPBOARD_ARGUMENT)
//     .long("from-clipboard")
//     .action(ArgAction::SetTrue)
//     .help("Read input from clipboard")
//     .long_help(
//       "When this option is provided the input for methods that require it \
//           will be read from the clipboard, \
//           instead of being read from the terminal, pipes or redirects.",
//     )
//     .global(true)
// }

pub(crate) fn matching_color_argument() -> Arg {
  Arg::new(MATCHING_COLOR_ARGUMENT)
    .long("matching-color")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<MatchingColor>::new())
    .value_name("COLOR")
    .help("Set color for matches")
    .long_help(
      "This option specifies the color to be used when printing matching results \
          for the find functions, e.q. when matching regular expressions. \
          If this argument is not provided, the value from environment variable \
          DSH_CLI_MATCHING_COLOR or the value from the settings file will be used. \
          The default style is 'black'.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn matching_style_argument() -> Arg {
  Arg::new(MATCHING_STYLE_ARGUMENT)
    .long("matching-style")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<MatchingStyle>::new())
    .value_name("STYLE")
    .help("Set styling for matches")
    .long_help(
      "This option specifies the styling to be used when printing matching results \
          for the find functions, e.q. when matching regular expressions. \
          If this argument is not provided, the value from environment variable \
          DSH_CLI_MATCHING_STYLE or the value from the settings file will be used. \
          The default style is 'bold'.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn no_escape_argument() -> Arg {
  Arg::new(NO_ESCAPE_ARGUMENT)
    .long("no-color")
    .alias("no-ansi")
    .action(ArgAction::SetTrue)
    .help("No color")
    .long_help(
      "When this option is provided the output will not contain \
          any color or other ansi escape sequences. \
          If this argument is not provided, the environment variable \
          DSH_CLI_NO_ESCAPE or the value from the settings file will be used. \
          The default behavior is to use ansi escape styling where applicable.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn no_headers_argument() -> Arg {
  Arg::new(NO_HEADERS_ARGUMENT)
    .long("no-headers")
    .action(ArgAction::SetTrue)
    .help("No headers")
    .long_help(
      "When this option is provided the output will not contain headers. \
          If this argument is not provided, the environment variable \
          DSH_CLI_NO_HEADERS or the value from the settings file will be used. \
          The default behavior is to use headers where applicable.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn output_format_argument() -> Arg {
  Arg::new(OUTPUT_FORMAT_ARGUMENT)
    .long("output-format")
    .short('o')
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<OutputFormat>::new())
    .value_name("FORMAT")
    .help("Set output format")
    .long_help(
      "This option specifies the format used when printing the output. \
          If this argument is not provided, the value from the environment variable \
          DSH_CLI_OUTPUT_FORMAT of the value from the settings file will be used. \
          By default, when stdout is a terminal 'table' will be used, \
          while if stdout is not a terminal 'json' will be used.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn quiet_argument() -> Arg {
  Arg::new(QUIET_ARGUMENT)
    .long("quiet")
    .short('q')
    .action(ArgAction::SetTrue)
    .help("Run in quiet mode")
    .long_help(
      "When this option is provided the dsh tool will run in quiet mode, \
          meaning that no output will be produced to the terminal (stdout and stderr).",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn set_verbosity_argument() -> Arg {
  Arg::new(VERBOSITY_ARGUMENT)
    .long("verbosity")
    .short('v')
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<Verbosity>::new())
    .value_name("VERBOSITY")
    .help("Set verbosity level")
    .long_help(
      "If this option is provided, \
    it will set the verbosity level. \
    The default verbosity setting is 'low'.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn show_execution_time_argument() -> Arg {
  Arg::new(SHOW_EXECUTION_TIME_ARGUMENT)
    .long("show-execution-time")
    .action(ArgAction::SetTrue)
    .help("Show execution time")
    .long_help(
      "When this option is provided the execution time of the executed function \
          will be shown, in milliseconds.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn target_password_file_argument() -> Arg {
  Arg::new(TARGET_PASSWORD_FILE_ARGUMENT)
    .long("password-file")
    .action(ArgAction::Set)
    .value_parser(ValueParser::path_buf())
    .value_name("FILE")
    .help("Provide target password file name")
    .long_help(
      "This option specifies the name of a file that contains the target password. \
          If this flag is not provided, the environment variable \
          DSH_CLI_PASSWORD_FILE will be tried. Else, if the platform and tenant are known, \
          the target settings file will be checked. \
          Finally, the user will be prompted for the password.",
    )
    .global(true)
}

pub(crate) fn target_platform_argument() -> Arg {
  let possible_values = DshPlatform::all()
    .iter()
    .map(|platform| {
      PossibleValue::new(platform.name())
        .alias(platform.alias())
        .help(format!("{} ({})", platform.description(), platform.alias()))
    })
    .collect::<Vec<_>>();
  Arg::new(TARGET_PLATFORM_ARGUMENT)
    .long("platform")
    .short('p')
    .action(ArgAction::Append)
    .value_parser(possible_values)
    .num_args(1..)
    .value_delimiter(',')
    .value_name("PLATFORM")
    .help("Provide target platform")
    .long_help(
      "This option specifies the name of the target platform. \
          If this argument is not provided, \
          the platform must be specified via the environment variable DSH_CLI_PLATFORM, \
          as a default setting in the settings file, or else the user will be prompted. \
          The value between parentheses can be used as an alias for the platform name. \
          Some functions allow multiple target platforms to be specified at once. \
          In this case the platform names must be separated by ',', without any spaces",
    )
    .global(true)
}

pub(crate) fn target_tenant_argument() -> Arg {
  Arg::new(TARGET_TENANT_ARGUMENT)
    .long("tenant")
    .short('t')
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .num_args(1..)
    .value_delimiter(',')
    .value_name("TENANT")
    .help("Provide target tenant")
    .long_help(
      "This option specifies the name of the target tenant. \
          If this argument is not provided, \
          the tenant should be specified via the environment variable DSH_CLI_TENANT, \
          as a default setting in the settings file, or else the user will be prompted. \
          Some functions allow multiple target tenants to be specified at once. \
          In this case the platform names must be separated by ',', without any spaces",
    )
    .global(true)
}

pub(crate) fn terminal_width_argument() -> Arg {
  Arg::new(TERMINAL_WIDTH_ARGUMENT)
    .long("terminal-width")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<usize>::from(40..))
    .value_name("WIDTH")
    .help("Set terminal width")
    .long_help(
      "With this option the maximum terminal width can be set. \
          If not set, the environment variable DSH_CLI_TERMINAL_WIDTH will be used \
          or else no terminal width value will be used.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

// pub(crate) fn to_clipboard_argument() -> Arg {
//   Arg::new(TO_CLIPBOARD_ARGUMENT)
//     .long("to-clipboard")
//     .action(ArgAction::SetTrue)
//     .help("Copy output to clipboard")
//     .long_help(
//       "When this option is provided the output will be copied to the clipboard, \
//           instead of being printed to the terminal.",
//     )
//     .global(true)
// }
