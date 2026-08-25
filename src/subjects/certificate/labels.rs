use crate::bundle::proxy::DshCertificate;
use crate::formatters::{hashmap_to_table, Value};
use crate::formatters::{Label, SubjectFormatter};
use crate::subjects::certificate::capabilities::ValidatedVhost;
use crate::subjects::certificate::get_relative_distinguished_name;
use dsh_api::types::{ActualCertificate, Certificate};
use itertools::Itertools;
use rcgen::{DistinguishedName, DnType, DnValue, OtherNameValue, SanType};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum CertificateLabel {
  CertChainSecret,
  CommonName,
  DistinguishedName,
  DnsNames,
  KeySecret,
  Kind,
  NotAfter,
  NotBefore,
  PassphraseSecret,
  SerialNumber,
  Target,
}

impl Label for CertificateLabel {
  fn as_str(&self) -> &str {
    match self {
      Self::CertChainSecret => "cert chain secret",
      Self::CommonName => "common name",
      Self::DistinguishedName => "distinguished name",
      Self::DnsNames => "dns names",
      Self::KeySecret => "key secret",
      Self::Kind => "kind",
      Self::NotAfter => "not after",
      Self::NotBefore => "not before",
      Self::PassphraseSecret => "pass phrase secret",
      Self::SerialNumber => "serial number",
      Self::Target => "certificate id",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Target)
  }
}

impl SubjectFormatter<CertificateLabel> for (&ActualCertificate, Option<u64>, Option<ValidatedVhost>) {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    let (actual_certificate, days, validated_vhost) = self;
    match label {
      CertificateLabel::CertChainSecret => Value::target(&actual_certificate.cert_chain_secret),
      CertificateLabel::CommonName => Value::some_or(get_relative_distinguished_name(&actual_certificate.distinguished_name, "CN"), Value::error("error")),
      CertificateLabel::DistinguishedName => Value::distinguished_name(&actual_certificate.distinguished_name),
      CertificateLabel::DnsNames => Value::plain(actual_certificate.dns_names.join("\n")),
      CertificateLabel::KeySecret => Value::target(&actual_certificate.key_secret),
      CertificateLabel::Kind => Value::some_or_hide(validated_vhost.as_ref().map(|(_, _, kafka, _)| if *kafka { "proxy" } else { "vhost" })),
      CertificateLabel::NotAfter => Value::datetime_expired(&actual_certificate.not_after, *days),
      CertificateLabel::NotBefore => Value::datetime_not_before(&actual_certificate.not_before),
      CertificateLabel::PassphraseSecret => match &actual_certificate.passphrase_secret {
        Some(passphrase_secret) => Value::target(passphrase_secret.clone()),
        None => Value::hide(),
      },
      CertificateLabel::SerialNumber => Value::plain(&actual_certificate.serial_number),
      CertificateLabel::Target => Value::target(target_id),
    }
  }
}

impl SubjectFormatter<CertificateLabel> for (ActualCertificate, Option<u64>, Option<ValidatedVhost>) {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    let (actual_certificate, days, validated_host) = self;
    (actual_certificate, *days, validated_host.clone()).value(label, target_id)
  }
}

impl SubjectFormatter<CertificateLabel> for Certificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    match label {
      CertificateLabel::CertChainSecret => Value::plain(&self.cert_chain_secret),
      CertificateLabel::KeySecret => Value::plain(&self.key_secret),
      CertificateLabel::PassphraseSecret => Value::some_or_empty(self.passphrase_secret.clone()),
      CertificateLabel::Target => Value::target(target_id),
      _ => Value::todo(),
    }
  }
}

impl SubjectFormatter<CertificateLabel> for DshCertificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> Value {
    match label {
      CertificateLabel::CertChainSecret => Value::unreachable(),
      CertificateLabel::CommonName => Value::some_or(
        get_rdn_from_distinguished_name(&self.certificate.params().distinguished_name, DnType::CommonName),
        Value::error("error"),
      ),
      CertificateLabel::DistinguishedName => Value::plain(hashmap_to_table(&hashmap_from_distinguished_name(&self.certificate.params().distinguished_name))),
      CertificateLabel::DnsNames => Value::plain(self.certificate.params().subject_alt_names.iter().map(san_to_string).collect_vec().join("\n")),
      CertificateLabel::KeySecret => Value::unreachable(),
      CertificateLabel::Kind => Value::unreachable(),
      CertificateLabel::NotAfter => Value::plain(self.certificate.params().not_after),
      CertificateLabel::NotBefore => Value::plain(self.certificate.params().not_before),
      CertificateLabel::PassphraseSecret => Value::unreachable(),
      CertificateLabel::SerialNumber => Value::some_or_hide(self.certificate.params().serial_number.as_ref().map(|serial_number| serial_number.to_string())),
      CertificateLabel::Target => Value::target(target_id),
    }
  }
}

fn san_to_string(san_type: &SanType) -> String {
  match san_type {
    SanType::Rfc822Name(rfc822) => format!("rfc822: {}", rfc822),
    SanType::DnsName(dns_name) => format!("dns: {}", dns_name),
    SanType::URI(uri) => format!("uri: {}", uri),
    SanType::IpAddress(ip_addr) => format!("ip address: {}", ip_addr),
    SanType::OtherName((_, OtherNameValue::Utf8String(utf8_string))) => format!("utf8: {}", utf8_string),
    _ => "".to_string(),
  }
}

pub(crate) fn get_rdn_from_distinguished_name(distinguished_name: &DistinguishedName, target_rdn_type: DnType) -> Option<String> {
  distinguished_name.iter().find_map(
    |(dn_type, dn_value)| {
      if *dn_type == target_rdn_type {
        Some(dn_value_string(dn_value))
      } else {
        None
      }
    },
  )
}

fn hashmap_from_distinguished_name(distinguished_name: &DistinguishedName) -> HashMap<String, String> {
  distinguished_name
    .iter()
    .map(|(dn_type, dn_value)| (dn_type_name(dn_type).to_string(), dn_value_string(dn_value)))
    .collect::<HashMap<_, _>>()
}

fn dn_value_string(dn_value: &DnValue) -> String {
  match dn_value {
    DnValue::Ia5String(ia5_string) => ia5_string.to_string(),
    DnValue::PrintableString(printable_string) => printable_string.to_string(),
    DnValue::TeletexString(teletex_string) => teletex_string.to_string(),
    DnValue::Utf8String(utf8_string) => utf8_string.to_string(),
    _ => "".to_string(),
  }
}

fn dn_type_name(dn_type: &DnType) -> &'static str {
  match dn_type {
    DnType::CountryName => "C",
    DnType::LocalityName => "L",
    DnType::StateOrProvinceName => "ST",
    DnType::OrganizationName => "O",
    DnType::OrganizationalUnitName => "OU",
    DnType::CommonName => "CN",
    DnType::CustomDnType(_) => "",
    _ => "",
  }
}
