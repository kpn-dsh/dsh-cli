use crate::common::dshservice_instance;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), String> {
  let dshservice_instance = dshservice_instance();
  println!("{}", dshservice_instance.undeploy().await?);
  Ok(())
}
