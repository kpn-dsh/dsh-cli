use crate::formatters::{Label, SubjectFormatter, Value};
use crate::subjects::platform::capabilities::ProvidedArguments;
use dsh_api::platform::{DshPlatform, VhostZone};
use serde::Serialize;

#[derive(Clone, Eq, Hash, PartialEq, Serialize, Debug)]
pub(crate) enum DshPlatformLabel {
  // From configuration
  Alias,
  CloudProvider,
  Description,
  IsProduction,
  IssuerEndpoint,
  Parameter,
  PrivateDomain,
  PublicDomain,
  Realm,
  Region,
  // Derived from configuration
  AccessTokenEndpoint,
  ConsoleDomain,
  ConsoleUrl,
  MqttMessagingApiEndpoint,
  MqttMessagingApiPort,
  MqttTokenEndpoint,
  RestApiDomain,
  RestApiEndpoint,
  RestTokenEndpoint,
  RobotClientId,
  SwaggerUrl,
  TracingUrl,
  // Derived from configuration and tenant
  InternalDomain,
  ProxyVhostDomain,
  RobotTenantClientId,
  TenantAppCatalogUrl,
  TenantConsoleUrl,
  TenantDataCatalogUrl,
  TenantMonitoringUrl,
  TenantPublicAppsDomain,
  // Derived from configuration, tenant and arguments
  BucketName,
  HttpMessagingApiUrlMulti,
  HttpMessagingApiUrlSingle,
  InternalServiceDomain,
  ProxyBrokerVhost,
  ProxyCommonName,
  ProxyConsumerGroup,
  ProxyConsumerGroupAcl,
  ProxySchemaStoreVhost,
  PublicVhostDomain,
  TenantAppCatalogAppUrl,
  TenantAppConsoleUrl,
  TenantPrivateAppDomain,
  TenantPrivateVhostDomain,
  TenantProxyPrivateBootstrapServers,
  TenantProxyPrivateSchemaStoreHost,
  TenantProxyPublicBootstrapServers,
  TenantProxyPublicSchemaStoreHost,
  TenantPublicAppDomain,
  TenantServiceConsoleUrl,
}

impl Label for DshPlatformLabel {
  fn as_str(&self) -> &str {
    match self {
      // From configuration
      Self::Alias => "alias",
      Self::CloudProvider => "cloud provider",
      Self::Description => "description",
      Self::IsProduction => "production",
      Self::IssuerEndpoint => "issuer endpoint",
      Self::Parameter => "parameter",
      Self::PrivateDomain => "private domain",
      Self::PublicDomain => "public domain",
      Self::Realm => "realm",
      Self::Region => "region",
      // Derived from configuration
      Self::AccessTokenEndpoint => "access token endpoint",
      Self::ConsoleDomain => "console domain",
      Self::ConsoleUrl => "console url",
      Self::MqttMessagingApiEndpoint => "mqtt messaging api endpoint",
      Self::MqttMessagingApiPort => "mqtt messaging api port",
      Self::MqttTokenEndpoint => "mqtt token endpoint",
      Self::RestApiDomain => "rest api domain",
      Self::RestApiEndpoint => "rest api endpoint",
      Self::RestTokenEndpoint => "rest token endpoint",
      Self::RobotClientId => "robot client id",
      Self::SwaggerUrl => "swagger url",
      Self::TracingUrl => "tracing url",
      // Derived from configuration and tenant
      Self::InternalDomain => "internal domain",
      Self::ProxyVhostDomain => "proxy vhost domain",
      Self::RobotTenantClientId => "robot api client id",
      Self::TenantAppCatalogUrl => "app catalog url",
      Self::TenantConsoleUrl => "console url",
      Self::TenantDataCatalogUrl => "data catalog url",
      Self::TenantMonitoringUrl => "monitoring url",
      Self::TenantPublicAppsDomain => "public apps domain",
      // Derived from configuration, tenant and arguments
      Self::BucketName => "bucket name",
      Self::HttpMessagingApiUrlMulti => "http messaging api url (multi)",
      Self::HttpMessagingApiUrlSingle => "http messaging api url (single)",
      Self::InternalServiceDomain => "internal service domain",
      Self::ProxyBrokerVhost => "proxy broker vhost",
      Self::ProxyCommonName => "proxy common name",
      Self::ProxyConsumerGroup => "proxy consumer group",
      Self::ProxyConsumerGroupAcl => "proxy consumer group acl",
      Self::ProxySchemaStoreVhost => "proxy schema store vhost",
      Self::PublicVhostDomain => "public vhost domain",
      Self::TenantAppCatalogAppUrl => "app catalog url",
      Self::TenantAppConsoleUrl => "app console url",
      Self::TenantPrivateAppDomain => "private app domain",
      Self::TenantPrivateVhostDomain => "private vhost domain",
      Self::TenantProxyPrivateBootstrapServers => "proxy private bootstrap server",
      Self::TenantProxyPrivateSchemaStoreHost => "proxy private schema store host",
      Self::TenantProxyPublicBootstrapServers => "proxy public bootstrap server",
      Self::TenantProxyPublicSchemaStoreHost => "proxy public schema store host",
      Self::TenantPublicAppDomain => "public app domain",
      Self::TenantServiceConsoleUrl => "service console url",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Parameter)
  }
}

// Subject formatter for DshPlatform only
impl SubjectFormatter<DshPlatformLabel> for DshPlatform {
  fn value(&self, label: &DshPlatformLabel, _target_id: &str) -> Value {
    match label {
      // From configuration
      DshPlatformLabel::Alias => Value::target(self.alias()),
      DshPlatformLabel::CloudProvider => Value::plain(self.cloud_provider()),
      DshPlatformLabel::Description => Value::plain(self.description()),
      DshPlatformLabel::IsProduction => Value::plain(self.is_production()),
      DshPlatformLabel::IssuerEndpoint => Value::plain(self.issuer_endpoint()),
      DshPlatformLabel::Parameter => Value::target(self.name()),
      DshPlatformLabel::PrivateDomain => Value::some_or(self.private_domain(), Value::ignore("not supported")),
      DshPlatformLabel::PublicDomain => Value::plain(self.public_domain()),
      DshPlatformLabel::Realm => Value::plain(self.realm()),
      DshPlatformLabel::Region => Value::some_or(self.region(), Value::ignore("not configured")),
      // Derived from configuration
      DshPlatformLabel::AccessTokenEndpoint => Value::plain(self.access_token_endpoint()),
      DshPlatformLabel::ConsoleDomain => Value::plain(self.console_domain()),
      DshPlatformLabel::ConsoleUrl => Value::plain(self.console_url()),
      DshPlatformLabel::MqttMessagingApiEndpoint => Value::plain(self.mqtt_messaging_api_endpoint()),
      DshPlatformLabel::MqttMessagingApiPort => Value::plain(self.mqtt_messaging_api_port().to_string()),
      DshPlatformLabel::MqttTokenEndpoint => Value::plain(self.mqtt_token_endpoint()),
      DshPlatformLabel::RestApiDomain => Value::plain(self.rest_api_domain()),
      DshPlatformLabel::RestApiEndpoint => Value::plain(self.rest_api_endpoint()),
      DshPlatformLabel::RestTokenEndpoint => Value::plain(self.rest_token_endpoint()),
      DshPlatformLabel::RobotClientId => Value::plain(self.robot_client_id()),
      DshPlatformLabel::SwaggerUrl => Value::plain(self.swagger_url()),
      DshPlatformLabel::TracingUrl => Value::plain(self.tracing_url()),
      _ => unreachable!(),
    }
  }
}

// Subject formatter for (DshPlatform, Tenant) tuple
impl SubjectFormatter<DshPlatformLabel> for (&DshPlatform, &String) {
  fn value(&self, label: &DshPlatformLabel, target_id: &str) -> Value {
    let (platform, tenant) = self;
    match label {
      // Derived from configuration and tenant
      DshPlatformLabel::InternalDomain => Value::plain(platform.internal_domain(tenant)),
      DshPlatformLabel::ProxyVhostDomain => Value::ok_or_empty(platform.proxy_vhost_domain(tenant, VhostZone::Public)),
      DshPlatformLabel::RobotTenantClientId => Value::plain(platform.robot_tenant_client_id(tenant)),
      DshPlatformLabel::TenantAppCatalogUrl => Value::plain(platform.tenant_app_catalog_url(tenant)),
      DshPlatformLabel::TenantConsoleUrl => Value::plain(platform.tenant_console_url(tenant)),
      DshPlatformLabel::TenantDataCatalogUrl => Value::plain(platform.tenant_data_catalog_url(tenant)),
      DshPlatformLabel::TenantMonitoringUrl => Value::plain(platform.tenant_monitoring_url(tenant)),
      DshPlatformLabel::TenantPublicAppsDomain => Value::ok_or_empty(platform.tenant_domain(tenant, VhostZone::Public)),
      _ => platform.value(label, target_id),
    }
  }
}

// Subject formatter for (DshPlatform, Tenant, ProvidedArguments) tuple
impl SubjectFormatter<DshPlatformLabel> for (DshPlatform, &String, &ProvidedArguments) {
  fn value(&self, label: &DshPlatformLabel, target_id: &str) -> Value {
    let (platform, tenant, provided_arguments) = self;
    let app_id = provided_arguments.app_id.clone().unwrap_or_default();
    let bucket_id = provided_arguments.bucket_id.clone().unwrap_or_default();
    let proxy_name = provided_arguments.proxy_id.clone().unwrap_or_default();
    let service_id = provided_arguments.service_id.clone().unwrap_or_default();
    let topic = provided_arguments.topic_id.clone().unwrap_or_default();
    let vendor_id = provided_arguments.vendor_id.clone().unwrap_or_default();
    let vhost = provided_arguments.vhost.clone().unwrap_or_default();
    match label {
      // Derived from configuration, tenant and arguments
      DshPlatformLabel::BucketName => Value::plain(
        platform
          .bucket_name(tenant, bucket_id, Some("ACCESS_KEY_ID"))
          .unwrap_or_else(|error| error.to_string()),
      ),
      DshPlatformLabel::HttpMessagingApiUrlMulti => Value::plain(platform.http_messaging_api_url_multi(topic)),
      DshPlatformLabel::HttpMessagingApiUrlSingle => Value::plain(platform.http_messaging_api_url_single(topic)),
      DshPlatformLabel::InternalServiceDomain => Value::plain(platform.internal_service_domain(tenant, service_id)),
      DshPlatformLabel::ProxyBrokerVhost => Value::ok_or_empty(platform.proxy_vhost_index(tenant, proxy_name, VhostZone::Public, 0)),
      DshPlatformLabel::ProxyCommonName => Value::ok_or_hide(platform.proxy_common_name(proxy_name, tenant, VhostZone::Public)),
      DshPlatformLabel::ProxyConsumerGroup => Value::plain(platform.proxy_consumer_group(tenant, proxy_name, 0)),
      DshPlatformLabel::ProxyConsumerGroupAcl => Value::plain(platform.proxy_consumer_group_acl(tenant, proxy_name, "acl-group-name", 0)),
      DshPlatformLabel::ProxySchemaStoreVhost => Value::ok_or_empty(platform.proxy_schema_store_vhost(tenant, proxy_name, VhostZone::Public)),
      DshPlatformLabel::PublicVhostDomain => Value::plain(platform.public_vhost_domain(vhost)),
      DshPlatformLabel::TenantAppCatalogAppUrl => Value::plain(platform.tenant_app_catalog_app_url(tenant, vendor_id, app_id)),
      DshPlatformLabel::TenantAppConsoleUrl => Value::plain(platform.tenant_app_console_url(tenant, app_id)),
      DshPlatformLabel::TenantProxyPrivateBootstrapServers => Value::ok_or_hide(platform.tenant_proxy_bootstrap_server(tenant, proxy_name, VhostZone::Private, None, 0)),
      DshPlatformLabel::TenantProxyPrivateSchemaStoreHost => Value::some_or_hide(platform.tenant_proxy_schema_store_host(tenant, proxy_name, VhostZone::Private).ok()),
      DshPlatformLabel::TenantProxyPublicBootstrapServers => Value::ok_or_empty(platform.tenant_proxy_bootstrap_server(tenant, proxy_name, VhostZone::Public, None, 0)),
      DshPlatformLabel::TenantProxyPublicSchemaStoreHost => Value::ok_or_empty(platform.tenant_proxy_schema_store_host(tenant, proxy_name, VhostZone::Public)),
      DshPlatformLabel::TenantPrivateAppDomain => Value::ok_or_hide(platform.tenant_private_app_domain(tenant, app_id)),
      DshPlatformLabel::TenantPrivateVhostDomain => Value::ok_or_hide(platform.tenant_private_vhost_domain(tenant, vhost)),
      DshPlatformLabel::TenantPublicAppDomain => Value::plain(platform.tenant_public_app_domain(tenant, app_id)),
      DshPlatformLabel::TenantServiceConsoleUrl => Value::plain(platform.tenant_service_console_url(tenant, service_id)),
      _ => (platform, *tenant).value(label, target_id),
    }
  }
}

/// Defines the parameters that are required for a `Label` variant.
///
/// * `app_id_required`
/// * `bucket_id_required`
/// * `proxy_id_required`
/// * `service_id_required`
/// * `tenant_required`
/// * `topic_required`
/// * `vendor_id_required`
/// * `vhost_required`
struct RequiredArguments {
  app_id_required: bool,
  bucket_id_required: bool,
  proxy_id_required: bool,
  service_id_required: bool,
  topic_required: bool,
  vendor_id_required: bool,
  vhost_required: bool,
}

impl DshPlatformLabel {
  fn required_arguments(&self) -> RequiredArguments {
    match self {
      DshPlatformLabel::BucketName => REQUIRED_ARGUMENTS_BUCKET,
      DshPlatformLabel::TenantAppCatalogAppUrl => REQUIRED_ARGUMENTS_APP_VENDOR,
      DshPlatformLabel::TenantAppConsoleUrl | DshPlatformLabel::TenantPrivateAppDomain | DshPlatformLabel::TenantPublicAppDomain => REQUIRED_ARGUMENTS_APP,
      DshPlatformLabel::TenantServiceConsoleUrl | DshPlatformLabel::InternalServiceDomain => REQUIRED_ARGUMENTS_SERVICE,
      DshPlatformLabel::InternalDomain
      | DshPlatformLabel::RobotTenantClientId
      | DshPlatformLabel::TenantAppCatalogUrl
      | DshPlatformLabel::TenantConsoleUrl
      | DshPlatformLabel::TenantDataCatalogUrl
      | DshPlatformLabel::TenantMonitoringUrl
      | DshPlatformLabel::TenantPublicAppsDomain
      | DshPlatformLabel::ProxyVhostDomain => REQUIRED_ARGUMENTS_TENANT,
      DshPlatformLabel::HttpMessagingApiUrlMulti | DshPlatformLabel::HttpMessagingApiUrlSingle => REQUIRED_ARGUMENTS_TOPIC,
      DshPlatformLabel::TenantPrivateVhostDomain | DshPlatformLabel::PublicVhostDomain => REQUIRED_ARGUMENTS_VHOST,
      DshPlatformLabel::ProxyBrokerVhost
      | DshPlatformLabel::ProxyCommonName
      | DshPlatformLabel::ProxyConsumerGroup
      | DshPlatformLabel::ProxyConsumerGroupAcl
      | DshPlatformLabel::ProxySchemaStoreVhost
      | DshPlatformLabel::TenantProxyPrivateBootstrapServers
      | DshPlatformLabel::TenantProxyPrivateSchemaStoreHost
      | DshPlatformLabel::TenantProxyPublicBootstrapServers
      | DshPlatformLabel::TenantProxyPublicSchemaStoreHost => REQUIRED_ARGUMENTS_PROXY_TENANT,
      _ => REQUIRED_ARGUMENTS_NONE,
    }
  }

  pub(crate) fn all_required_arguments_provided(&self, provided_argument: &ProvidedArguments) -> bool {
    let RequiredArguments { app_id_required, bucket_id_required, proxy_id_required, service_id_required, topic_required, vendor_id_required, vhost_required } =
      self.required_arguments();
    (!app_id_required || provided_argument.app_id.is_some())
      && (!bucket_id_required || provided_argument.bucket_id.is_some())
      && (!proxy_id_required || provided_argument.proxy_id.is_some())
      && (!service_id_required || provided_argument.service_id.is_some())
      && (!topic_required || provided_argument.topic_id.is_some())
      && (!vendor_id_required || provided_argument.vendor_id.is_some())
      && (!vhost_required || provided_argument.vhost.is_some())
  }
}

const REQUIRED_ARGUMENTS_BUCKET: RequiredArguments = RequiredArguments {
  app_id_required: false,
  bucket_id_required: true,
  proxy_id_required: false,
  service_id_required: false,
  topic_required: false,
  vendor_id_required: false,
  vhost_required: false,
};
const REQUIRED_ARGUMENTS_APP_VENDOR: RequiredArguments = RequiredArguments {
  app_id_required: true,
  bucket_id_required: false,
  proxy_id_required: false,
  service_id_required: false,
  topic_required: false,
  vendor_id_required: true,
  vhost_required: false,
};
const REQUIRED_ARGUMENTS_APP: RequiredArguments = RequiredArguments {
  app_id_required: true,
  bucket_id_required: false,
  proxy_id_required: false,
  service_id_required: false,
  topic_required: false,
  vendor_id_required: false,
  vhost_required: false,
};
const REQUIRED_ARGUMENTS_SERVICE: RequiredArguments = RequiredArguments {
  app_id_required: false,
  bucket_id_required: false,
  proxy_id_required: false,
  service_id_required: true,
  topic_required: false,
  vendor_id_required: false,
  vhost_required: false,
};
const REQUIRED_ARGUMENTS_TENANT: RequiredArguments = RequiredArguments {
  app_id_required: false,
  bucket_id_required: false,
  proxy_id_required: false,
  service_id_required: false,
  topic_required: false,
  vendor_id_required: false,
  vhost_required: false,
};
const REQUIRED_ARGUMENTS_TOPIC: RequiredArguments = RequiredArguments {
  app_id_required: false,
  bucket_id_required: false,
  proxy_id_required: false,
  service_id_required: false,
  topic_required: true,
  vendor_id_required: false,
  vhost_required: false,
};
const REQUIRED_ARGUMENTS_PROXY_TENANT: RequiredArguments = RequiredArguments {
  app_id_required: false,
  bucket_id_required: false,
  proxy_id_required: true,
  service_id_required: false,
  topic_required: false,
  vendor_id_required: false,
  vhost_required: false,
};
const REQUIRED_ARGUMENTS_VHOST: RequiredArguments = RequiredArguments {
  app_id_required: false,
  bucket_id_required: false,
  proxy_id_required: false,
  service_id_required: false,
  topic_required: false,
  vendor_id_required: false,
  vhost_required: true,
};
const REQUIRED_ARGUMENTS_NONE: RequiredArguments = RequiredArguments {
  app_id_required: false,
  bucket_id_required: false,
  proxy_id_required: false,
  service_id_required: false,
  topic_required: false,
  vendor_id_required: false,
  vhost_required: false,
};
