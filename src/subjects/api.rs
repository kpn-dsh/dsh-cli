use crate::capability::{Capability, CommandExecutor, SHOW_COMMAND, SHOW_COMMAND_PAIR};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::subject::Subject;
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::generic::{MethodDescriptor, DELETE_METHODS, GET_METHODS, HEAD_METHODS, PATCH_METHODS, POST_METHODS, PUT_METHODS};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::time::Instant;

pub(crate) struct ApiSubject {}

const API_SUBJECT_TARGET: &str = "api";

lazy_static! {
  pub static ref API_SUBJECT: Box<dyn Subject + Send + Sync> = Box::new(ApiSubject {});
}

#[async_trait]
impl Subject for ApiSubject {
  fn subject(&self) -> &'static str {
    API_SUBJECT_TARGET
  }

  /// Help text printed for -h flag
  fn subject_command_about(&self) -> String {
    "List and call DSH resource management api.".to_string()
  }

  /// Help text printed for --help flag
  fn subject_command_long_about(&self) -> String {
    "List and call DSH resource management api.".to_string()
  }

  fn requires_dsh_api_client(&self, sub_matches: &ArgMatches) -> bool {
    match sub_matches.subcommand() {
      Some((capability_command_id, _)) => !matches!(capability_command_id, SHOW_COMMAND),
      None => unreachable!(),
    }
  }

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DELETE_COMMAND => Some(API_DELETE_CAPABILITY.as_ref()),
      GET_COMMAND => Some(API_GET_CAPABILITY.as_ref()),
      HEAD_COMMAND => Some(API_HEAD_CAPABILITY.as_ref()),
      PATCH_COMMAND => Some(API_PATCH_CAPABILITY.as_ref()),
      POST_COMMAND => Some(API_POST_CAPABILITY.as_ref()),
      PUT_COMMAND => Some(API_PUT_CAPABILITY.as_ref()),
      SHOW_COMMAND => Some(API_SHOW_CAPABILITY.as_ref()),
      _ => None,
    }
  }

  fn capabilities(&self) -> &Vec<&(dyn Capability + Send + Sync)> {
    &API_CAPABILITIES
  }
}

lazy_static! {
  static ref API_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(DELETE_COMMAND, &ApiDelete {});
  static ref API_GET_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(GET_COMMAND, &ApiGet {});
  static ref API_HEAD_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(HEAD_COMMAND, &ApiGet {});
  static ref API_PATCH_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(PATCH_COMMAND, &ApiPatch {});
  static ref API_POST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(POST_COMMAND, &ApiPost {});
  static ref API_PUT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(PUT_COMMAND, &ApiPut {});
  static ref API_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(SHOW_COMMAND_PAIR, "Print the open api specification.").set_default_command_executor(&ApiShow {}));
  static ref API_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> = vec![
    API_DELETE_CAPABILITY.as_ref(),
    API_GET_CAPABILITY.as_ref(),
    API_HEAD_CAPABILITY.as_ref(),
    API_PATCH_CAPABILITY.as_ref(),
    API_POST_CAPABILITY.as_ref(),
    API_PUT_CAPABILITY.as_ref(),
    API_SHOW_CAPABILITY.as_ref(),
  ];
}

const DELETE_COMMAND: &str = "delete";
const GET_COMMAND: &str = "get";
const HEAD_COMMAND: &str = "head";
const PATCH_COMMAND: &str = "patch";
const POST_COMMAND: &str = "post";
const PUT_COMMAND: &str = "put";

fn method_descriptors(method: &str) -> Option<&'static [(&str, MethodDescriptor)]> {
  match method {
    DELETE_COMMAND => Some(&DELETE_METHODS),
    GET_COMMAND => Some(&GET_METHODS),
    HEAD_COMMAND => Some(&HEAD_METHODS),
    PATCH_COMMAND => Some(&PATCH_METHODS),
    POST_COMMAND => Some(&POST_METHODS),
    PUT_COMMAND => Some(&PUT_METHODS),
    _ => None,
  }
}

fn method_descriptor(method: &'static str, query_selector: &str) -> Option<&'static MethodDescriptor> {
  match method_descriptors(method) {
    Some(method_descriptors) => method_descriptors
      .iter()
      .find_or_first(|(selector, _)| selector == &query_selector)
      .map(|(_, method_descriptor)| method_descriptor),
    None => None,
  }
}

fn create_generic_capability<'a>(method: &'static str, command_executor: &'a (dyn CommandExecutor + Send + Sync)) -> Box<(dyn Capability + Send + Sync + 'a)> {
  let subcommands = match method_descriptors(method) {
    Some(method_descriptors) => method_descriptors
      .iter()
      .map(|(selector, method_descriptor)| create_generic_capability_command(method, selector, method_descriptor))
      .collect::<Vec<_>>(),
    None => unreachable!(),
  };
  Box::new(
    CapabilityBuilder::new((method, ""), format!("{} methods ", method))
      .add_subcommands(subcommands)
      .set_default_command_executor(command_executor),
  )
}

fn create_generic_capability_command(method_command: &str, selector: &str, method_descriptor: &MethodDescriptor) -> Command {
  let mut command = Command::new(selector.to_string()).alias(method_descriptor.path);
  if let Some(description) = method_descriptor.description {
    command = command.about(create_about(method_command, method_descriptor, description));
  }
  if !method_descriptor.parameters.is_empty() {
    command = command.args(
      method_descriptor
        .parameters
        .iter()
        .map(|(parameter_name, _, description)| {
          let mut arg = Arg::new(parameter_name).value_name(parameter_name.to_ascii_uppercase().to_string()).required(true);
          if let Some(description) = description {
            arg = arg.help(description);
          }
          arg
        })
        .collect::<Vec<_>>(),
    )
  }
  command
}

fn create_about(method_command: &str, method_descriptor: &MethodDescriptor, description: &str) -> String {
  [
    Some(description.to_string()),
    Some(format!("{} {}", method_command.to_ascii_uppercase(), method_descriptor.path)),
    if method_descriptor.parameters.is_empty() {
      None
    } else {
      Some(
        method_descriptor
          .parameters
          .iter()
          .map(|(parameter_name, parameter_type, parameter_description)| {
            format!(
              "- {}: {}{}",
              parameter_name,
              parameter_description.unwrap_or_default(),
              if parameter_type == &"str" { "".to_string() } else { format!(" ({})", parameter_type) }
            )
          })
          .collect::<Vec<_>>()
          .join("\n"),
      )
    },
    method_descriptor.body_type.map(|body_type| format!("body type: {}", body_type)),
  ]
  .iter()
  .flatten()
  .join("\n")
}

struct ApiDelete {}

#[async_trait]
impl CommandExecutor for ApiDelete {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((selector, matches)) => match method_descriptor("delete", selector) {
        Some(method_descriptor) => {
          context.print_explanation(format!("DELETE {}", method_descriptor.path));
          if context.confirmed("type 'yes' to delete: ")? {
            if context.dry_run {
              context.print_warning("dry-run mode, nothing deleted");
            } else {
              let parameters = method_descriptor
                .parameters
                .iter()
                .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
                .collect::<Vec<_>>();
              let start_instant = Instant::now();
              let response = context.dsh_api_client.as_ref().unwrap().delete(selector, &parameters).await?;
              context.print_execution_time(start_instant);
              context.print_serializable(response);
              context.print_outcome("deleted");
            }
          } else {
            context.print_outcome("cancelled, nothing deleted");
          }
        }
        None => unreachable!(),
      },
      None => unreachable!(),
    }
    Ok(())
  }
}

struct ApiGet {}

#[async_trait]
impl CommandExecutor for ApiGet {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((selector, matches)) => match method_descriptor("get", selector) {
        Some(method_descriptor) => {
          context.print_explanation(format!("GET {}", method_descriptor.path));
          let parameters = method_descriptor
            .parameters
            .iter()
            .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
            .collect::<Vec<_>>();
          let start_instant = Instant::now();
          let response = context.dsh_api_client.as_ref().unwrap().get(selector, &parameters).await?;
          context.print_execution_time(start_instant);
          context.print_serializable(response);
        }

        // TODO        HIER!!! Gaat mis als er geen argumenten worden gegeven
        None => unreachable!(),
      },
      None => unreachable!(),
    }
    Ok(())
  }
}

struct ApiPatch {}

#[async_trait]
impl CommandExecutor for ApiPatch {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((selector, matches)) => match method_descriptor("patch", selector) {
        Some(method_descriptor) => {
          context.print_explanation(format!("PATCH {}", method_descriptor.path));
          let parameters = method_descriptor
            .parameters
            .iter()
            .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
            .collect::<Vec<_>>();
          let body =
            if method_descriptor.body_type.is_some() { Some(context.read_multi_line("enter json request body (terminate input with ctrl-d after last line)")?) } else { None };
          if context.dry_run {
            context.print_warning("dry-run mode, nothing patched");
            Ok(())
          } else {
            let start_instant = Instant::now();
            context.dsh_api_client.as_ref().unwrap().patch(selector, &parameters, body).await?;
            context.print_execution_time(start_instant);
            context.print_outcome("patched");
            Ok(())
          }
        }
        None => unreachable!(),
      },
      None => unreachable!(),
    }
  }
}

struct ApiPost {}

#[async_trait]
impl CommandExecutor for ApiPost {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((selector, matches)) => match method_descriptor("post", selector) {
        Some(method_descriptor) => {
          context.print_explanation(format!("POST {}", method_descriptor.path));
          let parameters = method_descriptor
            .parameters
            .iter()
            .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
            .collect::<Vec<_>>();
          let body =
            if method_descriptor.body_type.is_some() { Some(context.read_multi_line("enter json request body (terminate input with ctrl-d after last line)")?) } else { None };
          if context.dry_run {
            context.print_warning("dry-run mode, nothing posted");
            Ok(())
          } else {
            let start_instant = Instant::now();
            context.dsh_api_client.as_ref().unwrap().post(selector, &parameters, body).await?;
            context.print_execution_time(start_instant);
            context.print_outcome("posted");
            Ok(())
          }
        }
        None => unreachable!(),
      },
      None => unreachable!(),
    }
  }
}

struct ApiPut {}

#[async_trait]
impl CommandExecutor for ApiPut {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    match matches.subcommand() {
      Some((selector, matches)) => match method_descriptor("put", selector) {
        Some(method_descriptor) => {
          context.print_explanation(format!("PUT {}", method_descriptor.path));
          let parameters = method_descriptor
            .parameters
            .iter()
            .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
            .collect::<Vec<_>>();
          let body =
            if method_descriptor.body_type.is_some() { Some(context.read_multi_line("enter json request body (terminate input with ctrl-d after last line)")?) } else { None };
          if context.dry_run {
            context.print_warning("dry-run mode, nothing put");
            Ok(())
          } else {
            let start_instant = Instant::now();
            context.dsh_api_client.as_ref().unwrap().put(selector, &parameters, body).await?;
            context.print_execution_time(start_instant);
            context.print_outcome("put");
            Ok(())
          }
        }
        None => unreachable!(),
      },
      None => unreachable!(),
    }
  }
}

struct ApiShow {}

#[async_trait]
impl CommandExecutor for ApiShow {
  async fn execute(&self, _target: Option<String>, _: Option<String>, _matches: &ArgMatches, context: &Context) -> DshCliResult {
    context.print_explanation("print the open api specification");
    context.print(DshApiClient::openapi_spec());
    Ok(())
  }
}
