//! # Manage Kafka topics
//!
//! Module that contains functions to manage Kafka topics.
//!
//! * [`create_topic(topic_id, configuration) -> ()`](DshApiClient::create_topic)
//! * [`delete_topic(topic_id) -> ()`](DshApiClient::delete_topic)
//! * [`get_topic(topic_id) -> TopicStatus`](DshApiClient::get_topic)
//! * [`get_topic_allocation_status(topic_id) -> AllocationStatus`](DshApiClient::get_topic_allocation_status)
//! * [`get_topic_configuration(topic_id) -> Topic`](DshApiClient::get_topic_configuration)
//! * [`get_topic_configuration_actual(topic_id) -> Topic`](DshApiClient::get_topic_configuration_actual)
//! * [`get_topic_ids() -> Vec<String>`](DshApiClient::get_topic_ids)

use crate::types::{AllocationStatus, Topic, TopicStatus};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::{DshApiClient, DshApiResult};

/// # Manage Kafka topics
///
/// Module that contains functions to manage Kafka topics.
///
/// * [`create_topic(topic_id, configuration) -> ()`](DshApiClient::create_topic)
/// * [`delete_topic(topic_id) -> ()`](DshApiClient::delete_topic)
/// * [`get_topic(topic_id) -> TopicStatus`](DshApiClient::get_topic)
/// * [`get_topic_allocation_status(topic_id) -> AllocationStatus`](DshApiClient::get_topic_allocation_status)
/// * [`get_topic_configuration(topic_id) -> Topic`](DshApiClient::get_topic_configuration)
/// * [`get_topic_configuration_actual(topic_id) -> Topic`](DshApiClient::get_topic_configuration_actual)
/// * [`get_topic_ids() -> Vec<String>`](DshApiClient::get_topic_ids)
impl DshApiClient<'_> {
  /// # Create topic
  ///
  /// `PUT /allocation/{tenant}/topic/{id}/configuration`
  ///
  /// ## Parameters
  /// * `topic_id` - name of the created topic
  /// * `configuration` - configuration for the created topic
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the topic has been successfully created)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_topic(&self, topic_id: &str, configuration: &Topic) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .topic_put_by_tenant_topic_by_id_configuration(self.tenant_name(), topic_id, self.token(), configuration)
          .await,
      )
      .map(|result| result.1)
  }

  /// # Delete topic
  ///
  /// `DELETE /allocation/{tenant}/topic/{id}/configuration`
  ///
  /// ## Parameters
  /// * `topic_id` - name of the topic to delete
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the topic has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_topic(&self, topic_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .topic_delete_by_tenant_topic_by_id_configuration(self.tenant_name(), topic_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return topic status
  ///
  /// `GET /allocation/{tenant}/topic/{id}`
  ///
  /// This method combines the results of the methods
  /// [`get_topic_allocation_status()`](DshApiClient::get_topic_allocation_status),
  /// [`get_topic_configuration()`](DshApiClient::get_topic_configuration) and
  /// [`get_topic_configuration_actual()`](DshApiClient::get_topic_configuration_actual)
  /// into one method call.
  ///
  /// ## Parameters
  /// * `topic_id` - name of the requested topic
  ///
  /// ## Returns
  /// * `Ok<`[`TopicStatus`]`>` - topic status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_topic(&self, topic_id: &str) -> DshApiResult<TopicStatus> {
    self
      .process(
        self
          .generated_client
          .topic_get_by_tenant_topic_by_id(self.tenant_name(), topic_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return topic status
  ///
  /// `GET /allocation/{tenant}/topic/{id}/status`
  ///
  /// ## Parameters
  /// * `topic_id` - name of the requested topic
  ///
  /// ## Returns
  /// * `Ok<`[`AllocationStatus`]`>` - topic allocation status
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_topic_allocation_status(&self, topic_id: &str) -> DshApiResult<AllocationStatus> {
    self
      .process(
        self
          .generated_client
          .topic_get_by_tenant_topic_by_id_status(self.tenant_name(), topic_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return topic configuration
  ///
  /// `GET /allocation/{tenant}/topic/{id}/configuration`
  ///
  /// ## Parameters
  /// * `topic_id` - name of the requested topic
  ///
  /// ## Returns
  /// * `Ok<`[`Topic`]`>` - topic configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_topic_configuration(&self, topic_id: &str) -> DshApiResult<Topic> {
    self
      .process(
        self
          .generated_client
          .topic_get_by_tenant_topic_by_id_configuration(self.tenant_name(), topic_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return deployed topic configuration
  ///
  /// `GET /allocation/{tenant}/topic/{id}/actual`
  ///
  /// ## Parameters
  /// * `topic_id` - name of the requested topic
  ///
  /// ## Returns
  /// * `Ok<`[`Topic`]`>` - topic configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_topic_configuration_actual(&self, topic_id: &str) -> DshApiResult<Topic> {
    self
      .process(
        self
          .generated_client
          .topic_get_by_tenant_topic_by_id_actual(self.tenant_name(), topic_id, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return sorted list of topic names
  ///
  /// `GET /allocation/{tenant}/topic`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - list of topic names
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_topic_ids(&self) -> DshApiResult<Vec<String>> {
    let mut topic_ids: Vec<String> = self
      .process(self.generated_client.topic_get_by_tenant_topic(self.tenant_name(), self.token()).await)
      .map(|result| result.1)
      .map(|secret_ids| secret_ids.iter().map(|secret_id| secret_id.to_string()).collect())?;
    topic_ids.sort();
    Ok(topic_ids)
  }
}
