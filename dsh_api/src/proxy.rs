//! # Manage proxies
//!
//! Module that contains functions to manage Kafka proxies.
//!
//! * [`delete_proxy(proxy_id) -> ()`](DshApiClient::delete_proxy)
//! * [`get_proxy(proxy_id) -> Proxy`](DshApiClient::get_proxy)
//! * [`get_proxy_ids() -> Vec<String>`](DshApiClient::get_proxy_ids)
//! * [`update_proxy(proxy_id, proxy) -> ()`](DshApiClient::update_proxy)

use crate::dsh_api_client::DshApiClient;
#[allow(unused_imports)]
use crate::types::Empty;
use crate::types::KafkaProxy;
#[allow(unused_imports)]
use crate::DshApiError;
use crate::DshApiResult;

/// # Manage proxies
///
/// Module that contains functions to manage Kafka proxies.
///
/// * [`delete_proxy(proxy_id) -> ()`](DshApiClient::delete_proxy)
/// * [`get_proxy(proxy_id) -> Proxy`](DshApiClient::get_proxy)
/// * [`get_proxy_ids() -> Vec<String>`](DshApiClient::get_proxy_ids)
/// * [`update_proxy(proxy_id, proxy) -> ()`](DshApiClient::update_proxy)
impl DshApiClient<'_> {
  /// # Delete proxy
  ///
  /// `DELETE /allocation/{tenant}/kafkaproxy/{id}/configuration`
  ///
  /// ## Parameters
  /// * `proxy_id` - id of the proxy to delete
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the proxy has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_proxy(&self, proxy_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .kafka_proxy_delete_by_tenant_kafkaproxy_by_id_configuration(self.tenant_name(), proxy_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return proxy
  ///
  /// `GET /allocation/{tenant}/kafkaproxy/{id}/configuration`
  ///
  /// ## Parameters
  /// * `proxy_id` - id of the requested proxy
  ///
  /// ## Returns
  /// * `Ok<KafkaProxy>` - proxy
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_proxy_configuration(&self, proxy_id: &str) -> DshApiResult<KafkaProxy> {
    self
      .process(
        self
          .generated_client
          .kafka_proxy_get_by_tenant_kafkaproxy_by_id_configuration(self.tenant_name(), proxy_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return sorted list of Kafka proxy ids
  ///
  /// `GET /allocation/{tenant}/kafkaproxy`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - list of proxy ids
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_proxy_ids(&self) -> DshApiResult<Vec<String>> {
    let mut proxy_ids: Vec<String> = self
      .process(self.generated_client.kafka_proxy_get_by_tenant_kafkaproxy(self.tenant_name(), self.token()).await)
      .map(|result| result.1)
      .map(|proxy_ids| proxy_ids.iter().map(|proxy_id| proxy_id.to_string()).collect())?;
    proxy_ids.sort();
    Ok(proxy_ids)
  }

  /// # Update proxy configuration
  ///
  /// `PUT /allocation/{tenant}/kafkaproxy/{id}/configuration`
  ///
  /// ## Parameters
  /// * `proxy_id` - id of the proxy to update
  /// * `proxy` - new configuration of the proxy
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the proxy has been successfully updated)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn update_proxy(&self, proxy_id: &str, proxy: KafkaProxy) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .kafka_proxy_put_by_tenant_kafkaproxy_by_id_configuration(self.tenant_name(), proxy_id, self.token(), &proxy)
          .await,
      )
      .map(|result| result.1)
  }
}
