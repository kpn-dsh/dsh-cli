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

use crate::types::{AllocationStatus, Application, Task, TaskStatus};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::{DshApiClient, DshApiResult};

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
