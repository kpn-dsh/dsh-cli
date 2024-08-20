use trifonius_dsh_api::types::AppCatalogManifest;
use trifonius_dsh_api::DshApiClient;

#[tokio::main]
async fn main() -> Result<(), String> {
  let client = DshApiClient::default_client().await;

  let manifests: Vec<AppCatalogManifest> = client.get_app_catalog_manifests().await?;
  for manifest in manifests {
    println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
  }

  Ok(())
}
