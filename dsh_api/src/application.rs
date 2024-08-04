use std::collections::HashMap;

use crate::types::{AllocationStatus, Application, Task, TaskStatus};
use crate::{DshApiClient, DshApiResult};

/// get_deployed_applications() -> HashMap<ApplicationId, Application>
/// get_deployed_application(application_id) -> Application
/// undeploy_application(application_id) -> ()
/// get_application(application_id) -> Application
/// get_application_status(application_id) -> AllocationStatus
/// get_applications() -> HashMap<ApplicationId, Application>
/// get_applications_with_tasks() -> Vec<ApplicationId>
/// get_tasks(application_id) -> Vec<TaskId>
/// get_task_status(application_id, task_id: &TaskId) -> TaskStatus
/// get_task_allocation_status(application_id, task_id: &TaskId) -> AllocationStatus
/// get_task_state(application_id, task_id: &TaskId) -> Task
/// deploy_application(application_id, application: Application) -> ()

impl DshApiClient<'_> {
  /// Returns all deployed services with their configuration
  ///
  /// `GET /allocation/{tenant}/application/actual`
  ///
  /// ```json
  /// {
  ///   "service-a": {
  ///     "...,
  ///     "user": "1903:1903",
  ///     "env": {
  ///       "LOG_LEVEL": "info",
  ///       ...,
  ///     },
  ///     ...
  ///   },
  ///   "service-b": {
  ///      ...
  ///   }
  /// }
  ///
  /// ```
  pub async fn get_deployed_applications(&self) -> DshApiResult<HashMap<String, Application>> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .application_get_by_tenant_application_actual(target_client.tenant(), target_client.token())
        .await,
    )
  }

  /// Returns configuration of deployed service
  ///
  /// `GET /allocation/{tenant}/application/{appid}/actual`
  ///
  /// ```json
  /// {
  ///   "...,
  ///   "user": "1903:1903",
  ///   "env": {
  ///     "LOG_LEVEL": "info",
  ///     ...,
  ///   },
  ///   ...
  /// }
  ///
  /// ```
  pub async fn get_deployed_application(&self, application_id: &str) -> DshApiResult<Application> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .application_get_by_tenant_application_by_appid_actual(target_client.tenant(), application_id, target_client.token())
        .await,
    )
  }

  /// Returns configuration of deployed service
  ///
  /// `DELETE /allocation/{tenant}/application/{appid}/configuration`
  ///
  /// ```json
  /// {
  ///   "...,
  ///   "user": "1903:1903",
  ///   "env": {
  ///     "LOG_LEVEL": "info",
  ///     ...,
  ///   },
  ///   ...
  /// }
  ///
  /// ```
  pub async fn undeploy_application(&self, application_id: &str) -> DshApiResult<()> {
    let target_client = self.target_client_factory.client().await?;
    self.process_delete(
      target_client
        .client()
        .application_delete_by_tenant_application_by_appid_configuration(target_client.tenant(), application_id, target_client.token())
        .await,
    )
  }

  /// Returns service configuration
  ///
  /// `GET /allocation/{tenant}/application/{appid}/configuration`
  ///
  /// ```json
  /// {
  ///   "...,
  ///   "user": "1903:1903",
  ///   "env": {
  ///     "LOG_LEVEL": "info",
  ///     ...,
  ///   },
  ///   ...
  /// }
  ///
  /// ```
  pub async fn get_application(&self, application_id: &str) -> DshApiResult<Application> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .application_get_by_tenant_application_by_appid_configuration(target_client.tenant(), application_id, target_client.token())
        .await,
    )
  }

  /// Returns status of a service
  ///
  /// `GET /allocation/{tenant}/application/{appid}/status`
  ///
  /// ```json
  /// {
  ///   "provisioned": true,
  ///   "notifications": []
  /// }
  /// ```
  pub async fn get_application_status(&self, application_id: &str) -> DshApiResult<AllocationStatus> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .application_get_by_tenant_application_by_appid_status(target_client.tenant(), application_id, target_client.token())
        .await,
    )
  }

  /// Returns all services with their configuration
  ///
  /// `GET /allocation/{tenant}/application/configuration`
  ///
  /// ```json
  /// {
  ///   "service-a": {
  ///     "...,
  ///     "user": "1903:1903",
  ///     "env": {
  ///       "LOG_LEVEL": "info",
  ///       ...,
  ///     },
  ///     ...
  ///   },
  ///   "service-b": {
  ///     ...
  ///   }
  /// }
  ///
  /// ```
  pub async fn get_applications(&self) -> DshApiResult<HashMap<String, Application>> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .application_get_by_tenant_application_configuration(target_client.tenant(), target_client.token())
        .await,
    )
  }

  /// Returns all services that have derived tasks
  /// ```json
  /// [
  ///   "service-a",
  ///   "service-b",
  ///   ...
  /// ]
  /// ```
  pub async fn get_applications_with_tasks(&self) -> DshApiResult<Vec<String>> {
    let target_client = self.target_client_factory.client().await?;
    self
      .process_get(
        target_client
          .client()
          .application_get_by_tenant_task(target_client.tenant(), target_client.token())
          .await,
      )
      .map(|application_ids| application_ids.iter().map(|application_id| application_id.to_string()).collect())
  }

  /// Returns all derived task ids
  /// ```json
  /// [
  ///   "56c97bb74-4b5w4-000000ed",
  ///   "56c97bb74-4b5w4-000000e3",
  ///   ...
  /// ]
  /// ```
  pub async fn get_tasks(&self, application_id: &str) -> DshApiResult<Vec<String>> {
    let target_client = self.target_client_factory.client().await?;
    self
      .process_get(
        target_client
          .client()
          .application_get_by_tenant_task_by_appid(target_client.tenant(), application_id, target_client.token())
          .await,
      )
      .map(|task_ids| task_ids.iter().map(|task_id| task_id.to_string()).collect())
  }

  /// Returns status of a derived task
  /// ```json
  /// {
  ///   "configuration": {
  ///     "healthy": true,
  ///     "host": "10.0.2.36",
  ///     "stagedAt": "2017-12-07T10:53:46.643Z",
  ///     "startedAt": "2017-12-07T10:55:41.765Z",
  ///     "stoppedAt": "2017-12-07T10:58:41.765Z",
  ///     "lastUpdate": 1639161445,
  ///     "state": "RUNNING"
  ///   },
  ///   "actual": {
  ///     "healthy": true,
  ///     "host": "10.0.2.36",
  ///     "stagedAt": "2017-12-07T10:53:46.643Z",
  ///     "startedAt": "2017-12-07T10:55:41.765Z",
  ///     "stoppedAt": "2017-12-07T10:58:41.765Z",
  ///     "lastUpdate": 1639161445,
  ///     "state": "RUNNING"
  ///   },
  ///   "status": {
  ///     "provisioned": true,
  ///     "notifications": []
  ///   }
  /// }
  /// ```
  pub async fn get_task_status(&self, application_id: &str, task_id: &str) -> DshApiResult<TaskStatus> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .application_get_by_tenant_task_by_appid_by_id(target_client.tenant(), application_id, task_id, target_client.token())
        .await,
    )
  }

  /// Returns task status description
  ///
  /// ```json
  /// {
  ///   "provisioned": true,
  ///   "notifications": []
  /// }
  /// ```
  ///
  pub async fn get_task_allocation_status(&self, application_id: &str, task_id: &str) -> DshApiResult<AllocationStatus> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .application_get_by_tenant_task_by_appid_by_id_status(target_client.tenant(), application_id, task_id, target_client.token())
        .await,
    )
  }

  /// Returns task state
  ///
  /// ```json
  /// {
  ///   "healthy": true,
  ///   "host": "10.0.2.36",
  ///   "stagedAt": "2017-12-07T10:53:46.643Z",
  ///   "startedAt": "2017-12-07T10:55:41.765Z",
  ///   "stoppedAt": "2017-12-07T10:58:41.765Z",
  ///   "lastUpdate": 1639161445,
  ///   "state": "RUNNING"
  /// }
  /// ```
  pub async fn get_task_state(&self, application_id: &str, task_id: &str) -> DshApiResult<Task> {
    let target_client = self.target_client_factory.client().await?;
    self.process_get(
      target_client
        .client()
        .application_get_by_tenant_task_by_appid_by_id_actual(target_client.tenant(), application_id, task_id, target_client.token())
        .await,
    )
  }

  /// Deploy application
  /// `PUT /allocation/{tenant}/application/{appid}/configuration`
  pub async fn deploy_application(&self, application_id: &str, application: Application) -> DshApiResult<()> {
    let target_client = self.target_client_factory.client().await?;
    self.process_put(
      target_client
        .client()
        .application_put_by_tenant_application_by_appid_configuration(target_client.tenant(), application_id, target_client.token(), &application)
        .await,
    )
  }
}
