//! # Manage applications
//!
//! Module that contains a function to manage applications.
//!
//! * [`create_application(application_id, application) -> ()`](DshApiClient::create_application)
//! * [`delete_application(application_id) -> ()`](DshApiClient::delete_application)
//! * [`get_application_actual_configuration(application_id) -> Application`](DshApiClient::get_application_actual_configuration)
//! * [`get_application_actual_configurations() -> HashMap<ApplicationId, Application>`](DshApiClient::get_application_actual_configurations)
//! * [`get_application_allocation_status(application_id) -> AllocationStatus`](DshApiClient::get_application_allocation_status)
//! * [`get_application_configuration(application_id) -> Application`](DshApiClient::get_application_configuration)
//! * [`get_application_configurations() -> HashMap<ApplicationId, Application>`](DshApiClient::get_application_configurations)
//! * [`get_application_derived_task_ids(application_id) -> Vec<TaskId>`](DshApiClient::get_application_derived_task_ids)
//! * [`get_application_ids() -> Vec<ApplicationId>`](DshApiClient::get_application_ids)
//! * [`get_application_ids_with_derived_tasks() -> Vec<ApplicationId>`](DshApiClient::get_application_ids_with_derived_tasks)
//! * [`get_application_task(application_id, task_id) -> TaskStatus`](DshApiClient::get_application_task)
//! * [`get_application_task_allocation_status(application_id, task_id) -> AllocationStatus`](DshApiClient::get_application_task_allocation_status)
//! * [`get_application_task_state(application_id, task_id) -> Task`](DshApiClient::get_application_task_state)

use std::collections::HashMap;

use crate::dsh_api_client::DshApiClient;
use crate::types::{AllocationStatus, Application, ApplicationSecret, ApplicationVolumes, HealthCheck, Metrics, PortMapping, Task, TaskStatus};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::DshApiResult;

/// # Manage applications
///
/// Module that contains a function to manage applications.
///
/// * [`create_application(application_id, application) -> ()`](DshApiClient::create_application)
/// * [`delete_application(application_id) -> ()`](DshApiClient::delete_application)
/// * [`get_application_actual_configuration(application_id) -> Application`](DshApiClient::get_application_actual_configuration)
/// * [`get_application_actual_configurations() -> HashMap<ApplicationId, Application>`](DshApiClient::get_application_actual_configurations)
/// * [`get_application_allocation_status(application_id) -> AllocationStatus`](DshApiClient::get_application_allocation_status)
/// * [`get_application_configuration(application_id) -> Application`](DshApiClient::get_application_configuration)
/// * [`get_application_configurations() -> HashMap<ApplicationId, Application>`](DshApiClient::get_application_configurations)
/// * [`get_application_derived_task_ids(application_id) -> Vec<TaskId>`](DshApiClient::get_application_derived_task_ids)
/// * [`get_application_ids() -> Vec<ApplicationId>`](DshApiClient::get_application_ids)
/// * [`get_application_ids_with_derived_tasks() -> Vec<ApplicationId>`](DshApiClient::get_application_ids_with_derived_tasks)
/// * [`get_application_task(application_id, task_id) -> TaskStatus`](DshApiClient::get_application_task)
/// * [`get_application_task_allocation_status(application_id, task_id) -> AllocationStatus`](DshApiClient::get_application_task_allocation_status)
/// * [`get_application_task_state(application_id, task_id) -> Task`](DshApiClient::get_application_task_state)
impl DshApiClient<'_> {
  /// # Deploy application
  ///
  /// `PUT /allocation/{tenant}/application/{appid}/configuration`
  ///
  /// ## Parameters
  /// * `application_id` - application name used when deploying the application
  /// * `configuration` - configuration used when deploying the application
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the application has been successfully
  ///              deployed)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_application(&self, application_id: &str, configuration: Application) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .application_put_by_tenant_application_by_appid_configuration(self.tenant_name(), application_id, self.token(), &configuration)
          .await,
      )
      .map(|result| result.1)
  }

  /// # Undeploy application
  ///
  /// `DELETE /allocation/{tenant}/application/{appid}/configuration`
  ///
  /// ## Parameters
  /// * `application_id` - application name of the application to undeploy
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the application has been successfully
  ///              undeployed)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_application(&self, application_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .application_delete_by_tenant_application_by_appid_configuration(self.tenant_name(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return configuration of deployed application
  ///
  /// `GET /allocation/{tenant}/application/{appid}/actual`
  ///
  /// ## Parameters
  /// * `application_id` - application id of the requested application
  ///
  /// ## Returns
  /// * `Ok<`[`Application`]`>` - application configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_actual_configuration(&self, application_id: &str) -> DshApiResult<Application> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_by_appid_actual(self.tenant_name(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return all deployed applications with their configurations
  ///
  /// `GET /allocation/{tenant}/application/actual`
  ///
  /// ## Returns
  /// * `Ok<HashMap<String, `[`Application`]`>>` - hashmap containing the application configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_actual_configurations(&self) -> DshApiResult<HashMap<String, Application>> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_actual(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return allocation status of application
  ///
  /// `GET /allocation/{tenant}/application/{appid}/status`
  ///
  /// ## Parameters
  /// * `application_id` - application id of the requested application
  ///
  /// ## Returns
  /// * `Ok<`[`AllocationStatus`]`>` - application allocation status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_allocation_status(&self, application_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_by_appid_status(self.tenant_name(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return application configuration
  ///
  /// `GET /allocation/{tenant}/application/{appid}/configuration`
  ///
  /// ## Parameters
  /// * `application_id` - application id of the requested application
  ///
  /// ## Returns
  /// * `Ok<`[`Application`]`>` - application configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_configuration(&self, application_id: &str) -> DshApiResult<Application> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_by_appid_configuration(self.tenant_name(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return all applications with their configuration
  ///
  /// `GET /allocation/{tenant}/application/configuration`
  ///
  /// ## Returns
  /// * `Ok<HashMap<String, `[`Application`]`>>` - hashmap containing the application configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_configurations(&self) -> DshApiResult<HashMap<String, Application>> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_configuration(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return all derived task ids for an application
  ///
  /// `GET /allocation/{tenant}/task{appid}`
  ///
  /// ## Parameters
  /// * `application_id` - application name for which the tasks will be returned
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - vector containing names of all derived tasks for the application
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_derived_task_ids(&self, application_id: &str) -> DshApiResult<Vec<String>> {
    let mut task_ids: Vec<String> = self
      .process(
        self
          .generated_client
          .application_get_by_tenant_task_by_appid(self.tenant_name(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
      .map(|task_ids| task_ids.iter().map(|task_id| task_id.to_string()).collect())?;
    task_ids.sort();
    Ok(task_ids)
  }

  /// # Return all application ids
  ///
  /// If you also need the application configuration, use
  /// [`get_application_configurations()`](Self::get_application_configurations) instead.
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - vector containing the sorted application ids
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_ids(&self) -> DshApiResult<Vec<String>> {
    let mut application_ids: Vec<String> = self
      .get_application_configurations()
      .await?
      .keys()
      .map(|application_id| application_id.to_string())
      .collect();
    application_ids.sort();
    Ok(application_ids)
  }

  /// # Return ids of all applications that have derived tasks
  ///
  /// `GET /allocation/{tenant}/task`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - vector containing names of all application that have derived tasks
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_ids_with_derived_tasks(&self) -> DshApiResult<Vec<String>> {
    let mut application_ids: Vec<String> = self
      .process(self.generated_client.application_get_by_tenant_task(self.tenant_name(), self.token()).await)
      .map(|result| result.1)
      .map(|secret_ids| secret_ids.iter().map(|secret_id| secret_id.to_string()).collect())?;
    application_ids.sort();
    Ok(application_ids)
  }

  /// # Return status of derived task
  ///
  /// `GET /allocation/{tenant}/task{appid}/{id}`
  ///
  /// This method combines the results of the methods
  /// [`get_application_task_actual()`](DshApiClient::get_application_task_state) and
  /// [`get_application_task_allocation_status()`](DshApiClient::get_application_task_allocation_status)
  /// into one method call.
  ///
  /// ## Parameters
  /// * `application_id` - application name of the requested application
  /// * `task_id` - id of the requested task
  ///
  /// ## Returns
  /// * `Ok<`[`TaskStatus`]`>` - application task status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_task(&self, application_id: &str, task_id: &str) -> DshApiResult<TaskStatus> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_task_by_appid_by_id(self.tenant_name(), application_id, task_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return task allocation status
  ///
  /// `GET /allocation/{tenant}/task{appid}/{id}/status`
  ///
  /// Note that the result of this method is also included in the result of the method
  /// [`get_application_task_actual()`](DshApiClient::get_application_task).
  ///
  /// ## Parameters
  /// * `application_id` - application name of the requested application
  /// * `task_id` - id of the requested task
  ///
  /// ## Returns
  /// * `Ok<`[`AllocationStatus`]`>` - application task allocation status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_task_allocation_status(&self, application_id: &str, task_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_task_by_appid_by_id_status(self.tenant_name(), application_id, task_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return task actual state
  ///
  /// `GET /allocation/{tenant}/task{appid}/{id}/actual`
  ///
  /// Note that the result of this method is also included in the result of the method
  /// [`get_application_task_actual()`](DshApiClient::get_application_task).
  ///
  /// ## Parameters
  /// * `application_id` - application name of the requested application
  /// * `task_id` - id of the requested task
  ///
  /// ## Returns
  /// * `Ok<`[`Task`]`>` - actual application task status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_application_task_state(&self, application_id: &str, task_id: &str) -> DshApiResult<Task> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_task_by_appid_by_id_actual(self.tenant_name(), application_id, task_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }
}

#[derive(Debug)]
pub struct ApplicationDiff {
  pub cpus: Option<(f64, f64)>,
  pub env: Option<(HashMap<String, String>, HashMap<String, String>)>,
  pub exposed_ports: Option<(HashMap<String, PortMapping>, HashMap<String, PortMapping>)>,
  pub health_check: Option<(Option<HealthCheck>, Option<HealthCheck>)>,
  pub image: Option<(String, String)>,
  pub instances: Option<(u64, u64)>,
  pub mem: Option<(u64, u64)>,
  pub metrics: Option<(Option<Metrics>, Option<Metrics>)>,
  pub needs_token: Option<(bool, bool)>,
  pub readable_streams: Option<(Vec<String>, Vec<String>)>,
  pub secrets: Option<(Vec<ApplicationSecret>, Vec<ApplicationSecret>)>,
  pub single_instance: Option<(bool, bool)>,
  pub spread_group: Option<(Option<String>, Option<String>)>,
  pub topics: Option<(Vec<String>, Vec<String>)>,
  pub user: Option<(String, String)>,
  pub volumes: Option<(HashMap<String, ApplicationVolumes>, HashMap<String, ApplicationVolumes>)>,
  pub writable_streams: Option<(Vec<String>, Vec<String>)>,
}

impl ApplicationDiff {
  pub fn is_empty(&self) -> bool {
    self.cpus.is_none()
      && self.env.is_none()
      && self.exposed_ports.is_none()
      && self.health_check.is_none()
      && self.image.is_none()
      && self.instances.is_none()
      && self.mem.is_none()
      && self.metrics.is_none()
      && self.needs_token.is_none()
      && self.readable_streams.is_none()
      && self.secrets.is_none()
      && self.single_instance.is_none()
      && self.spread_group.is_none()
      && self.topics.is_none()
      && self.user.is_none()
      && self.volumes.is_none()
      && self.writable_streams.is_none()
  }

  pub fn differences(&self) -> Vec<(String, String)> {
    vec![
      self.env.as_ref().map(|value| ("env".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self
        .exposed_ports
        .as_ref()
        .map(|value| ("exposed ports".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self
        .health_check
        .as_ref()
        .map(|value| ("healt check".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self.image.as_ref().map(|value| ("image".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self
        .instances
        .map(|value| ("number of instances".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self.mem.map(|value| ("memory".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self.metrics.as_ref().map(|value| ("metrics".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self.needs_token.map(|value| ("needs token".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self
        .readable_streams
        .as_ref()
        .map(|value| ("readable streams".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self.secrets.as_ref().map(|value| ("secrets".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self
        .single_instance
        .map(|value| ("single instance".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self
        .spread_group
        .as_ref()
        .map(|value| ("spread group".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self.topics.as_ref().map(|value| ("topics".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self.user.as_ref().map(|value| ("user".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self.volumes.as_ref().map(|value| ("volumes".to_string(), format!("{:?} / {:?}", value.0, value.1))),
      self
        .writable_streams
        .as_ref()
        .map(|value| ("writable streams".to_string(), format!("{:?} / {:?}", value.0, value.1))),
    ]
    .iter()
    .flatten()
    .collect::<Vec<_>>()
    .iter()
    .map(|p| p.to_owned().to_owned())
    .collect::<Vec<_>>()
  }
}

pub fn application_diff(baseline: &Application, sample: &Application) -> ApplicationDiff {
  ApplicationDiff {
    cpus: if baseline.cpus == sample.cpus { None } else { Some((baseline.cpus, sample.cpus)) },
    env: if baseline.env == sample.env { None } else { Some((baseline.env.clone(), sample.env.clone())) },
    exposed_ports: if baseline.exposed_ports == sample.exposed_ports.clone() { None } else { Some((baseline.exposed_ports.clone(), sample.exposed_ports.clone())) },
    health_check: if baseline.health_check == sample.health_check { None } else { Some((baseline.health_check.clone(), sample.health_check.clone())) },
    image: if baseline.image == sample.image.clone() { None } else { Some((baseline.image.clone(), sample.image.clone())) },
    instances: if baseline.instances == sample.instances { None } else { Some((baseline.instances, sample.instances)) },
    mem: if baseline.mem == sample.mem { None } else { Some((baseline.mem, sample.mem)) },
    metrics: if baseline.metrics == sample.metrics { None } else { Some((baseline.metrics.clone(), sample.metrics.clone())) },
    needs_token: if baseline.needs_token == sample.needs_token { None } else { Some((baseline.needs_token, sample.needs_token)) },
    readable_streams: if baseline.readable_streams == sample.readable_streams { None } else { Some((baseline.readable_streams.clone(), sample.readable_streams.clone())) },
    secrets: if baseline.secrets == sample.secrets { None } else { Some((baseline.secrets.clone(), sample.secrets.clone())) },
    single_instance: if baseline.single_instance == sample.single_instance { None } else { Some((baseline.single_instance, sample.single_instance)) },
    spread_group: if baseline.spread_group == sample.spread_group { None } else { Some((baseline.spread_group.clone(), sample.spread_group.clone())) },
    topics: if baseline.topics == sample.topics { None } else { Some((baseline.topics.clone(), sample.topics.clone())) },
    user: if baseline.user == sample.user { None } else { Some((baseline.user.clone(), sample.user.clone())) },
    volumes: if baseline.volumes == sample.volumes { None } else { Some((baseline.volumes.clone(), sample.volumes.clone())) },
    writable_streams: if baseline.writable_streams == sample.writable_streams { None } else { Some((baseline.writable_streams.clone(), sample.writable_streams.clone())) },
  }
}
