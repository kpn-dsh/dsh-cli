use crate::bundle::DshCertificate;
use crate::formatters::{hashmap_to_table, Value};
use crate::formatters::{Label, SubjectFormatter};
use crate::subjects::certificate::capabilities::ValidatedVhost;
use dsh_api::types::{ActualCertificate, Certificate};
use itertools::Itertools;
use rcgen::{DistinguishedName, DnType, DnValue, OtherNameValue, SanType};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum CertificateLabel {
  CertChainSecret,
  DistinguishedName,
  DnsNames,
  DnsNamesSummary,
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
      Self::DistinguishedName => "distinguished name",
      Self::DnsNames => "dns names",
      Self::DnsNamesSummary => "dns names",
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
      CertificateLabel::DistinguishedName => Value::distinguished_name(&actual_certificate.distinguished_name),
      CertificateLabel::DnsNames => Value::plain(actual_certificate.dns_names.join("\n")),
      CertificateLabel::DnsNamesSummary => Value::plain(summarize_dns_names(&actual_certificate.dns_names)),
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

fn summarize_dns_names(dns_names: &[String]) -> String {
  if dns_names.len() <= 4 {
    dns_names.join("\n")
  } else {
    if dns_names.last().is_some_and(|last_dns| last_dns.contains("-schema-store.")) {
      let mut summary = String::new();
      summary.push_str(dns_names.first().unwrap_or_else(|| unreachable!()));
      summary.push('\n');
      summary.push_str("...");
      summary.push('\n');
      summary.push_str(dns_names.get(dns_names.len() - 2).unwrap_or_else(|| unreachable!()));
      summary.push('\n');
      summary.push_str(dns_names.last().unwrap_or_else(|| unreachable!()));
      summary
    } else {
      let mut summary = String::new();
      summary.push_str(dns_names.first().unwrap_or_else(|| unreachable!()));
      summary.push('\n');
      summary.push_str("...");
      summary.push('\n');
      summary.push_str(dns_names.last().unwrap_or_else(|| unreachable!()));
      summary
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
      CertificateLabel::DistinguishedName => Value::plain(hashmap_to_table(&hashmap_from_distinguished_name(&self.certificate.params().distinguished_name))),
      CertificateLabel::DnsNames => Value::plain(self.certificate.params().subject_alt_names.iter().map(san_to_string).collect_vec().join("\n")),
      CertificateLabel::DnsNamesSummary => Value::plain(summarize_dns_names(
        &self.certificate.params().subject_alt_names.iter().map(san_to_string).collect_vec(),
      )),
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
