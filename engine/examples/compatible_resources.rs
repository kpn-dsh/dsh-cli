use trifonius_engine::processor::JunctionId;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), String> {
  let dshservice_instance = crate::common::dshservice_instance();

  let inbound_dsh_topic = JunctionId::new("inbound-dsh-topic");
  let inbound_compatible_junctions = dshservice_instance.compatible_junctions(&inbound_dsh_topic).await;
  println!(
    "{}",
    serde_json::to_string_pretty(&inbound_compatible_junctions).map_err(|error| error.to_string())?
  );

  let outbound_dsh_topic = JunctionId::new("outbound-dsh-topic");
  let outbound_compatible_junctions = dshservice_instance.compatible_junctions(&outbound_dsh_topic).await;
  println!("{}", serde_json::to_string_pretty(&outbound_compatible_junctions).unwrap());

  Ok(())
}
