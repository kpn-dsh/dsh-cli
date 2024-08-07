use trifonius_dsh_api::types::AppCatalogManifest;
use trifonius_dsh_api::DEFAULT_DSH_API_CLIENT_FACTORY;

#[tokio::main]
async fn main() -> Result<(), String> {
  let client = &DEFAULT_DSH_API_CLIENT_FACTORY.client().await?;

  let manifests: Vec<AppCatalogManifest> = client.get_app_catalog_manifests().await?;
  for manifest in manifests {
    println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
  }

  Ok(())
}
