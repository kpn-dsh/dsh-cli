//! # Manage applications
//!
//! Module that contains a function to manage applications.
//!
//! * [`deploy_application(application_id, application) -> ()`](DshApiClient::deploy_application)
//! * [`get_application(application_id) -> Application`](DshApiClient::get_application)
//! * [`get_application_status(application_id) -> AllocationStatus`](DshApiClient::get_application_status)
//! * [`get_applications() -> HashMap<ApplicationId, Application>`](DshApiClient::get_applications)
//! * [`get_applications_with_tasks() -> Vec<ApplicationId>`](DshApiClient::get_applications_with_tasks)
//! * [`get_deployed_application(application_id) -> Application`](DshApiClient::get_deployed_application)
//! * [`get_deployed_applications() -> HashMap<ApplicationId, Application>`](DshApiClient::get_deployed_applications)
//! * [`get_task_allocation_status(application_id, task_id) -> AllocationStatus`](DshApiClient::get_task_allocation_status)
//! * [`get_task_state(application_id, task_id) -> Task`](DshApiClient::get_task_state)
//! * [`get_task_status(application_id, task_id) -> TaskStatus`](DshApiClient::get_task_status)
//! * [`get_tasks(application_id) -> Vec<TaskId>`](DshApiClient::get_tasks)
//! * [`undeploy_application(application_id) -> ()`](DshApiClient::undeploy_application)

use std::collections::HashMap;

use crate::types::{AllocationStatus, Application, Task, TaskStatus};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::{DshApiClient, DshApiResult};

/// # Manage applications
///
/// Module that contains a function to manage applications.
///
/// * [`deploy_application(application_id, application) -> ()`](DshApiClient::deploy_application)
/// * [`get_application(application_id) -> Application`](DshApiClient::get_application)
/// * [`get_application_status(application_id) -> AllocationStatus`](DshApiClient::get_application_status)
/// * [`get_applications() -> HashMap<ApplicationId, Application>`](DshApiClient::get_applications)
/// * [`get_applications_with_tasks() -> Vec<ApplicationId>`](DshApiClient::get_applications_with_tasks)
/// * [`get_deployed_application(application_id) -> Application`](DshApiClient::get_deployed_application)
/// * [`get_deployed_applications() -> HashMap<ApplicationId, Application>`](DshApiClient::get_deployed_applications)
/// * [`get_task_allocation_status(application_id, task_id) -> AllocationStatus`](DshApiClient::get_task_allocation_status)
/// * [`get_task_state(application_id, task_id) -> Task`](DshApiClient::get_task_state)
/// * [`get_task_status(application_id, task_id) -> TaskStatus`](DshApiClient::get_task_status)
/// * [`get_tasks(application_id) -> Vec<TaskId>`](DshApiClient::get_tasks)
/// * [`undeploy_application(application_id) -> ()`](DshApiClient::undeploy_application)
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
  pub async fn deploy_application(&self, application_id: &str, configuration: Application) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .application_put_by_tenant_application_by_appid_configuration(self.tenant_name(), application_id, self.token(), &configuration)
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
  pub async fn get_application(&self, application_id: &str) -> DshApiResult<Application> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_by_appid_configuration(self.tenant_name(), application_id, self.token())
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
  pub async fn get_application_status(&self, application_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_by_appid_status(self.tenant_name(), application_id, self.token())
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
  pub async fn get_applications(&self) -> DshApiResult<HashMap<String, Application>> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_configuration(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return names of all applications that have derived tasks
  ///
  /// `GET /allocation/{tenant}/task`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - vector containing names of all application that have derived tasks
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_applications_with_tasks(&self) -> DshApiResult<Vec<String>> {
    self
      .process(self.generated_client.application_get_by_tenant_task(self.tenant_name(), self.token()).await)
      .map(|result| result.1)
      .map(|application_ids| application_ids.iter().map(|application_id| application_id.to_string()).collect())
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
  pub async fn get_deployed_application(&self, application_id: &str) -> DshApiResult<Application> {
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
  pub async fn get_deployed_applications(&self) -> DshApiResult<HashMap<String, Application>> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_application_actual(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return task allocation status
  ///
  /// `GET /allocation/{tenant}/task{appid}/{id}/status`
  ///
  /// ## Parameters
  /// * `application_id` - application name of the requested application
  /// * `task_id` - id of the requested task
  ///
  /// ## Returns
  /// * `Ok<`[`AllocationStatus`]`>` - application task allocation status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_task_allocation_status(&self, application_id: &str, task_id: &str) -> DshApiResult<AllocationStatus> {
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
  /// ## Parameters
  /// * `application_id` - application name of the requested application
  /// * `task_id` - id of the requested task
  ///
  /// ## Returns
  /// * `Ok<`[`Task`]`>` - actual application rask status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_task_state(&self, application_id: &str, task_id: &str) -> DshApiResult<Task> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_task_by_appid_by_id_actual(self.tenant_name(), application_id, task_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return status of derived task
  ///
  /// `GET /allocation/{tenant}/task{appid}/{id}`
  ///
  /// ## Parameters
  /// * `application_id` - application name of the requested application
  /// * `task_id` - id of the requested task
  ///
  /// ## Returns
  /// * `Ok<`[`TaskStatus`]`>` - application task status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_task_status(&self, application_id: &str, task_id: &str) -> DshApiResult<TaskStatus> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_task_by_appid_by_id(self.tenant_name(), application_id, task_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return all derived tasks for an application
  ///
  /// `GET /allocation/{tenant}/task{appid}`
  ///
  /// ## Parameters
  /// * `application_id` - application name for which the tasks will be returned
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - vector containing names of all derived tasks for the application
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_tasks(&self, application_id: &str) -> DshApiResult<Vec<String>> {
    self
      .process(
        self
          .generated_client
          .application_get_by_tenant_task_by_appid(self.tenant_name(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
      .map(|task_ids| task_ids.iter().map(|task_id| task_id.to_string()).collect())
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
  pub async fn undeploy_application(&self, application_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .application_delete_by_tenant_application_by_appid_configuration(self.tenant_name(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }
}
