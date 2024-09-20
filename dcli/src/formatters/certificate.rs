use dsh_api::types::{ActualCertificate, Certificate};

use crate::formatters::formatter::{Label, SubjectFormatter};

#[derive(Eq, Hash, PartialEq)]
pub enum CertificateLabel {
  CertChainSecret,
  DistinguishedName,
  DnsNames,
  KeySecret,
  NotAfter,
  NotBefore,
  PassphraseSecret,
  SerialNumber,
  Target,
}

impl Label for CertificateLabel {
  fn label_show(&self) -> &str {
    match self {
      CertificateLabel::CertChainSecret => "cert chain secret",
      CertificateLabel::DistinguishedName => "distinguished name",
      CertificateLabel::DnsNames => "dns names",
      CertificateLabel::KeySecret => "key secret",
      CertificateLabel::NotAfter => "not after",
      CertificateLabel::NotBefore => "not before",
      CertificateLabel::PassphraseSecret => "pass phrase secret",
      CertificateLabel::SerialNumber => "serial number",
      CertificateLabel::Target => "certificate id",
    }
  }
}

impl SubjectFormatter<CertificateLabel> for ActualCertificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> String {
    match label {
      CertificateLabel::CertChainSecret => self.cert_chain_secret.clone(),
      CertificateLabel::DistinguishedName => self.distinguished_name.clone(),
      CertificateLabel::DnsNames => self.dns_names.join("\n"),
      CertificateLabel::KeySecret => self.key_secret.clone(),
      CertificateLabel::NotAfter => self.not_after.to_string(),
      CertificateLabel::NotBefore => self.not_before.to_string(),
      CertificateLabel::PassphraseSecret => self.clone().passphrase_secret.unwrap_or_default(),
      CertificateLabel::SerialNumber => self.serial_number.clone(),
      CertificateLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<CertificateLabel> {
    Some(CertificateLabel::Target)
  }
}

impl SubjectFormatter<CertificateLabel> for Certificate {
  fn value(&self, label: &CertificateLabel, target_id: &str) -> String {
    match label {
      CertificateLabel::CertChainSecret => self.cert_chain_secret.clone(),
      CertificateLabel::KeySecret => self.key_secret.clone(),
      CertificateLabel::PassphraseSecret => self.clone().passphrase_secret.unwrap_or_default(),
      CertificateLabel::Target => target_id.to_string(),
      _ => unreachable!(),
    }
  }

  fn target_label(&self) -> Option<CertificateLabel> {
    Some(CertificateLabel::Target)
  }
}

pub static CERTIFICATE_CONFIGURATION_LABELS: [CertificateLabel; 4] =
  [CertificateLabel::Target, CertificateLabel::CertChainSecret, CertificateLabel::KeySecret, CertificateLabel::PassphraseSecret];

pub static ACTUAL_CERTIFICATE_LABELS_LIST: [CertificateLabel; 8] = [
  CertificateLabel::Target,
  CertificateLabel::CertChainSecret,
  CertificateLabel::KeySecret,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::PassphraseSecret,
  CertificateLabel::SerialNumber,
  CertificateLabel::DistinguishedName,
];

pub static ACTUAL_CERTIFICATE_LABELS_SHOW: [CertificateLabel; 9] = [
  CertificateLabel::Target,
  CertificateLabel::CertChainSecret,
  CertificateLabel::DnsNames,
  CertificateLabel::KeySecret,
  CertificateLabel::NotAfter,
  CertificateLabel::NotBefore,
  CertificateLabel::PassphraseSecret,
  CertificateLabel::SerialNumber,
  CertificateLabel::DistinguishedName,
];
