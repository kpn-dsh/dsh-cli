use crate::capability::{Capability, CommandExecutor, SHOW_COMMAND, SHOW_COMMAND_ALIAS};
use crate::capability_builder::CapabilityBuilder;
use crate::context::Context;
use crate::formatters::OutputFormat;
use crate::subject::{Requirements, Subject};
use crate::DshCliResult;
use async_trait::async_trait;
use clap::{builder, Arg, ArgAction, ArgMatches, Command};
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::generic::{MethodDescriptor, DELETE_METHODS, GET_METHODS, POST_METHODS, PUT_METHODS};
#[cfg(feature = "manage")]
use dsh_api::generic::{HEAD_METHODS, PATCH_METHODS};
use itertools::Itertools;
use lazy_static::lazy_static;

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

  fn capability(&self, capability_command: &str) -> Option<&(dyn Capability + Send + Sync)> {
    match capability_command {
      DELETE_COMMAND => Some(API_DELETE_CAPABILITY.as_ref()),
      GET_COMMAND => Some(API_GET_CAPABILITY.as_ref()),
      #[cfg(feature = "manage")]
      HEAD_COMMAND => Some(API_HEAD_CAPABILITY.as_ref()),
      #[cfg(feature = "manage")]
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

#[cfg(feature = "manage")]
lazy_static! {
  static ref API_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(DELETE_COMMAND, DELETE_ABOUT, DELETE_LONG_ABOUT, &ApiDelete {});
  static ref API_GET_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(GET_COMMAND, GET_ABOUT, GET_LONG_ABOUT, &ApiGet {});
  static ref API_HEAD_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(HEAD_COMMAND, HEAD_ABOUT, HEAD_LONG_ABOUT, &ApiHead {});
  static ref API_PATCH_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(PATCH_COMMAND, PATCH_ABOUT, PATCH_LONG_ABOUT, &ApiPatch {});
  static ref API_POST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(POST_COMMAND, POST_ABOUT, POST_LONG_ABOUT, &ApiPost {});
  static ref API_PUT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(PUT_COMMAND, PUT_ABOUT, PUT_LONG_ABOUT, &ApiPut {});
  static ref API_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), "Print the open api specification").set_default_command_executor(&ApiShow {}));
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

#[cfg(not(feature = "manage"))]
lazy_static! {
  static ref API_DELETE_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(DELETE_COMMAND, DELETE_ABOUT, DELETE_LONG_ABOUT, &ApiDelete {});
  static ref API_GET_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(GET_COMMAND, GET_ABOUT, GET_LONG_ABOUT, &ApiGet {});
  static ref API_POST_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(POST_COMMAND, POST_ABOUT, POST_LONG_ABOUT, &ApiPost {});
  static ref API_PUT_CAPABILITY: Box<(dyn Capability + Send + Sync)> = create_generic_capability(PUT_COMMAND, PUT_ABOUT, PUT_LONG_ABOUT, &ApiPut {});
  static ref API_SHOW_CAPABILITY: Box<(dyn Capability + Send + Sync)> =
    Box::new(CapabilityBuilder::new(SHOW_COMMAND, Some(SHOW_COMMAND_ALIAS), "Print the open api specification").set_default_command_executor(&ApiShow {}));
  static ref API_CAPABILITIES: Vec<&'static (dyn Capability + Send + Sync)> =
    vec![API_DELETE_CAPABILITY.as_ref(), API_GET_CAPABILITY.as_ref(), API_POST_CAPABILITY.as_ref(), API_PUT_CAPABILITY.as_ref(), API_SHOW_CAPABILITY.as_ref(),];
}

const DELETE_COMMAND: &str = "delete";
const GET_COMMAND: &str = "get";
#[cfg(feature = "manage")]
const HEAD_COMMAND: &str = "head";
#[cfg(feature = "manage")]
const PATCH_COMMAND: &str = "patch";
const POST_COMMAND: &str = "post";
const PUT_COMMAND: &str = "put";

fn method_descriptors(method: &str) -> &'static [(&str, MethodDescriptor)] {
  match method {
    DELETE_COMMAND => &DELETE_METHODS,
    GET_COMMAND => &GET_METHODS,
    #[cfg(feature = "manage")]
    HEAD_COMMAND => &HEAD_METHODS,
    #[cfg(feature = "manage")]
    PATCH_COMMAND => &PATCH_METHODS,
    POST_COMMAND => &POST_METHODS,
    PUT_COMMAND => &PUT_METHODS,
    _ => unreachable!(),
  }
}

fn find_method_descriptor(method: &'static str, query_selector: &str) -> Option<&'static MethodDescriptor> {
  method_descriptors(method)
    .iter()
    .find_or_first(|(selector, _)| selector == &query_selector)
    .map(|(_, method_descriptor)| method_descriptor)
}

fn create_generic_capability<'a>(
  method: &'static str,
  about: &str,
  long_about: &str,
  command_executor: &'a (dyn CommandExecutor + Send + Sync),
) -> Box<(dyn Capability + Send + Sync + 'a)> {
  let subcommands = method_descriptors(method)
    .iter()
    .map(|(selector, method_descriptor)| create_generic_capability_selector_command(method, selector, method_descriptor))
    .collect::<Vec<_>>();
  Box::new(
    CapabilityBuilder::new(method, None, about)
      .set_long_about(long_about)
      .add_subcommands(subcommands)
      .set_default_command_executor(command_executor),
  )
}

fn create_generic_capability_selector_command(method_command: &str, selector: &str, method_descriptor: &MethodDescriptor) -> Command {
  let mut command = Command::new(selector.to_string()).alias(method_descriptor.path);
  if let Some(description) = method_descriptor.description {
    command = command.about(create_about(description));
    command = command.long_about(create_long_about(method_command, method_descriptor, description));
  }
  if !method_descriptor.parameters.is_empty() {
    command = command.args(
      method_descriptor
        .parameters
        .iter()
        .map(|(parameter_name, _, description)| {
          let mut arg = Arg::new(parameter_name)
            .value_name(parameter_name.to_ascii_uppercase().to_string())
            .action(ArgAction::Set)
            .value_parser(builder::NonEmptyStringValueParser::new())
            .required(true);
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

fn create_about(description: &str) -> String {
  let first = match description.split(". ").collect::<Vec<&str>>().first() {
    Some(first) => first.to_string(),
    None => description.trim_end_matches('.').to_string(),
  };
  first.trim_end_matches('.').to_string()
}

fn create_long_about(method_command: &str, method_descriptor: &MethodDescriptor, description: &str) -> String {
  let response_string = method_descriptor
    .response_type
    .map(|response_type| {
      if method_command == "get" {
        if response_type == "String" {
          "Returns a string.".to_string()
        } else if response_type == "Vec<String>" {
          "Returns a list of identifiers.".to_string()
        } else {
          format!("Returns a formatted {}.", response_type)
        }
      } else {
        format!("Returns ok when {}.", response_type)
      }
    })
    .unwrap_or_default();
  [
    Some(format!("{} {}", method_command.to_ascii_uppercase(), method_descriptor.path)),
    Some(description.to_string()),
    if method_descriptor.parameters.is_empty() {
      None
    } else {
      Some(format!(
        "Parameters:\n{}",
        method_descriptor
          .parameters
          .iter()
          .map(|(parameter_name, parameter_type, parameter_description)| {
            format!(
              "- {}: {}{}",
              parameter_name,
              parameter_description.unwrap_or_default(),
              if parameter_type == &"&str" { "".to_string() } else { format!(" (string representing a {})", parameter_type.trim_start_matches('&')) }
            )
          })
          .collect::<Vec<_>>()
          .join("\n"),
      ))
    },
    Some(
      method_descriptor
        .body_type
        .map(|body_type| {
          if body_type == "String" {
            format!(
              "Requires a string. This string can either be piped from another application, redirected from a file or provided by the user interactively. {}",
              response_string
            )
          } else {
            format!(
          "Requires string data representing a {}. This string data can either be piped from another application, redirected from a file or provided by the user interactively. {}",
          body_type, response_string
        )
          }
        })
        .unwrap_or(response_string),
    ),
  ]
  .iter()
  .flatten()
  .join("\n")
}

struct ApiDelete {}

#[async_trait]
impl CommandExecutor for ApiDelete {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (selector, matches) = matches.subcommand().unwrap_or_else(|| unreachable!());
    let method_descriptor = find_method_descriptor("delete", selector).unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("DELETE {}", method_descriptor.path));
    if context.confirmed("delete?")? {
      if context.dry_run {
        context.print_warning("dry-run mode, nothing deleted");
      } else {
        let parameters = method_descriptor
          .parameters
          .iter()
          .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
          .collect::<Vec<_>>();
        let start_instant = context.now();
        context.client_unchecked().delete(selector, &parameters).await?;
        context.print_execution_time(start_instant);
        context.print_outcome("deleted");
      }
    } else {
      context.print_outcome("cancelled, nothing deleted");
    }
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApiGet {}

#[async_trait]
impl CommandExecutor for ApiGet {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (selector, matches) = matches.subcommand().unwrap_or_else(|| unreachable!());
    let method_descriptor = find_method_descriptor("get", selector).unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("GET {}", method_descriptor.path));
    let parameters = method_descriptor
      .parameters
      .iter()
      .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
      .collect::<Vec<_>>();
    let start_instant = context.now();
    let response = context.client_unchecked().get(selector, &parameters).await?;
    context.print_execution_time(start_instant);
    context.print_serializable(response);
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api_multiple(true, true, Some(OutputFormat::Json))
  }
}

#[cfg(feature = "manage")]
struct ApiHead {}

#[cfg(feature = "manage")]
#[async_trait]
impl CommandExecutor for ApiHead {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (selector, matches) = matches.subcommand().unwrap_or_else(|| unreachable!());
    let method_descriptor = find_method_descriptor("head", selector).unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("HEAD {}", method_descriptor.path));
    let parameters = method_descriptor
      .parameters
      .iter()
      .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
      .collect::<Vec<_>>();
    let start_instant = context.now();
    context.client_unchecked().head(selector, &parameters).await?;
    context.print_execution_time(start_instant);
    context.print_outcome("ok");
    Ok(())
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

#[cfg(feature = "manage")]
struct ApiPatch {}

#[cfg(feature = "manage")]
#[async_trait]
impl CommandExecutor for ApiPatch {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (selector, matches) = matches.subcommand().unwrap_or_else(|| unreachable!());
    let method_descriptor = find_method_descriptor("patch", selector).unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("PATCH {}", method_descriptor.path));
    let parameters = method_descriptor
      .parameters
      .iter()
      .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
      .collect::<Vec<_>>();
    let body = if method_descriptor.body_type.is_some() { Some(context.read_multi_line("enter json request body (terminate input with ctrl-d after last line)")?) } else { None };
    if context.dry_run {
      context.print_warning("dry-run mode, nothing patched");
      Ok(())
    } else {
      let start_instant = context.now();
      context.client_unchecked().patch(selector, &parameters, body).await?;
      context.print_execution_time(start_instant);
      context.print_outcome("patched");
      Ok(())
    }
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApiPost {}

#[async_trait]
impl CommandExecutor for ApiPost {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (selector, matches) = matches.subcommand().unwrap_or_else(|| unreachable!());
    let method_descriptor = find_method_descriptor("post", selector).unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("POST {}", method_descriptor.path));
    let parameters = method_descriptor
      .parameters
      .iter()
      .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
      .collect::<Vec<_>>();
    let body = if method_descriptor.body_type.is_some() { Some(context.read_multi_line("enter json request body (terminate input with ctrl-d after last line)")?) } else { None };
    if context.dry_run {
      context.print_warning("dry-run mode, nothing posted");
      Ok(())
    } else {
      let start_instant = context.now();
      context.client_unchecked().post(selector, &parameters, body).await?;
      context.print_execution_time(start_instant);
      context.print_outcome("posted");
      Ok(())
    }
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
  }
}

struct ApiPut {}

#[async_trait]
impl CommandExecutor for ApiPut {
  async fn execute(&self, _target: Option<String>, _sub_argument: Option<String>, matches: &ArgMatches, context: &Context) -> DshCliResult {
    let (selector, matches) = matches.subcommand().unwrap_or_else(|| unreachable!());
    let method_descriptor = find_method_descriptor("put", selector).unwrap_or_else(|| unreachable!());
    context.print_explanation(format!("PUT {}", method_descriptor.path));
    let parameters = method_descriptor
      .parameters
      .iter()
      .map(|(parameter_name, _, _)| matches.get_one::<String>(parameter_name).unwrap().as_str())
      .collect::<Vec<_>>();
    let body = if method_descriptor.body_type.is_some() { Some(context.read_multi_line("enter json request body (terminate input with ctrl-d after last line)")?) } else { None };
    if context.dry_run {
      context.print_warning("dry-run mode, nothing put");
      Ok(())
    } else {
      let start_instant = context.now();
      context.client_unchecked().put(selector, &parameters, body).await?;
      context.print_execution_time(start_instant);
      context.print_outcome("put");
      Ok(())
    }
  }

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_with_api(None)
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

  fn requirements(&self, _sub_matches: &ArgMatches) -> Requirements {
    Requirements::standard_without_api(Some(OutputFormat::Json))
  }
}

const DELETE_ABOUT: &str = "Call delete operation";
const DELETE_LONG_ABOUT: &str = "Call a delete operation on the DSH resource management api. \
   Delete operations are typically used to delete resources on the platform. \
   The type of the resource that needs to be deleted is specified by the required selector command, \
   which must directly follow the 'delete' command. \
   The resource instance that will be deleted must be specified by one or more required identifiers, \
   which must follow the selector command. \
   Delete operations do not require any further data or values, \
   but the user will be asked to confirm the action (unless the --force option was used). \
   A delete operation will only give a confirmation that the server accepted the request, \
   but this does not necessarily mean that the resource will be successfully deleted. \
   The method will return an error message when the server did not accept the request, \
   for example if the resource specified by the identifier does not exist.";

const GET_ABOUT: &str = "Call get operation";
const GET_LONG_ABOUT: &str = "Call a get operation on the DSH resource management api. \
   Get operations are typically used to request configuration or other data \
   from resources on the platform. \
   The type of the requested data needs to be specified by the required selector command, \
   which must directly follow the 'get' command. \
   The resource instance for which the request is done must be specified by one or more \
   required identifiers, which must follow the selector command. \
   Get operations do not require any further data or values. \
   A get operation will normally print the requested data to the stdout device. \
   The format of the printed data can be controlled by the --output-format option, \
   or else a default format will be used which can depend on the resource type \
   and the configured output. \
   The method will return an error message when the server did not accept the request, \
   for example if the resource specified by the identifier does not exist.";

#[cfg(feature = "manage")]
const HEAD_ABOUT: &str = "Call head operation";
#[cfg(feature = "manage")]
const HEAD_LONG_ABOUT: &str = "Call a head operation on the DSH resource management api. \
   Head operations are typically used to check whether a resource on the platform exists, \
   or if a user is entitled to use it. \
   The type of the resource that needs to be checked is specified by the required selector command, \
   which must directly follow the 'head' command. \
   The resource instance that will be checked must be specified by one or more required identifiers, \
   which must follow the selector command. \
   Head operations do not require any further data or values. \
   A head operation will return a positive confirmation if the resource exists \
   or if the user is entitled to use it. \
   It will return an error message when the resource specified by the identifier \
   does not exist or cannot be used by the user.";

#[cfg(feature = "manage")]
const PATCH_ABOUT: &str = "Call patch operation";
#[cfg(feature = "manage")]
const PATCH_LONG_ABOUT: &str = "Call a patch operation on the DSH resource management api. \
   Patch operations are typically used to update an already existing resources on the platform. \
   The type of the resource that needs to be updated is specified by the required selector command, \
   which must directly follow the 'patch' command. \
   The resource instance that will be patched must be specified by one or more required identifiers, \
   which must follow the selector command. \
   Patch operations sometimes require extra data or values, \
   which cannot be provided directly via the command line. \
   It can either be piped or redirected via the shell command, \
   or the user will be asked to provide it interactively. \
   A patch operation will only give a confirmation that the server accepted the request, \
   but this does not necessarily mean that the resource will be successfully patched. \
   The method will return an error message when the server did not accept the request, \
   for example if the resource specified by the identifier does not exist.";

const POST_ABOUT: &str = "Call post operation";
const POST_LONG_ABOUT: &str = "Call a post operation on the DSH resource management api. \
   Post operations are typically used to create new resources on the platform. \
   The type of the resource that needs to be posted is specified by the required selector command, \
   which must directly follow the 'post' command. \
   The resource instance that will be created must be identified by one or more required identifiers, \
   which must follow the selector command. \
   Post operations usually require extra data or values, \
   which cannot be provided directly via the command line. \
   It can either be piped from another application or redirected from a file via the shell command, \
   or the user will be asked to provide it interactively. \
   A post operation will only give a confirmation that the server accepted the request, \
   but this does not necessarily mean that the resource will be successfully created. \
   The method will return an error message when the server did not accept the request, \
   for example if the resource specified by the identifier already exists.";

const PUT_ABOUT: &str = "Call put operation";
const PUT_LONG_ABOUT: &str = "Call a put operation on the DSH resource management api. \
   Put operations are typically used to update existing resources on the platform, \
   but are sometimes also used to create new resources. \
   The type of the resource that needs to be put is specified by the required selector command, \
   which must directly follow the 'put' command. \
   The resource instance that will be updated/created must be identified \
   by one or more required identifiers, which must follow the selector command. \
   Put operations usually require extra data or values, \
   which cannot be provided directly via the command line. \
   It can either be piped from another application or redirected from a file via the shell command, \
   or the user will be asked to provide it interactively. \
   A put operation will only give a confirmation that the server accepted the request, \
   but this does not necessarily mean that the resource will be successfully updated. \
   The method will return an error message when the server did not accept the request, \
   for example if the resource specified by the identifier does not exist.";
