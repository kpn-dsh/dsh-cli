use crate::common::default_dsh_service_instance;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), String> {
  let dsh_service_instance = default_dsh_service_instance();
  let status = dsh_service_instance.status().await?;
  println!("{}", status);
  Ok(())
}
