use std::collections::HashMap;

use dsh_rest_api_client::types::{AllocationStatus, Application, ChildList, Task, TaskStatus};
use dsh_rest_api_client::Error::UnexpectedResponse;
use dsh_rest_api_client::{Error, ResponseValue};
use futures::future::join_all;
use log::debug;
use reqwest::StatusCode;
use serde::Serialize;

use crate::processor::dsh_service::{DshServiceName, TaskId};
use crate::target_client::{TargetClientFactory, DEFAULT_TARGET_CLIENT_FACTORY};

// TODO Move to a separate crate.

pub struct ServiceClient<'a> {
  target_client_factory: &'a TargetClientFactory,
}

// application_get_by_tenant_application_actual(&self) -> Result<HashMap<String, Application>, String>
// application_get_by_tenant_application_by_appid_actual(&self) -> Result<Application, String>
// application_get_by_tenant_application_by_appid_configuration(&self) -> Result<Application, String>
// application_get_by_tenant_application_by_appid_status(&self) -> Result<AllocationStatus, String>
// application_get_by_tenant_application_configuration(&self) -> Result<HashMap<String, Application>, String>
// application_get_by_tenant_task(&self) -> Result<ChildList, String>
// application_get_by_tenant_task_by_appid(&self) -> Result<ChildList, String>
// application_get_by_tenant_task_by_appid_by_id(&self, task_id: &str) -> Result<TaskStatus, String>
// application_get_by_tenant_task_by_appid_by_id_status(&self, task_id: &str) -> Result<AllocationStatus, String>
// application_get_by_tenant_task_by_appid_by_id_actual(&self, task_id: &str) -> Result<Task, String>

impl ServiceClient<'_> {
  pub fn new() -> Self {
    ServiceClient { target_client_factory: &DEFAULT_TARGET_CLIENT_FACTORY }
  }

  pub async fn _get_tasks(&self, service_name: &DshServiceName) -> Result<HashMap<TaskId, TaskStatus>, String> {
    // TODO Panics
    match self.get_all_derived_task_ids(service_name).await {
      Ok(task_ids) => Ok(
        join_all(task_ids.iter().map(|task_id| async {
          let task_id = TaskId::try_from(task_id.clone()).unwrap();
          (task_id.clone(), self.get_derived_task_status(service_name, &task_id).await.unwrap())
        }))
        .await
        .into_iter()
        .collect::<HashMap<TaskId, TaskStatus>>(),
      ),
      Err(_) => Err("".to_string()),
    }
  }

  /// Returns all deployed services with their configuration
  ///
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
  pub async fn get_all_deployed_service_configurations(&self) -> Result<HashMap<String, Application>, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_application_actual(target_client.tenant(), target_client.token())
        .await,
    )
  }

  /// Returns configuration of deployed service
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
  pub async fn get_deployed_service_configuration(&self, service_name: &DshServiceName) -> Result<Application, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_application_by_appid_actual(target_client.tenant(), service_name, target_client.token())
        .await,
    )
  }

  /// Returns service configuration
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
  pub async fn get_service_configuration(&self, service_name: &DshServiceName) -> Result<Application, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_application_by_appid_configuration(target_client.tenant(), service_name, target_client.token())
        .await,
    )
  }

  /// Returns status of a service
  /// ```json
  /// {
  ///   "provisioned": true,
  ///   "notifications": []
  /// }
  /// ```
  pub async fn get_service_status(&self, service_name: &DshServiceName) -> Result<AllocationStatus, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_application_by_appid_status(target_client.tenant(), service_name, target_client.token())
        .await,
    )
  }

  /// Returns all services with their configuration
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
  //      ...
  ///   }
  /// }
  ///
  /// ```
  pub async fn get_all_service_configurations(&self) -> Result<HashMap<String, Application>, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
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
  pub async fn get_all_services_with_derived_tasks(&self) -> Result<ChildList, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_task(target_client.tenant(), target_client.token())
        .await,
    )
  }

  /// Returns all derived task ids
  /// ```json
  /// [
  ///   "56c97bb74-4b5w4-000000ed",
  ///   "56c97bb74-4b5w4-000000e3",
  ///   ...
  /// ]
  /// ```
  pub async fn get_all_derived_task_ids(&self, service_name: &DshServiceName) -> Result<ChildList, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_task_by_appid(target_client.tenant(), service_name, target_client.token())
        .await,
    )
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
  pub async fn get_derived_task_status(&self, service_name: &DshServiceName, task_id: &TaskId) -> Result<TaskStatus, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_task_by_appid_by_id(target_client.tenant(), service_name, task_id, target_client.token())
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
  pub async fn get_task_status_description(&self, service_name: &DshServiceName, task_id: &TaskId) -> Result<AllocationStatus, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_task_by_appid_by_id_status(target_client.tenant(), service_name, task_id, target_client.token())
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
  pub async fn get_task_state(&self, service_name: &DshServiceName, task_id: &TaskId) -> Result<Task, String> {
    let target_client = self.target_client_factory.client().await?;
    self.process(
      target_client
        .client()
        .application_get_by_tenant_task_by_appid_by_id_actual(target_client.tenant(), service_name, task_id, target_client.token())
        .await,
    )
  }

  fn process<T: Serialize>(&self, response: Result<ResponseValue<T>, Error>) -> Result<T, String> {
    match response {
      Ok(response) => match response.status() {
        StatusCode::OK => {
          let inner = response.into_inner();
          debug!("{}", serde_json::to_string_pretty(&inner).unwrap());
          Ok(inner)
        }
        not_ok => {
          let inner = response.into_inner();
          debug!("{}", serde_json::to_string_pretty(&inner).unwrap());
          Err(format!("status code not ok ({})", not_ok))
        }
      },
      Err(UnexpectedResponse(response)) => match response.status() {
        StatusCode::NOT_FOUND => {
          debug!("{:#?}", response);
          Err(format!("unexpected response (status code {})", StatusCode::NOT_FOUND))
        }
        other => {
          debug!("{:#?}", response);
          Err(format!("unexpected response (status code {})", other))
        }
      },
      Err(error) => {
        debug!("{}", error);
        Err(format!("error ({})", error))
      }
    }
  }
}

impl Default for ServiceClient<'_> {
  fn default() -> Self {
    Self::new()
  }
}
