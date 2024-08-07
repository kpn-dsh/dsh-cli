//! # application
//!
//! Manage applications
//!
//! * `get_deployed_applications() -> HashMap<ApplicationId, Application>`
//! * `get_deployed_application(application_id) -> Application`
//! * `undeploy_application(application_id) -> ()`
//! * `get_application(application_id) -> Application`
//! * `get_application_status(application_id) -> AllocationStatus`
//! * `get_applications() -> HashMap<ApplicationId, Application>`
//! * `get_applications_with_tasks() -> Vec<ApplicationId>`
//! * `get_tasks(application_id) -> Vec<TaskId>`
//! * `get_task_status(application_id, task_id) -> TaskStatus`
//! * `get_task_allocation_status(application_id, task_id) -> AllocationStatus`
//! * `get_task_state(application_id, task_id) -> Task`
//! * `deploy_application(application_id, application) -> ()`

use std::collections::HashMap;

use crate::types::{AllocationStatus, Application, Task, TaskStatus};
use crate::DshApiClient;
use crate::DshApiResult;

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
  /// ```
  pub async fn get_deployed_applications(&self) -> DshApiResult<HashMap<String, Application>> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_application_actual(self.tenant(), self.token())
          .await,
      )
      .map(|result| result.1)
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
  /// ```
  /// ## Parameters
  /// `application_id` - service name of the requested application
  pub async fn get_deployed_application(&self, application_id: &str) -> DshApiResult<Application> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_application_by_appid_actual(self.tenant(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
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
  /// ```
  /// ## Parameters
  /// `application_id` - service name of the application to undeploy
  pub async fn undeploy_application(&self, application_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client()
          .application_delete_by_tenant_application_by_appid_configuration(self.tenant(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
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
  /// ```
  /// ## Parameters
  /// `application_id` - service name of the requested application
  pub async fn get_application(&self, application_id: &str) -> DshApiResult<Application> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_application_by_appid_configuration(self.tenant(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
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
  /// ## Parameters
  /// `application_id` - service name of the requested application
  pub async fn get_application_status(&self, application_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_application_by_appid_status(self.tenant(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
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
  /// ```
  pub async fn get_applications(&self) -> DshApiResult<HashMap<String, Application>> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_application_configuration(self.tenant(), self.token())
          .await,
      )
      .map(|result| result.1)
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
    self
      .process(self.generated_client().application_get_by_tenant_task(self.tenant(), self.token()).await)
      .map(|result| result.1)
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
  /// ## Parameters
  /// `application_id` - service name for which the tasks will be returned
  pub async fn get_tasks(&self, application_id: &str) -> DshApiResult<Vec<String>> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_task_by_appid(self.tenant(), application_id, self.token())
          .await,
      )
      .map(|result| result.1)
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
  /// ## Parameters
  /// `application_id` - service name of the requested application
  /// `task_id`        - id of the requested task
  pub async fn get_task_status(&self, application_id: &str, task_id: &str) -> DshApiResult<TaskStatus> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_task_by_appid_by_id(self.tenant(), application_id, task_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// Returns task status description
  ///
  /// ```json
  /// {
  ///   "provisioned": true,
  ///   "notifications": []
  /// }
  /// ```
  /// ## Parameters
  /// `application_id` - service name of the requested application
  /// `task_id`        - id of the requested task
  pub async fn get_task_allocation_status(&self, application_id: &str, task_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_task_by_appid_by_id_status(self.tenant(), application_id, task_id, self.token())
          .await,
      )
      .map(|result| result.1)
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
  /// ## Parameters
  /// `application_id` - service name of the requested application
  /// `task_id`        - id of the requested task
  pub async fn get_task_state(&self, application_id: &str, task_id: &str) -> DshApiResult<Task> {
    self
      .process(
        self
          .generated_client()
          .application_get_by_tenant_task_by_appid_by_id_actual(self.tenant(), application_id, task_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// Deploy application
  ///
  /// `PUT /allocation/{tenant}/application/{appid}/configuration`
  ///
  /// ## Parameters
  /// `application_id` - service name used when deploying the application
  /// `application`    - configuration used when deploying the application
  pub async fn deploy_application(&self, application_id: &str, application: Application) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client()
          .application_put_by_tenant_application_by_appid_configuration(self.tenant(), application_id, self.token(), &application)
          .await,
      )
      .map(|result| result.1)
  }
}
