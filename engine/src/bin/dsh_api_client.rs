use std::time::SystemTime;

use trifonius_engine::processor::dsh_service::dsh_api_client::ServiceClient;
use trifonius_engine::processor::{ProcessorId, ServiceId, ServiceName, TaskId};

const PROCESSOR_ID: &str = "consentfilter";
const SERVICE_ID: &str = "test002";
const SERVICE_NAME: &str = "consentfilter-test002";
const TASK_ID: &str = "8f4b5747-lnmj4-00000000";
// const TASK_ID: &str = "cd5c9d9f9-ftz8t-00000000";

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();

  ProcessorId::regex();
  let _processor_id = ProcessorId::new(PROCESSOR_ID);
  let _service_id = ServiceId::new(SERVICE_ID);
  let service_name = ServiceName::new(SERVICE_NAME);
  let task_id = TaskId::new(TASK_ID);
  let client = ServiceClient::new();

  let start_time = SystemTime::now();

  // Returns all deployed services with their configuration
  let all_deployed_services = client.get_all_deployed_service_configurations().await?;
  println!("{}", serde_json::to_string_pretty(&all_deployed_services).unwrap());

  // Returns configuration of deployed service
  let deployed_service_configuration = client.get_deployed_service_configuration(&service_name).await?;
  println!("{}", serde_json::to_string_pretty(&deployed_service_configuration).unwrap());

  // Returns service configuration
  let service_configuration = client.get_service_configuration(&service_name).await?;
  println!("{}", serde_json::to_string_pretty(&service_configuration).unwrap());

  // Returns status of a service
  let service_status = client.get_service_status(&service_name).await?;
  println!("{}", serde_json::to_string_pretty(&service_status).unwrap());

  // Returns all services with their configuration
  let all_service_configurations = client.get_all_service_configurations().await?;
  println!("{}", serde_json::to_string_pretty(&all_service_configurations).unwrap());

  // Returns all services that have derived tasks
  let all_services_with_derived_tasks = client.get_all_services_with_derived_tasks().await?;
  println!("{}", serde_json::to_string_pretty(&all_services_with_derived_tasks).unwrap());

  // Returns all derived task ids
  let all_derived_task_ids = client.get_all_derived_task_ids(&service_name).await?;
  println!("{}", serde_json::to_string_pretty(&all_derived_task_ids).unwrap());

  // Returns status of a derived task
  let derived_task_status = client.get_derived_task_status(&service_name, &task_id).await?;
  println!("{}", serde_json::to_string_pretty(&derived_task_status).unwrap());

  // Returns task status description
  let derived_task_status_description = client.get_task_status_description(&service_name, &task_id).await?;
  println!("{}", serde_json::to_string_pretty(&derived_task_status_description).unwrap());

  // Returns task state
  let task_state = client.get_task_state(&service_name, &task_id).await?;
  println!("{}", serde_json::to_string_pretty(&task_state).unwrap());

  println!("{:?}", SystemTime::now().duration_since(start_time));

  Ok(())
}
