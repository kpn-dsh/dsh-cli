//! # Manage internal and public Kafka streams
//!
//! Module that contains functions to manage internal and public Kafka streams.
//!
//! * [`create_internal_stream(stream_id, configuration) -> ()`](DshApiClient::create_internal_stream)
//! * [`create_public_stream(stream_id, configuration) -> ()`](DshApiClient::create_public_stream)
//! * [`delete_internal_stream(stream_id) -> ()`](DshApiClient::delete_internal_stream)
//! * [`delete_public_stream(stream_id) -> ()`](DshApiClient::delete_public_stream)
//! * [`get_internal_stream(stream_id) -> InternalManagedStream`](DshApiClient::get_internal_stream)
//! * [`get_internal_streams() -> Vec<InternalManagedStream>`](DshApiClient::get_internal_streams)
//! * [`get_public_stream(stream_id) -> PublicManagedStream`](DshApiClient::get_public_stream)
//! * [`get_public_streams() -> Vec<PublicManagedStream>`](DshApiClient::get_public_streams)
//! * [`get_stream_ids() -> Vec<String>`](DshApiClient::get_stream_ids)

use std::collections::HashMap;

use futures::future::join_all;

use crate::dsh_api_client::DshApiClient;
use crate::types::{InternalManagedStream, ManagedInternalStreamId, ManagedPublicStreamId, PublicManagedStream};
#[allow(unused_imports)]
use crate::DshApiError;
use crate::DshApiResult;

/// # Manage Kafka streams
///
/// Module that contains functions to manage Kafka streams.
///
/// * [`create_internal_stream(stream_id, configuration) -> ()`](DshApiClient::create_internal_stream)
/// * [`create_public_stream(stream_id, configuration) -> ()`](DshApiClient::create_public_stream)
/// * [`delete_internal_stream(stream_id) -> ()`](DshApiClient::delete_internal_stream)
/// * [`delete_public_stream(stream_id) -> ()`](DshApiClient::delete_public_stream)
/// * [`get_internal_stream(stream_id) -> InternalManagedStream`](DshApiClient::get_internal_stream)
/// * [`get_internal_streams() -> Vec<InternalManagedStream>`](DshApiClient::get_internal_streams)
/// * [`get_public_stream(stream_id) -> PublicManagedStream`](DshApiClient::get_public_stream)
/// * [`get_public_streams() -> Vec<PublicManagedStream>`](DshApiClient::get_public_streams)
/// * [`get_stream_ids() -> Vec<String>`](DshApiClient::get_stream_ids)
impl DshApiClient<'_> {
  /// # Create internal stream
  ///
  /// `POST /manage/{manager}/stream/internal/{streamId}/configuration`
  ///
  /// ## Parameters
  /// * `stream_id` - name of the internal created stream
  /// * `configuration` - configuration for the created internal stream
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the stream has been successfully created)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_internal_stream(&self, stream_id: &str, configuration: &InternalManagedStream) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .managed_streams_post_manage_by_manager_stream_internal_by_streamid_configuration(
            self.tenant_name(),
            &ManagedInternalStreamId::try_from(stream_id)?,
            self.token(),
            configuration,
          )
          .await,
      )
      .map(|result| result.1)
  }

  /// # Create public stream
  ///
  /// `POST /manage/{manager}/stream/public/{streamId}/configuration`
  ///
  /// ## Parameters
  /// * `stream_id` - name of the public created stream
  /// * `configuration` - configuration for the created public stream
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the stream has been successfully created)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn create_public_stream(&self, stream_id: &str, configuration: &PublicManagedStream) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .managed_streams_post_manage_by_manager_stream_public_by_streamid_configuration(
            self.tenant_name(),
            &ManagedPublicStreamId::try_from(stream_id)?,
            self.token(),
            configuration,
          )
          .await,
      )
      .map(|result| result.1)
  }

  /// # Delete internal stream
  ///
  /// `DELETE /manage/{manager}/stream/internal/{streamId}/configuration`
  ///
  /// ## Parameters
  /// * `stream_id` - name of the internal stream to delete
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the stream has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_internal_stream(&self, stream_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .managed_streams_delete_manage_by_manager_stream_internal_by_streamid_configuration(self.tenant_name(), &ManagedInternalStreamId::try_from(stream_id)?, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Delete public stream
  ///
  /// `DELETE /manage/{manager}/stream/public/{streamId}/configuration`
  ///
  /// ## Parameters
  /// * `stream_id` - name of the public stream to delete
  ///
  /// ## Returns
  /// * `Ok(())` - when DSH has properly received the request
  ///              (note that this does not mean that the stream has been successfully deleted)
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn delete_public_stream(&self, stream_id: &str) -> DshApiResult<()> {
    self
      .process(
        self
          .generated_client
          .managed_streams_delete_manage_by_manager_stream_public_by_streamid_configuration(self.tenant_name(), &ManagedPublicStreamId::try_from(stream_id)?, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return internal stream configuration
  ///
  /// `GET /manage/{manager}/stream/internal/{streamId}/configuration`
  ///
  /// ## Parameters
  /// * `stream_id` - name of the requested internal stream
  ///
  /// ## Returns
  /// * `Ok<`[`InternalManagedStream`]`>` - internal stream configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_internal_stream(&self, stream_id: &str) -> DshApiResult<InternalManagedStream> {
    self
      .process(
        self
          .generated_client
          .managed_streams_get_manage_by_manager_stream_internal_by_streamid_configuration(self.tenant_name(), &ManagedInternalStreamId::try_from(stream_id)?, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return internal stream configurations as a `HashMap`
  ///
  /// `GET /manage/{manager}/stream`
  /// `GET /manage/{manager}/stream/internal/{streamId}/configuration`
  ///
  /// ## Returns
  /// * `Ok<`[`HashMap<>`]`>` - internal stream configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_internal_streams(&self) -> DshApiResult<HashMap<String, InternalManagedStream>> {
    let stream_ids = self.get_stream_ids().await?;
    let streams = join_all(stream_ids.iter().map(|stream_id| self.get_internal_stream(stream_id.as_str()))).await;
    let mut internal_streams = HashMap::<String, InternalManagedStream>::new();
    for (stream_id, stream) in stream_ids.iter().zip(streams) {
      if let Ok(internal_stream) = stream {
        internal_streams.insert(stream_id.to_string(), internal_stream);
      }
    }
    Ok(internal_streams)
  }

  /// # Return public stream configuration
  ///
  /// `GET /manage/{manager}/stream/public/{streamId}/configuration`
  ///
  /// ## Parameters
  /// * `stream_id` - name of the requested public stream
  ///
  /// ## Returns
  /// * `Ok<`[`PublicManagedStream`]`>` - public stream configuration
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_public_stream(&self, stream_id: &str) -> DshApiResult<PublicManagedStream> {
    self
      .process(
        self
          .generated_client
          .managed_streams_get_manage_by_manager_stream_public_by_streamid_configuration(self.tenant_name(), &ManagedPublicStreamId::try_from(stream_id)?, self.token())
          .await,
      )
      .map(|result| result.1)
  }

  /// # Return public stream configurations as a `HashMap`
  ///
  /// `GET /manage/{manager}/stream`
  /// `GET /manage/{manager}/stream/public/{streamId}/configuration`
  ///
  /// ## Returns
  /// * `Ok<`[`HashMap<>`]`>` - public stream configurations
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_public_streams(&self) -> DshApiResult<HashMap<String, PublicManagedStream>> {
    let stream_ids = self.get_stream_ids().await?;
    let streams = join_all(stream_ids.iter().map(|stream_id| self.get_public_stream(stream_id.as_str()))).await;
    let mut public_streams = HashMap::<String, PublicManagedStream>::new();
    for (stream_id, stream) in stream_ids.iter().zip(streams) {
      if let Ok(public_stream) = stream {
        public_streams.insert(stream_id.to_string(), public_stream);
      }
    }
    Ok(public_streams)
  }

  /// # Return sorted list of internal and public stream names
  ///
  /// `GET /manage/{manager}/stream`
  ///
  /// ## Returns
  /// * `Ok<Vec<String>>` - list of stream names
  /// * `Err<`[`DshApiError`]`>` - when the request could not be processed by the DSH
  pub async fn get_stream_ids(&self) -> DshApiResult<Vec<String>> {
    let mut stream_ids: Vec<String> = self
      .process(
        self
          .generated_client
          .managed_tenant_get_manage_by_manager_tenant(self.tenant_name(), self.token())
          .await,
      )
      .map(|result| result.1)
      .map(|secret_ids| secret_ids.iter().map(|secret_id| secret_id.to_string()).collect())?;
    stream_ids.sort();
    Ok(stream_ids)
  }
}
