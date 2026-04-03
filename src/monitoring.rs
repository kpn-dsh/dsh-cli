use dsh_api::platform::DshPlatform;
use percent_encoding::{utf8_percent_encode, AsciiSet, PercentEncode, CONTROLS};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{BuildHasher, Hasher, RandomState};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn tenant_service_monitoring_url(platform: &DshPlatform, tenant_name: impl Display, service_id: impl Display, task_id: impl Display) -> String {
  format!(
    "{}/explore?{}",
    platform.tenant_monitoring_url(tenant_name),
    MonitoringUrlParameters::new(service_id, task_id)
  )
}

struct MonitoringUrlParameters {
  schema_version: u32,
  panes: HashMap<String, MonitoringPane>,
  org_id: u32,
}

fn random_pane_id() -> String {
  const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz123456";
  let mut random = RandomState::new().build_hasher().finish() as usize;
  (0..3)
    .map(|_| {
      let idx = random & 0x001f;
      random = random >> 5;
      char::from(CHARSET[idx])
    })
    .collect()
}

impl MonitoringUrlParameters {
  fn new(service_id: impl Display, containerd: impl Display) -> Self {
    Self { schema_version: 1, panes: HashMap::from([(random_pane_id(), MonitoringPane::new(service_id, containerd))]), org_id: 1 }
  }
}

impl Display for MonitoringUrlParameters {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    log::trace!("{}", serde_json::to_string_pretty(&self.panes).unwrap());
    let panes = serde_json::to_string(&self.panes).unwrap();
    write!(f, "schemaVersion={}&panes={}&orgId={}", self.schema_version, encode_path(&panes), self.org_id)
  }
}

#[derive(Serialize)]
struct MonitoringPane {
  #[serde(rename = "datasource")]
  data_source: String,
  queries: Vec<MonitoringQuery>,
  range: MonitoringRange,
}

impl MonitoringPane {
  fn new(service_id: impl Display, containerd: impl Display) -> Self {
    Self { data_source: "loki".to_string(), queries: vec![MonitoringQuery::new(service_id, containerd)], range: MonitoringRange::default() }
  }
}

#[derive(Serialize)]
struct MonitoringQuery {
  #[serde(rename = "refId")]
  ref_id: String,
  #[serde(rename = "editorMode")]
  editor_mode: String,
  expr: String,
  #[serde(rename = "queryType")]
  query_type: String,
  datasource: MonitoringDataSource,
  direction: String,
}

impl MonitoringQuery {
  fn new(service_id: impl Display, containerd: impl Display) -> Self {
    Self {
      ref_id: "A".to_string(),
      editor_mode: "builder".to_string(),
      expr: expr(service_id, containerd),
      query_type: "range".to_string(),
      datasource: Default::default(),
      direction: "forward".to_string(),
    }
  }
}

#[derive(Serialize)]
struct MonitoringDataSource {
  #[serde(rename = "type")]
  kind: String,
  uid: String,
}

impl Default for MonitoringDataSource {
  fn default() -> Self {
    Self { kind: "loki".to_string(), uid: "loki".to_string() }
  }
}

#[derive(Serialize)]
struct MonitoringRange {
  from: String,
  to: String,
}

impl Default for MonitoringRange {
  fn default() -> Self {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok().unwrap().as_millis();
    Self::new(now - 60 * 60 * 1000, now)
  }
}
impl MonitoringRange {
  fn new(from: u128, to: u128) -> Self {
    Self { from: from.to_string(), to: to.to_string() }
  }
}

fn expr(service_id: impl Display, containerd: impl Display) -> String {
  format!(
    "{{app=\"{}\"}}+|+pod_container_id+=+\"{}\"+or+container_id+=+\"{}\"",
    service_id, containerd, containerd
  )
}

fn encode_path(pc: &str) -> PercentEncode {
  {
    const PATH_SET: &AsciiSet = &CONTROLS
      .add(b' ')
      .add(b':')
      .add(b',')
      .add(b'"')
      .add(b'|')
      .add(b'#')
      .add(b'<')
      .add(b'>')
      .add(b'[')
      .add(b']')
      .add(b'`')
      .add(b'{')
      .add(b'}')
      .add(b'/')
      .add(b'=')
      .add(b'\\')
      .add(b'%');
    utf8_percent_encode(pc, PATH_SET)
  }
}
