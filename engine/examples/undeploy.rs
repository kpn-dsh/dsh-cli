use crate::common::default_dsh_service_instance;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), String> {
  let dsh_service_instance = default_dsh_service_instance();
  println!("{}", dsh_service_instance.undeploy().await?);
  Ok(())
}
