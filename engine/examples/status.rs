use crate::common::default_dshservice_instance;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), String> {
  let dshservice_instance = default_dshservice_instance();
  let status = dshservice_instance.status().await?;
  println!("{}", status);
  Ok(())
}
