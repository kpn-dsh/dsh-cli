use trifonius_dsh_api::dsh_api_client_factory::DshApiClientFactory;
use trifonius_dsh_api::types::{AllocationStatus, Task};

const SERVICE_ID: &str = "consentfilter-test002";
const TASK_ID: &str = "8f4b5747-lnmj4-00000000";

#[tokio::main]
async fn main() -> Result<(), String> {
  let application_id = SERVICE_ID;
  let task_id = TASK_ID;

  let client_factory = DshApiClientFactory::default();
  let client = client_factory.client().await?;

  // Return applications that have derived tasks
  let applications: Vec<String> = client.get_application_ids_with_derived_tasks().await?;
  println!("applications with tasks\n{}", serde_json::to_string_pretty(&applications).unwrap());

  // Return task ids
  let tasks: Vec<String> = client.get_application_derived_task_ids(&application_id).await?;
  println!("task ids {}\n{}", application_id, serde_json::to_string_pretty(&tasks).unwrap());

  // Return task allocation status
  let allocation_status: AllocationStatus = client.get_application_task_allocation_status(&application_id, &task_id).await?;
  println!(
    "task allocation status {}, {}\n{}",
    application_id,
    task_id,
    serde_json::to_string_pretty(&allocation_status).unwrap()
  );

  // Return task state
  let state: Task = client.get_application_task_state(&application_id, &task_id).await?;
  println!("task state {}, {}\n{}", application_id, task_id, serde_json::to_string_pretty(&state).unwrap());

  Ok(())
}
