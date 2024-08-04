use std::time::SystemTime;
use trifonius_dsh_api::DshApiClient;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();

  // let app_catalog_id: AppCatalogId = "eavesdropper".to_string();
  let app_catalog_id = "keyring-dev-proxy";
  // let body = AppCatalogAppConfiguration { configuration: Default::default(), manifest_urn: "".to_string(), name: "".to_string(), stopped: false };

  let client = DshApiClient::new();

  let start_time = SystemTime::now();

  let resp = client.get_apps().await?;
  let resp = client.get_app(&app_catalog_id).await?;
  // println!("{}", serde_json::to_string_pretty(&resp.configuration).unwrap());
  println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  // let resp: HashMap<String, AppCatalogApp> = client.app_catalog_get_by_tenant_appcatalogapp_configuration().await?;
  // println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  // let resp: Vec<AppCatalogManifest> = client.app_catalog_manifest_get_appcatalog_by_tenant_manifest().await?;
  // println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  // // let resp = client
  // //   .app_catalog_app_configuration_delete_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(app_catalog_id)
  // //   .await?;
  // // println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  // let resp = client
  //   .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(app_catalog_id)
  //   .await?;
  // println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  // let resp = client
  //   .app_catalog_app_configuration_get_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_status(app_catalog_id)
  //   .await?;
  // println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  // let resp = client
  //   .app_catalog_app_configuration_put_appcatalog_by_tenant_appcatalogapp_by_appcatalogappid_configuration(app_catalog_id, &body)
  //   .await?;
  // println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  // let resp = client.app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_actual(app_catalog_id).await?;
  // println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  // let resp = client
  //   .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_configuration(app_catalog_id)
  //   .await?;
  // println!("{}", serde_json::to_string_pretty(&resp).unwrap());

  println!("{:?}", SystemTime::now().duration_since(start_time));

  Ok(())
}
