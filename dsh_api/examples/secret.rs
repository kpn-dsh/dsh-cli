use dsh_api::dsh_api_client_factory::DshApiClientFactory;
use dsh_api::types::{AllocationStatus, Empty};

#[tokio::main]
async fn main() -> Result<(), String> {
  let test_secret_id = "test-secret";
  // let test_secret = Secret { name: test_secret_id.to_string(), value: "TEST_SECRET".to_string() };

  // println!("create_secret()\n{:?}", client.create_secret(&test_secret).await?);
  // let secret = client.delete_secret(secret_id).await?;
  // let secret = client.update_secret(secret_id).await?;

  let client_factory = DshApiClientFactory::default();
  let client = client_factory.client().await?;

  let mut secrets: Vec<String> = client.get_secret_ids().await?;
  secrets.sort();
  println!("get_secret_ids()");
  for secret in secrets {
    println!("{}", secret);
  }

  let secret: String = client.get_secret("greenbox_backend_password").await?;
  println!("get_secret(greenbox_backend_password)\n{}", secret);

  let secret_actual: Empty = client.get_secret_actual_configuration(test_secret_id).await?;
  println!("get_secret_actual({})\n{}", test_secret_id, serde_json::to_string_pretty(&secret_actual).unwrap());

  let secret_configuration: Empty = client.get_secret_configuration(test_secret_id).await?;
  println!(
    "get_secret_configuration({})\n{}",
    test_secret_id,
    serde_json::to_string_pretty(&secret_configuration).unwrap()
  );

  let secret_status: AllocationStatus = client.get_secret_allocation_status(test_secret_id).await?;
  println!("get_secret_status({})\n{}", test_secret_id, serde_json::to_string_pretty(&secret_status).unwrap());

  Ok(())
}
