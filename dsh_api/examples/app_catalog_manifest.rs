use serde_json::de::from_str;
use serde_json::Value;
use trifonius_dsh_api::types::AppCatalogManifest;
use trifonius_dsh_api::DshApiClient;

#[tokio::main]
async fn main() -> Result<(), String> {
  let client = DshApiClient::default_client().await;

  let manifests: Vec<AppCatalogManifest> = client.get_app_catalog_manifests().await?;

  for (index, manifest) in manifests.iter().enumerate() {
    let payload = &manifest.payload;
    let des = from_str::<Value>(payload.as_str()).unwrap();
    let object = des.as_object().unwrap();

    println!("--------------------------------------- {}", index);
    println!("api version    {}", object.get(API_VERSION).unwrap());
    // println!("configuration  {}", object.get(CONFIGURATION).unwrap());
    // println!("contact        {}", object.get(CONTACT).unwrap());
    // println!("description    {}", object.get(DESCRIPTION).unwrap());
    println!("id             {}", object.get(ID).unwrap().as_str().unwrap());
    println!("kind           {}", object.get(KIND).unwrap());
    // println!("more info      {}", object.get(MORE_INFO).unwrap());
    println!("name           {}", object.get(NAME).unwrap());
    // println!("resources      {}", object.get(RESOURCES).unwrap());
    println!("vendor         {}", object.get(VENDOR).unwrap());
    println!("version        {}", object.get(VERSION).unwrap());
  }

  Ok(())
}

const API_VERSION: &str = "apiVersion";
const CONFIGURATION: &str = "configuration";
const CONTACT: &str = "contact";
const DESCRIPTION: &str = "description";
const ID: &str = "id";
const KIND: &str = "kind";
const MORE_INFO: &str = "moreInfo";
const NAME: &str = "name";
const RESOURCES: &str = "resources";
const VENDOR: &str = "vendor";
const VERSION: &str = "version";

// kpn/aep-sink-connect      0.1.0
// kpn/aep-sink-connect      0.1.1
//
// kpn/airflow-ephemeral      0.9.0
//
// kpn/airflow-persistent      1.0.1
//
// kpn/cmdline      1.1.3
// kpn/cmdline      1.1.5
// kpn/cmdline      1.1.6
//
// kpn/dsh-database-ingester      0.1.2
// kpn/dsh-database-ingester      0.3.0
// kpn/dsh-database-ingester      0.4.0
// kpn/dsh-database-ingester      0.4.2
//
// kpn/eavesdropper      0.7.1
// kpn/eavesdropper      0.8.0
// kpn/eavesdropper      0.8.1
// kpn/eavesdropper      0.9.1
// kpn/eavesdropper      0.9.2
//
// kpn/http-source-connector      0.5.0
// kpn/http-source-connector      0.5.2
// kpn/http-source-connector      0.6.0
//
// kpn/kafdrop      4.0.1
//
// kpn/kafka-data-archiver      1.0.0
// kpn/kafka-data-archiver      1.3.0
//
// kpn/kafka2kafka      1.0.0
//
// kpn/keyring-kafka-database-extractor      0.2.4
// kpn/keyring-kafka-database-extractor      0.3.0
// kpn/keyring-kafka-database-extractor      0.4.0
// kpn/keyring-kafka-database-extractor      0.4.1
// kpn/keyring-kafka-database-extractor      0.4.2
// kpn/keyring-kafka-database-extractor      0.4.4
//
// kpn/keyring-service      0.4.1
// kpn/keyring-service      0.4.2
// kpn/keyring-service      0.4.3
// kpn/keyring-service      0.5.0
// kpn/keyring-service      0.5.1
//
// kpn/metrics-proxy      0.1.1
// kpn/metrics-proxy      0.1.2
//
// kpn/schema-store-ui      0.0.10-beta
// kpn/schema-store-ui      0.0.11-beta
// kpn/schema-store-ui      0.0.12
// kpn/schema-store-ui      0.0.13
// kpn/schema-store-ui      0.0.14
//
// kpn/secor      0.30.2
// kpn/secor      0.30.3
//
// kpn/sql-database      1.1.2
// kpn/sql-database      1.1.3
//
// kpn/sql-database-viewer      1.1.2
//
// kpn/whoami      0.0.1
// kpn/whoami      0.0.3
//
// kpn/zookeeper-proxy      1.2.1
// kpn/zookeeper-proxy      1.2.2
