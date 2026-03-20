use crate::authentication::{get_access_token, AuthenticationMethod};
use crate::context::Context;
use crate::directory::supports_dsh_directory;
use crate::error::DshCliError;
use crate::global_arguments::{TARGET_TENANTS_ALL_ARGUMENT, TARGET_TENANTS_ARGUMENT};
use crate::robot_arguments::{get_robot_password, get_robot_platform, get_robot_tenant};
use crate::subject::Requirements;
use crate::target_arguments::{get_target_platform, get_target_tenant};
use crate::{err, DshCliResult};
use clap::ArgMatches;
use dsh_api::dsh_api_client::DshApiClient;
use dsh_api::dsh_api_client_factory::{DshApiClientFactory, DshApiPlatformClientFactory};
use dsh_api::dsh_api_tenant::DshApiTenant;
use dsh_api::platform::DshPlatform;
use futures::future::try_join_all;
use itertools::Itertools;
use log::debug;

/// Create clients
///
/// # Parameters
/// * `matches`
/// * `context`
///
/// Returns
/// * `Ok(Some(Vec<Client>))` - Client were successfully created. Note that there will always be at
///   least one client created, else an error is returned.
/// * `Ok(None)` - User needs to log in.
pub(crate) async fn create_clients(matches: &ArgMatches, requirements: &Requirements, context: &Context) -> DshCliResult<Option<Vec<DshApiClient>>> {
  match context.authentication_method() {
    AuthenticationMethod::Robot => create_client_robot_password(matches).await.map(Some),
    AuthenticationMethod::SingleSignOn => create_clients_access_token(matches, requirements, context).await,
  }
}

/// Create client from robot password
///
/// # Parameters
/// * `matches`
///
/// Returns
/// * `Ok(Vec<Client>)` - Client was successfully created. Note that there always be only one
///   client created.
async fn create_client_robot_password(matches: &ArgMatches) -> DshCliResult<Vec<DshApiClient>> {
  let robot_platform = get_robot_platform(matches)?;
  let robot_tenant_name = get_robot_tenant(matches)?;
  debug!("create client with token fetcher for '{}@{}'", robot_tenant_name, robot_platform);
  let dsh_api_tenant = DshApiTenant::new(robot_tenant_name, robot_platform);
  let robot_password = get_robot_password(matches, &dsh_api_tenant)?;
  let dsh_api_client_factory = DshApiClientFactory::create_with_token_fetcher(dsh_api_tenant, robot_password);
  let dsh_api_client = dsh_api_client_factory.client().await?;
  debug!("api client created");
  Ok(vec![dsh_api_client])
}

/// Create client from single sign on
///
/// # Parameters
/// * `matches`
/// * `requirements`
/// * `context`
///
/// Returns
/// * `Ok(Some(Vec<Client>))` - Clients were successfully created. Note that there will always be at
///   least one client created, else an error is returned.
/// * `Ok(None)` - User needs to log in.
async fn create_clients_access_token(matches: &ArgMatches, requirements: &Requirements, context: &Context) -> DshCliResult<Option<Vec<DshApiClient>>> {
  if !supports_dsh_directory() {
    return Err(DshCliError::String("single-sign-on requires dsh directory to be enabled".to_string()));
  }
  let target_platform = get_target_platform(matches, context.settings())?;
  if matches.get_flag(TARGET_TENANTS_ALL_ARGUMENT) {
    if requirements.all_tenants_allowed() {
      create_clients_for_all_authorized_tenants(target_platform).await
    } else {
      Err(DshCliError::String("command is not allowed to run for all tenants".to_string()))
    }
  } else {
    match matches.get_one::<String>(TARGET_TENANTS_ARGUMENT) {
      Some(target_tenants_string) => {
        let target_tenant_names = target_tenants_string.split(",").map(|s| s.to_string()).collect_vec();
        for target_tenant_name in &target_tenant_names {
          debug!("create client with static access token for target '{}@{}'", target_tenant_name, target_platform);
        }
        create_clients_for_tenants(target_platform, &target_tenant_names, context).await
      }
      None => {
        let target_tenant_name = get_target_tenant(matches, context.settings())?;
        match get_access_token(target_platform.clone()).await {
          Ok(Some((access_token, jwt))) => {
            if jwt
              .authorized_tenants()
              .is_some_and(|authorized_tenants| authorized_tenants.contains(&target_tenant_name.as_str()))
            {
              let dsh_api_tenant = DshApiTenant::new(target_tenant_name, target_platform);
              let dsh_api_client_factory = DshApiClientFactory::create_from_static_token(dsh_api_tenant, access_token);
              Ok(Some(vec![dsh_api_client_factory.client().await?]))
            } else {
              err!("not authorized for tenant '{}' at platform '{}'", target_tenant_name, target_platform)
            }
          }
          Ok(None) => {
            context.print_warning(format!("please log in to platform {} using the 'dsh login' command", target_platform));
            Ok(None)
          }
          Err(error) => Err(error),
        }
      }
    }
  }
}

/// Create client for explicit platform and tenant from single sign on
///
/// # Parameters
/// * `api_platform`
/// * `api_tenant`
/// * `context`
///
/// Returns
/// * `Ok(Some(Vec<Client>))` - Client successfully created.
/// * `Ok(None)` - User needs to log in.
pub(crate) async fn create_client_access_token_from_platform_tenant(api_platform: &DshPlatform, api_tenant: &str, context: &Context) -> DshCliResult<Option<DshApiClient>> {
  if supports_dsh_directory() {
    match get_access_token(api_platform.clone()).await {
      Ok(Some((access_token, jwt))) => {
        if jwt.authorized_tenants().is_some_and(|authorized_tenants| authorized_tenants.contains(&api_tenant)) {
          let dsh_api_tenant = DshApiTenant::new(api_tenant, api_platform.clone());
          let dsh_api_client_factory = DshApiClientFactory::create_from_static_token(dsh_api_tenant, access_token);
          Ok(Some(dsh_api_client_factory.client().await?))
        } else {
          err!("not authorized for tenant '{}' at platform '{}'", api_tenant, api_platform)
        }
      }
      Ok(None) => {
        context.print_warning(format!("please log in to platform '{}' using the 'dsh login' command", api_platform));
        Ok(None)
      }
      Err(error) => Err(error),
    }
  } else {
    Err(DshCliError::String("single-sign-on requires dsh directory to be enabled".to_string()))
  }
}

/// Create multiple clients from single sign on
///
/// # Parameters
/// * `target_platform`
/// * `target_tenant_names`
/// * `context`
///
/// Returns
/// * `Ok(Some(Vec<Client>))` - User is logged in and all clients were successfully created.
/// * `Ok(Some([]))` - User is logged in but no clients were created because the
///   user is not authorized for the requested tenants.
/// * `Ok(None)` - User needs to log in.
async fn create_clients_for_tenants(target_platform: DshPlatform, target_tenant_names: &[String], context: &Context) -> DshCliResult<Option<Vec<DshApiClient>>> {
  match get_access_token(target_platform.clone()).await {
    Ok(Some((access_token, jwt))) => {
      let unauthorized_tenants = target_tenant_names
        .iter()
        .filter(|target_tenant_name| {
          !jwt
            .authorized_tenants()
            .is_some_and(|authorized_tenants| authorized_tenants.contains(&target_tenant_name.as_str()))
        })
        .collect_vec();
      if !unauthorized_tenants.is_empty() {
        for unauthorized_tenant in &unauthorized_tenants {
          context.print_error(format!("not authorized for tenant {}@{}", unauthorized_tenant, target_platform));
        }
        return err!("not authorized for tenants {}", unauthorized_tenants.iter().join(", "));
      }
      let dsh_api_platform_client_factory = DshApiPlatformClientFactory::create_from_static_token(target_platform.clone(), access_token)?;
      let clients = try_join_all(target_tenant_names.iter().map(|target_tenant_name| {
        debug!("create client with static access token for target '{}@{}'", target_tenant_name, target_platform);
        dsh_api_platform_client_factory.client(target_tenant_name)
      }))
      .await?;
      debug!("api clients created");
      Ok(Some(clients))
    }
    Ok(None) => {
      context.print_warning(format!("please log in to platform '{}' using the 'dsh login' command", target_platform));
      Ok(None)
    }
    Err(error) => Err(error),
  }
}

/// Create clients for all authorized tenants from single sign on
///
/// # Parameters
/// * `target_platform`
///
/// Returns
/// * `Ok(Some(Vec<Client>))` - Clients were successfully created.
/// * `Ok(None)` - User needs to log in.
async fn create_clients_for_all_authorized_tenants(target_platform: DshPlatform) -> DshCliResult<Option<Vec<DshApiClient>>> {
  debug!("create client with static access token for all tenants at platform '{}'", target_platform);
  match get_access_token(target_platform.clone()).await {
    Ok(Some((access_token, jwt))) => match jwt.authorized_tenants() {
      Some(authorized_tenants) => {
        let dsh_api_platform_client_factory = DshApiPlatformClientFactory::create_from_static_token(target_platform, access_token)?;
        let clients = try_join_all(
          authorized_tenants
            .iter()
            .map(|authorized_tenant| dsh_api_platform_client_factory.client(*authorized_tenant)),
        )
        .await?;
        debug!("clients created");
        Ok(Some(clients))
      }
      None => err!("json web token does not provide authorized tenants"),
    },
    Ok(None) => Ok(None),
    Err(error) => Err(error),
  }
}
