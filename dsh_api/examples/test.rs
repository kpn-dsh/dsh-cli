// This file is intended for trying and testing the generated client code.
// Temporarily make Client::generated_client() public for this to work.

fn main() -> () {}

// use progenitor_client::{Error as ProgenitorError, ResponseValue as ProgenitorResponseValue};
// use reqwest::{Error as ReqwestError, Response as ReqwestResponse, StatusCode as ReqwestStatusCode};
// use serde::Serialize;
// use serde_json::Error as JsonError;
// use std::fmt::Display;
// use trifonius_dsh_api::{DshApiClient, DshApiError, DEFAULT_DSH_API_CLIENT_FACTORY};
//
// #[tokio::main]
// async fn main() -> Result<(), String> {
//   let app_catalog_id = "keyring-050";
//
//   let client: &DshApiClient = &DEFAULT_DSH_API_CLIENT_FACTORY.client().await?;
//
//   let response = client
//     .generated_client()
//     .app_catalog_get_by_tenant_appcatalogapp_by_appcatalogappid_configuration("greenbox-dev", app_catalog_id, client.token())
//     .await;
//
//   match process_get(response) {
//     Ok(app_catalog_app) => println!("{}", serde_json::to_string_pretty(&app_catalog_app).unwrap()),
//     Err(dsh_api_error) => println!("{}", dsh_api_error),
//   }
//
//   pub(crate) fn process_get<T: Serialize>(response: Result<ProgenitorResponseValue<T>, ProgenitorError>) -> Result<T, DshApiError> {
//     match response {
//       Ok::<ProgenitorResponseValue<T>, ProgenitorError>(response) => {
//         println!(">>>> ResponseValue >>>>");
//         println!("status:  {:?}", &response.status());
//         println!("headers: {:?}", &response.headers());
//         Ok(response.into_inner())
//       }
//       Err(ProgenitorError::InvalidRequest(string)) => {
//         println!(">>>> ProgenitorError::InvalidRequest >>>>\nstring: {}", string);
//         Err(DshApiError::Unexpected(format!("invalid request ({})", string)))
//       }
//       Err(ProgenitorError::CommunicationError(error)) => {
//         let reqwest_error: ReqwestError = error;
//         println!(">>>> ProgenitorError::CommunicationError >>>>\nreqwest error: {}", reqwest_error);
//         Err(DshApiError::Unexpected(format!("communication error (reqwest error: {})", reqwest_error)))
//       }
//       Err(ProgenitorError::InvalidUpgrade(error)) => {
//         let reqwest_error: ReqwestError = error;
//         println!(">>>> ProgenitorError::InvalidUpgrade >>>>\nreqwest error: {}", reqwest_error);
//         Err(DshApiError::Unexpected(format!("invalid upgrade (reqwest error: {})", reqwest_error)))
//       }
//       Err(ProgenitorError::ErrorResponse(response_value)) => {
//         let progenitor_response_value: ProgenitorResponseValue<()> = response_value;
//         println!(">>>> ProgenitorError::ErrorResponse >>>>\nprogenitor response: {:#?}", progenitor_response_value);
//         Err(DshApiError::Unexpected(format!(
//           "error response (progenitor response value: {:?})",
//           progenitor_response_value
//         )))
//       }
//       Err(ProgenitorError::ResponseBodyError(error)) => {
//         let reqwest_error: ReqwestError = error;
//         println!(">>>> ProgenitorError::ResponseBodyError >>>>\nreqwest error: {}", reqwest_error);
//         Err(DshApiError::Unexpected(format!("response body error (reqwest error: {})", reqwest_error)))
//       }
//       Err(ProgenitorError::InvalidResponsePayload(_bytes, error)) => {
//         let json_error: JsonError = error;
//         println!(">>>> ProgenitorError::InvalidResponsePayload >>>>\njson error: {}", json_error);
//         Err(DshApiError::Unexpected(format!("invalid response payload (json error: {})", json_error)))
//       }
//       Err(ProgenitorError::UnexpectedResponse(response_value)) => {
//         let reqwest_response: ReqwestResponse = response_value;
//         let status: ReqwestStatusCode = reqwest_response.status();
//         match status {
//           ReqwestStatusCode::NOT_FOUND => {
//             println!(">>>> ProgenitorError::UnexpectedResponse::NotFound >>>>\nreqwest response: {:#?}", reqwest_response);
//             Err(DshApiError::NotFound)
//           }
//           ReqwestStatusCode::UNAUTHORIZED | ReqwestStatusCode::FORBIDDEN => {
//             println!(
//               ">>>> ProgenitorError::UnexpectedResponse::(UNAUTHORIZED|FORBIDDEN) >>>>\nreqwest response: {:#?}",
//               reqwest_response
//             );
//             Err(DshApiError::NotAuthorized)
//           }
//           other_status_code => {
//             println!(
//               ">>>> ProgenitorError::UnexpectedResponse::{} >>>>\nreqwest response: {:#?}",
//               other_status_code, reqwest_response
//             );
//             Err(DshApiError::Unexpected(format!(
//               "unexpected response (status: {}, reqwest response: {:?})",
//               other_status_code, reqwest_response
//             )))
//           }
//         }
//       }
//       Err(ProgenitorError::PreHookError(string)) => {
//         println!(">>>> ProgenitorError::PreHookError >>>>\nstring: {}", string);
//         Err(DshApiError::Unexpected(format!("pre-hook error ({})", string)))
//       }
//     }
//   }
//
//   Ok(())
// }
