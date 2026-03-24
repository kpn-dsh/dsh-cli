use crate::authentication::AuthenticationMethod;
use crate::context::BrowserMethod;
use crate::formatters::OutputFormat;
use crate::verbosity::Verbosity;
use crate::{OUTPUT_OPTIONS_HEADING, TOOL_OPTIONS_HEADING};
use builder::EnumValueParser;
use clap::builder::{PossibleValue, ValueParser};
use clap::{builder, Arg, ArgAction};
use dsh_api::platform::DshPlatform;
use itertools::Itertools;

pub(crate) const AUTHENTICATION_OPTION: &str = "authentication-option";
pub(crate) const BROWSER_OPTION: &str = "browser-option";
pub(crate) const DRY_RUN_FLAG: &str = "dry-run-flag";
pub(crate) const ENVIRONMENT_VARIABLE_OPTION: &str = "environment-variable-option";
pub(crate) const FORCE_FLAG: &str = "force-flag";
// pub(crate) const FROM_CLIPBOARD_FLAG: &str = "from-clipboard-flag";
pub(crate) const NO_CSV_HEADERS_FLAG: &str = "no-csv-headers-flag";
pub(crate) const NO_ESCAPE_FLAG: &str = "no-escape-flag";
pub(crate) const OUTPUT_FORMAT_OPTION: &str = "output-format-option";
pub(crate) const QUIET_FLAG: &str = "quiet-flag";
pub(crate) const ROBOT_PASSWORD_FILE_OPTION: &str = "robot-password-file-option";
pub(crate) const ROBOT_PLATFORM_OPTION: &str = "robot-platform-option";
pub(crate) const ROBOT_TENANT_OPTION: &str = "robot-tenant-option";
pub(crate) const SHOW_EXECUTION_TIME_FLAG: &str = "show-execution-time-flag";
pub(crate) const SUPPRESS_EXIT_STATUS_FLAG: &str = "suppress-exit-status-flag";
pub(crate) const TERMINAL_WIDTH_OPTION: &str = "terminal-width-option";
// pub(crate) const TO_CLIPBOARD_FLAG: &str = "to-clipboard-flag";
pub(crate) const VERBOSITY_OPTION: &str = "set-verbosity-option";
pub(crate) const VERSION_FLAG: &str = "version-flag";

pub(crate) fn authentication_option() -> Arg {
  Arg::new(AUTHENTICATION_OPTION)
    .long("authentication")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<AuthenticationMethod>::new())
    .value_name("METHOD")
    .help("Set authentication method")
    .long_help(
      "This option specifies the authentication method that will be used \
          to access the resource management api. If this argument is not provided, the value \
          from the environment variable 'DSH_CLI_AUTHENTICATION' or the value from the \
          settings file will be used. By default, when stdout is a terminal 'single-sign-on' \
          (single sign on) will be used, while if stdout is not a terminal 'robot' will be used.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

pub(crate) fn browser_option() -> Arg {
  Arg::new(BROWSER_OPTION)
    .long("browser")
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<BrowserMethod>::new())
    .value_name("METHOD")
    .help("Specifies whether the tool may try to open a browser")
    .long_help(
      "This option specifies whether the cli tool will try to automatically open a browser \
     (e.g. for authentication or to open the console) or will only instruct the user to open it. \
     If this variable is not provided, the value from the environment variable 'DSH_CLI_BROWSER' \
     or the value from the settings file will be used, if it exists. Else, the default value \
     will be 'open' when the cli tool is run interactive ('stdin' is a terminal) and \
     'instruct' if not.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

pub(crate) fn dry_run_flag() -> Arg {
  Arg::new(DRY_RUN_FLAG)
    .long("dry-run")
    .action(ArgAction::SetTrue)
    .help("Execute in dry-run mode")
    .long_help(
      "When this flag is provided the dsh tool will run in dry-run mode, \
          meaning that no changes will be made to the \
          resources and services on the DSH. Dry-run mode can also be set by the \
          environment variable DSH_CLI_DRY_RUN or in the settings file.",
    )
    .global(true)
}

pub(crate) fn environment_variable_option() -> Arg {
  Arg::new(ENVIRONMENT_VARIABLE_OPTION)
    .long("environment-variable")
    .short('e')
    .action(ArgAction::Append)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("ENV_VAR=value")
    .help("Set environment variable")
    .long_help(
      "This option allows setting environment variables from the command line. \
          The environment variable must be specified as \"VAR=value\". \
          This option can be provided multiple times.",
    )
    .hide_short_help(true)
    .global(true)
}

pub(crate) fn force_flag() -> Arg {
  Arg::new(FORCE_FLAG)
    .long("force")
    .action(ArgAction::SetTrue)
    .help("Force changes without confirmation")
    .long_help(
      "When this flag is provided all change, update and delete actions \
          will be executed without asking for confirmation. \
          Note that dry-run mode will take precedence over the --force flag and force-mode \
          cannot be enabled via an environment variable of in settings file.",
    )
    .global(true)
}

// pub(crate) fn from_clipboard_flag() -> Arg {
//   Arg::new(FROM_CLIPBOARD_FLAG)
//     .long("from-clipboard")
//     .action(ArgAction::SetTrue)
//     .help("Read input from clipboard")
//     .long_help(
//       "When this flag is provided the input for methods that require it \
//           will be read from the clipboard, \
//           instead of being read from the terminal, pipes or redirects.",
//     )
//     .global(true)
//     .help_heading(MAIN_OPTIONS_HEADING)
// }

pub(crate) fn no_escape_flag() -> Arg {
  Arg::new(NO_ESCAPE_FLAG)
    .long("no-color")
    .alias("no-ansi")
    .action(ArgAction::SetTrue)
    .long_help(
      "When this flag is provided the output will not contain \
          any color or other ansi escape sequences. \
          If this argument is not provided, the environment variable \
          DSH_CLI_NO_ESCAPE or the value from the settings file will be used. \
          The default behavior is to use ansi escape styling where applicable.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn no_csv_headers_flag() -> Arg {
  Arg::new(NO_CSV_HEADERS_FLAG)
    .long("no-csv-headers")
    .action(ArgAction::SetTrue)
    .long_help(
      "When this flag is provided csv output will not contain headers. \
          If this argument is not provided, the environment variable \
          DSH_CLI_NO_CSV_HEADERS or the value from the settings file will be used. \
          The default behavior is to use headers where applicable.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn output_format_option() -> Arg {
  Arg::new(OUTPUT_FORMAT_OPTION)
    .long("output-format")
    .short('o')
    .action(ArgAction::Set)
    .value_parser(EnumValueParser::<OutputFormat>::new())
    .value_name("FORMAT")
    .help("Set output format")
    .long_help(
      "This option specifies the format used when printing the output. \
          If this argument is not provided, the value from the environment variable \
          DSH_CLI_OUTPUT_FORMAT or the value from the settings file will be used. \
          By default, when stdout is a terminal 'table' will be used, \
          while if stdout is not a terminal 'json' will be used.",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn robot_password_file_option() -> Arg {
  Arg::new(ROBOT_PASSWORD_FILE_OPTION)
    .long("robot-password-file")
    .action(ArgAction::Set)
    .value_parser(ValueParser::path_buf())
    .value_name("FILE")
    .help("Provide robot password file name")
    .long_help(
      "This option specifies the name of a file that contains the robot password, which can \
          be used when authenticating via the robot access pattern. \
          If this flag is not provided, the environment variable DSH_CLI_ROBOT_PASSWORD_FILE \
          will be tried. Else, the user will be prompted for the password.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

pub(crate) fn robot_platform_option() -> Arg {
  let possible_values = DshPlatform::all()
    .iter()
    .map(|platform| {
      PossibleValue::new(platform.name())
        .alias(platform.alias())
        .help(format!("{} ({})", platform.description(), platform.alias()))
    })
    .collect_vec();
  Arg::new(ROBOT_PLATFORM_OPTION)
    .long("robot-platform")
    .action(ArgAction::Set)
    .value_parser(possible_values)
    .value_name("PLATFORM")
    .help("Provide robot platform")
    .long_help(
      "This option specifies the name of the robot platform which is required when \
          authenticating via the robot access pattern. If this argument is not provided, \
          the robot platform can also be specified via the environment variable \
          DSH_CLI_ROBOT_PLATFORM, or else the user will be prompted.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

pub(crate) fn robot_tenant_option() -> Arg {
  Arg::new(ROBOT_TENANT_OPTION)
    .long("robot-tenant")
    .action(ArgAction::Set)
    .value_parser(builder::NonEmptyStringValueParser::new())
    .value_name("TENANT")
    .help("Provide robot tenant")
    .long_help(
      "This option specifies the name of the robot tenant which is required when \
          authenticating via the robot access pattern. If this argument is not provided, \
          the tenant should be specified via the environment variable DSH_CLI_ROBOT_TENANT, \
          or else the user will be prompted.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

pub(crate) fn quiet_flag() -> Arg {
  Arg::new(QUIET_FLAG)
    .long("quiet")
    .short('q')
    .action(ArgAction::SetTrue)
    .help("Run in quiet mode")
    .long_help(
      "When this flag is provided the dsh tool will run in quiet mode, \
          meaning that no output will be produced to the terminal (stdout and stderr).",
    )
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn set_verbosity_option() -> Arg {
  Arg::new(VERBOSITY_OPTION)
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

pub(crate) fn show_execution_time_flag() -> Arg {
  Arg::new(SHOW_EXECUTION_TIME_FLAG)
    .long("show-execution-time")
    .action(ArgAction::SetTrue)
    .long_help(
      "When this flag is provided the execution time of the executed function \
          will be shown, in milliseconds.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

pub(crate) fn suppress_exit_status_flag() -> Arg {
  Arg::new(SUPPRESS_EXIT_STATUS_FLAG)
    .long("suppress-exit-status")
    .action(ArgAction::SetTrue)
    .long_help(
      "When this flag is provided the dsh tool will always return exit status 0, \
            even when an error has occurred. This can be useful in scripting environments.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}

pub(crate) fn terminal_width_option() -> Arg {
  Arg::new(TERMINAL_WIDTH_OPTION)
    .long("terminal-width")
    .action(ArgAction::Set)
    .value_parser(builder::RangedU64ValueParser::<usize>::from(40..))
    .value_name("WIDTH")
    .long_help(
      "With this option the maximum terminal width can be set. \
          If not set, the environment variable DSH_CLI_TERMINAL_WIDTH will be used \
          or else no terminal width value will be used.",
    )
    .hide_short_help(true)
    .global(true)
    .help_heading(OUTPUT_OPTIONS_HEADING)
}

// pub(crate) fn to_clipboard_flag() -> Arg {
//   Arg::new(TO_CLIPBOARD_FLAG)
//     .long("to-clipboard")
//     .action(ArgAction::SetTrue)
//     .help("Copy output to clipboard")
//     .long_help(
//       "When this flag is provided the output will be copied to the clipboard, \
//           instead of being printed to the terminal.",
//     )
//     .global(true)
//     .help_heading(MAIN_OPTIONS_HEADING)
// }

pub(crate) fn version_flag() -> Arg {
  Arg::new(VERSION_FLAG)
    .long("version")
    .action(ArgAction::SetTrue)
    .long_help(
      "If this flag is provided, the dsh tool will show its version number \
          and the versions of some of its dependencies.",
    )
    .exclusive(true)
    .hide_short_help(true)
    .help_heading(TOOL_OPTIONS_HEADING)
}
