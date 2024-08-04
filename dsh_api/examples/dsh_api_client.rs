use std::time::SystemTime;
use trifonius_dsh_api::DshApiClient;

// use trifonius_engine::processor::dsh_service::{DshServiceName, TaskId};

const SERVICE_NAME: &str = "consentfilter-test002";
const TASK_ID: &str = "8f4b5747-lnmj4-00000000";

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();

  let service_name = SERVICE_NAME;
  let task_id = TASK_ID;
  let client = DshApiClient::new();

  let start_time = SystemTime::now();

  // Returns all deployed services with their configuration
  let all_deployed_services = client.get_deployed_applications().await?;
  println!("{}", serde_json::to_string_pretty(&all_deployed_services).unwrap());

  // Returns configuration of deployed service
  let deployed_service_configuration = client.get_deployed_application(&service_name).await?;
  println!("{}", serde_json::to_string_pretty(&deployed_service_configuration).unwrap());

  // Returns service configuration
  let service_configuration = client.get_application(&service_name).await?;
  println!("{}", serde_json::to_string_pretty(&service_configuration).unwrap());

  // Returns status of a service
  let service_status = client.get_application_status(&service_name).await?;
  println!("{}", serde_json::to_string_pretty(&service_status).unwrap());

  // Returns all services with their configuration
  let all_service_configurations = client.get_applications().await?;
  println!("{}", serde_json::to_string_pretty(&all_service_configurations).unwrap());

  // Returns all services that have derived tasks
  let all_services_with_derived_tasks = client.get_applications_with_tasks().await?;
  println!("{}", serde_json::to_string_pretty(&all_services_with_derived_tasks).unwrap());

  // Returns all derived task ids
  let all_derived_task_ids = client.get_tasks(&service_name).await?;
  println!("{}", serde_json::to_string_pretty(&all_derived_task_ids).unwrap());

  // Returns status of a derived task
  let derived_task_status = client.get_task_allocation_status(&service_name, &task_id).await?;
  println!("{}", serde_json::to_string_pretty(&derived_task_status).unwrap());

  // Returns task status description
  let derived_task_status_description = client.get_task_state(&service_name, &task_id).await?;
  println!("{}", serde_json::to_string_pretty(&derived_task_status_description).unwrap());

  // Returns task state
  let task_state = client.get_task_status(&service_name, &task_id).await?;
  println!("{}", serde_json::to_string_pretty(&task_state).unwrap());

  println!("{:?}", SystemTime::now().duration_since(start_time));

  Ok(())
}
