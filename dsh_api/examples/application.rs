use std::collections::HashMap;

use trifonius_dsh_api::types::{AllocationStatus, Application, Task};
use trifonius_dsh_api::DEFAULT_DSH_API_CLIENT_FACTORY;

const SERVICE_NAME: &str = "consentfilter-test002";
const TASK_ID: &str = "8f4b5747-lnmj4-00000000";

#[tokio::main]
async fn main() -> Result<(), String> {
  let application_name = SERVICE_NAME;
  let task_id = TASK_ID;

  let client = &DEFAULT_DSH_API_CLIENT_FACTORY.client().await?;

  // Return deployed applications
  let applications: HashMap<String, Application> = client.get_deployed_applications().await?;
  println!("deployed applications\n{}", serde_json::to_string_pretty(&applications).unwrap());

  // Return deployed application
  let application: Application = client.get_deployed_application(&application_name).await?;
  println!("deployed application {}\n{}", application_name, serde_json::to_string_pretty(&application).unwrap());

  // Return applications
  let applications: HashMap<String, Application> = client.get_applications().await?;
  println!("applications\n{}", serde_json::to_string_pretty(&applications).unwrap());

  // Return application
  let application: Application = client.get_application(&application_name).await?;
  println!("application {}\n{}", application_name, serde_json::to_string_pretty(&application).unwrap());

  // Return application status
  let status: AllocationStatus = client.get_application_status(&application_name).await?;
  println!("application status {}\n{}", application_name, serde_json::to_string_pretty(&status).unwrap());

  // Return applications that have derived tasks
  let applications: Vec<String> = client.get_applications_with_tasks().await?;
  println!("applications with tasks\n{}", serde_json::to_string_pretty(&applications).unwrap());

  // Return task ids
  let tasks: Vec<String> = client.get_tasks(&application_name).await?;
  println!("task ids {}\n{}", application_name, serde_json::to_string_pretty(&tasks).unwrap());

  // Return task allocation status
  let allocation_status: AllocationStatus = client.get_task_allocation_status(&application_name, &task_id).await?;
  println!(
    "task allocation status {}, {}\n{}",
    application_name,
    task_id,
    serde_json::to_string_pretty(&allocation_status).unwrap()
  );

  // Return task state
  let state: Task = client.get_task_state(&application_name, &task_id).await?;
  println!("task state {}, {}\n{}", application_name, task_id, serde_json::to_string_pretty(&state).unwrap());

  // Return task status
  let status = client.get_task_status(&application_name, &task_id).await?;
  println!("task status {}, {}\n{}", application_name, task_id, serde_json::to_string_pretty(&status).unwrap());

  Ok(())
}
