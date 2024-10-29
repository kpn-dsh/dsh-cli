use chrono::{TimeZone, Utc};
use serde_json::de::from_str;
use serde_json::Value;

use dsh_api::app_catalog_manifest::CONTACT;
use dsh_api::types::AppCatalogManifest;

use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) enum ManifestLabel {
  Configuration,
  Contact,
  Draft,
  LastModified,
  Name,
  Vendor,
  Version,
  Target,
}

const CONFIGURATION: &str = "configuration";
const ID: &str = "id";
const NAME: &str = "name";
const VENDOR: &str = "vendor";
const VERSION: &str = "version";

impl Label for ManifestLabel {
  fn label_for_show(&self) -> &str {
    match self {
      Self::Configuration => CONFIGURATION,
      Self::Target => "app",
      Self::Contact => "contact",
      Self::Draft => "draft",
      Self::LastModified => "last modified",
      Self::Name => NAME,
      Self::Vendor => VENDOR,
      Self::Version => VERSION,
    }
  }

  fn is_target_label(&self) -> bool {
    *self == Self::Target
  }
}

#[derive(Debug)]
pub struct Manifest {
  pub manifest_id: String,
  pub contact: String,
  pub draft: bool,
  pub last_modified: String,
  pub name: String,
  pub vendor: String,
  pub version: String,
}

impl TryFrom<&AppCatalogManifest> for Manifest {
  type Error = String;

  fn try_from(value: &AppCatalogManifest) -> Result<Self, Self::Error> {
    match from_str::<Value>(value.payload.as_str()) {
      Ok(payload_value) => match payload_value.as_object() {
        Some(payload_object) => Ok(Manifest {
          manifest_id: payload_object.get(&ID.to_string()).unwrap().as_str().unwrap().to_string(),
          contact: payload_object.get(&CONTACT.to_string()).unwrap().as_str().unwrap().to_string(),
          draft: value.draft,
          last_modified: Utc.timestamp_opt(value.last_modified as i64 / 1000, 0).unwrap().to_string(),
          name: payload_object.get(&NAME.to_string()).unwrap().as_str().unwrap().to_string(),
          vendor: payload_object.get(&VENDOR.to_string()).unwrap().as_str().unwrap().to_string(),
          version: payload_object.get(&VERSION.to_string()).unwrap().as_str().unwrap().to_string(),
        }),
        None => Err("".to_string()),
      },
      Err(_) => Err("".to_string()),
    }
  }
}

impl SubjectFormatter<ManifestLabel> for Manifest {
  fn value(&self, label: &ManifestLabel, target_id: &str) -> String {
    match label {
      ManifestLabel::Configuration => "".to_string(),
      ManifestLabel::Contact => self.contact.clone(),
      ManifestLabel::Draft => self.draft.to_string(),
      ManifestLabel::LastModified => self.last_modified.clone(),
      ManifestLabel::Name => self.name.clone(),
      ManifestLabel::Vendor => self.vendor.clone(),
      ManifestLabel::Version => self.version.clone(),
      ManifestLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<ManifestLabel> {
    Some(ManifestLabel::Target)
  }
}

pub static MANIFEST_LABELS_LIST: [ManifestLabel; 6] =
  [ManifestLabel::Target, ManifestLabel::Version, ManifestLabel::Name, ManifestLabel::Draft, ManifestLabel::Vendor, ManifestLabel::LastModified];

pub static MANIFEST_LABELS_SHOW: [ManifestLabel; 8] = [
  ManifestLabel::Target,
  ManifestLabel::Configuration,
  ManifestLabel::Contact,
  ManifestLabel::Draft,
  ManifestLabel::LastModified,
  ManifestLabel::Name,
  ManifestLabel::Vendor,
  ManifestLabel::Version,
];

//   {
//     "apiVersion": String("v0-alpha"),
//     "configuration": Object {
//         "$schema": String("https://json-schema.org/draft/2019-09/schema"),
//         "properties": Object {
//             "ALLOW_CUSTOM_GROUP_ID": Object {
//                 "default": String("true"),
//                 "description": String("allow custom group id"),
//                 "enum": Array [
//                     String("true"),
//                     String("false"),
//                 ],
//                 "type": String("string"),
//             },
//             "ENVELOPE_DESERIALIZERS": Object {
//                 "default": String(""),
//                 "description": String("enable envelope deserializers"),
//                 "enum": Array [
//                     String(""),
//                     String("dsh-envelope"),
//                     String("gzip"),
//                     String("dsh-envelope,gzip"),
//                 ],
//                 "type": String("string"),
//             },
//             "LOG_LEVEL": Object {
//                 "default": String("info"),
//                 "description": String("application log level"),
//                 "enum": Array [
//                     String("error"),
//                     String("warn"),
//                     String("info"),
//                 ],
//                 "type": String("string"),
//             },
//             "REPRESENTATION_DESERIALIZERS": Object {
//                 "default": String(""),
//                 "description": String("enable representation deserializers"),
//                 "enum": Array [
//                     String(""),
//                     String("codomain-values-record"),
//                     String("greenbox-ri-avro,greenbox-ri-protobuf"),
//                     String("schema-store-deserializer"),
//                     String("codomain-values-record,greenbox-ri-avro,greenbox-ri-protobuf,schema-store-deserializer"),
//                 ],
//                 "type": String("string"),
//             },
//         },
//         "type": String("object"),
//     },
//     "contact": String("wilbert.schelvis@kpn.com"),
//     "description": String("Web application to visualize messages on Kafka topics in realtime"),
//     "id": String("kpn/eavesdropper"),
//     "kind": String("manifest"),
//     "moreInfo": String("## Realtime record visualization\n\nThis app enables you to view records on your DSH Kafka topics in realtime. It can show the record's keys, values and headers in json, text or binary format, and also recognizes some custom formats for some special topics. Furthermore, it allows you to:\n\n* unwrap a record from the envelope that the DSH platform enforces on stream topics, showing the envelope metadata and the envelope payload separately,\n* filter records based on regular expressions and/or throttling,\n* show records individually or in list view,\n* download/copy record values in json, text or binary format.\n\nWhen started from the App Catalog the Eavesdropper will be available with the same SSO authorization as the DSH console, and will expose all topics that your tenant is entitled to see."),
//     "name": String("eavesdropper"),
//     "resources": Object {
//         "allocation/${@tenant}/application/${@name}": Object {
//             "cpus": Number(0.1),
//             "env": Object {
//                 "ALLOW_CUSTOM_GROUP_ID": String("${ALLOW_CUSTOM_GROUP_ID}"),
//                 "CONSUMER_BUILDER": String("dsh-datastreams-properties"),
//                 "DEFAULT_GROUP_IDS": String("*"),
//                 "ENVELOPE_DESERIALIZERS": String("${ENVELOPE_DESERIALIZERS}"),
//                 "EXCLUDED_TOPICS": String(""),
//                 "EXCLUDED_TOPICS_REGEX": String(""),
//                 "GROUP_ID_PREFIX": String("${@tenant}_"),
//                 "INCLUDED_TOPICS": String(""),
//                 "INCLUDED_TOPICS_REGEX": String(""),
//                 "INCLUDE___CONSUMER_OFFSETS_TOPIC": String("false"),
//                 "INSTANCE_IDENTIFIER": String("${@tenant}_${@name}"),
//                 "LOG_LEVEL": String("${LOG_LEVEL}"),
//                 "LOG_LEVEL_ENTRYPOINT": String("${LOG_LEVEL}"),
//                 "LOG_LEVEL_GREENBOX": String("error"),
//                 "LOG_LEVEL_KAFKA_CLIENT": String("error"),
//                 "LOG_LEVEL_MONITOR": String("${LOG_LEVEL}"),
//                 "LOG_LEVEL_RECDES": String("info"),
//                 "LOG_LEVEL_SERVICE": String("${LOG_LEVEL}"),
//                 "REPRESENTATION_DESERIALIZERS": String("${REPRESENTATION_DESERIALIZERS}"),
//             },
//             "exposedPorts": Object {
//                 "8081": Object {
//                     "auth": String("system-fwd-auth@view,manage"),
//                     "tls": String("auto"),
//                     "vhost": String("{ vhost('${@name}.${@tenant}', 'public') }"),
//                 },
//             },
//             "image": String("${@appcatalog}/draft/kpn/eavesdropper:0.9.3"),
//             "instances": Number(1),
//             "mem": Number(384),
//             "name": String("${@name}"),
//             "needsToken": Bool(true),
//             "singleInstance": Bool(false),
//             "user": String("${@uid}:${@gid}"),
//         },
//         "allocation/${@tenant}/vhost/${@name}.${@tenant}@public": String("${@name}.${@tenant}@public"),
//     },
//     "vendor": String("KPN"),
//     "version": String("0.9.3"),
// }
