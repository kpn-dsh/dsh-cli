use dsh_api::dsh_api_client_factory::DshApiClientFactory;
use dsh_api::types::{AllocationStatus, AppCatalogAppConfiguration};

#[tokio::main]
async fn main() -> Result<(), String> {
  let app_catalog_id = "keyring-dev-proxy";

  let client_factory = DshApiClientFactory::default();
  let client = client_factory.client().await?;

  // let deleted: () = client.delete_app_catalog_app(app_catalog_id).await?;
  // println!("{}", serde_json::to_string_pretty(&deleted).unwrap());

  // let deployed: () = client.deploy_app_catalog_app(app_catalog_id, body).await?;
  // println!("{}", serde_json::to_string_pretty(&deployed).unwrap());

  let configuration: AppCatalogAppConfiguration = client.get_app_catalog_app_configuration(app_catalog_id).await?;
  println!("{}", serde_json::to_string_pretty(&configuration).unwrap());

  let status: AllocationStatus = client.get_app_catalog_app_allocation_status(app_catalog_id).await?;
  println!("{}", serde_json::to_string_pretty(&status).unwrap());

  Ok(())
}
