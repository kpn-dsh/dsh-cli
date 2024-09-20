use dsh_api::dsh_api_client_factory::DshApiClientFactory;
use std::collections::HashMap;

use dsh_api::types::{AllocationStatus, Application, Task, TaskStatus};

const APPLICATION_ID: &str = "consentfilter-test002";
const TASK_ID: &str = "8f4b5747-lnmj4-00000000";

#[tokio::main]
async fn main() -> Result<(), String> {
  let client_factory = DshApiClientFactory::default();
  let client = client_factory.client().await?;

  // deploy_application
  // undeploy_application

  // Show all applications
  let applications: HashMap<String, Application> = client.get_application_configurations().await?;
  println!(
    "get_applications() -> {} entries\n{}",
    applications.len(),
    serde_json::to_string_pretty(&applications).unwrap()
  );

  // Show all deployed applications
  let applications_actual: HashMap<String, Application> = client.get_application_actual_configurations().await?;
  println!(
    "get_applications_actual() -> {} entries\n{}",
    applications_actual.len(),
    serde_json::to_string_pretty(&applications_actual).unwrap()
  );

  // Show application
  let application: Application = client.get_application_configuration(APPLICATION_ID).await?;
  println!("get_application({})\n{}", APPLICATION_ID, serde_json::to_string_pretty(&application).unwrap());

  // Show deployed application
  let application: Application = client.get_application_actual_configuration(APPLICATION_ID).await?;
  println!(
    "get_application_actual({})\n{}",
    APPLICATION_ID,
    serde_json::to_string_pretty(&application).unwrap()
  );

  // Show application allocation status
  let allocation_status: AllocationStatus = client.get_application_allocation_status(APPLICATION_ID).await?;
  println!(
    "get_application_allocation_status({})\n{}",
    APPLICATION_ID,
    serde_json::to_string_pretty(&allocation_status).unwrap()
  );

  // List application ids with tasks
  let mut applications_with_tasks: Vec<String> = client.get_application_ids_with_derived_tasks().await?;
  applications_with_tasks.sort();
  println!("get_applications_with_tasks_ids() -> {}", applications_with_tasks.len());
  for application_id in applications_with_tasks {
    println!("{}", application_id);
  }

  // Show application task
  let task_status: TaskStatus = client.get_application_task(APPLICATION_ID, TASK_ID).await?;
  println!(
    "get_application_task({}, {})\n{}",
    APPLICATION_ID,
    TASK_ID,
    serde_json::to_string_pretty(&task_status).unwrap()
  );

  // Show deployed application task
  let task: Task = client.get_application_task_state(APPLICATION_ID, TASK_ID).await?;
  println!(
    "get_application_task_actual({}, {})\n{}",
    APPLICATION_ID,
    TASK_ID,
    serde_json::to_string_pretty(&task).unwrap()
  );

  // Show application task allocation status
  let allocation_status: AllocationStatus = client.get_application_task_allocation_status(APPLICATION_ID, TASK_ID).await?;
  println!(
    "get_application_task_allocation_status({}, {})\n{}",
    APPLICATION_ID,
    TASK_ID,
    serde_json::to_string_pretty(&allocation_status).unwrap()
  );

  // List application task ids
  let mut application_task_ids: Vec<String> = client.get_application_derived_task_ids(APPLICATION_ID).await?;
  application_task_ids.sort();
  println!("get_application_task_ids({}) -> {}", APPLICATION_ID, application_task_ids.len());
  for application_task_id in application_task_ids {
    println!("{}", application_task_id);
  }

  Ok(())
}
