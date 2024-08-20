use std::collections::HashMap;

use trifonius_dsh_api::types::AppCatalogApp as ApiAppCatalogApp;

use crate::engine_target::{template_resolver, TemplateMapping};
use crate::processor::dsh_app::dsh_app_config::{DshAppSpecificConfig, ProfileConfig};
use crate::processor::dsh_app::DshAppName;
use crate::processor::processor_config::VariableType;
use crate::processor::JunctionId;

const TRIFONIUS_PREFIX: &str = "TRIFONIUS";

pub fn into_api_app_catalog(
  app_name: &DshAppName,
  dsh_app_specific_config: &DshAppSpecificConfig,
  inbound_junctions: &HashMap<JunctionId, String>,
  outbound_junctions: &HashMap<JunctionId, String>,
  parameters: &HashMap<String, String>, // TODO ParameterId?
  _profile: &ProfileConfig,             // TODO
  _user: String,                        // TODO
  template_mapping: &TemplateMapping,
) -> Result<ApiAppCatalogApp, String> {
  let mut environment_variables: HashMap<String, String> = HashMap::new();
  environment_variables.insert(format!("{}_APP_NAME", TRIFONIUS_PREFIX), app_name.to_string());
  if let Some(ref configured_environment_variables) = dsh_app_specific_config.environment_variables {
    for (configured_environment_variable, variable) in configured_environment_variables.clone() {
      match variable.typ {
        VariableType::InboundJunction => match variable.id {
          Some(ref junction_id) => match inbound_junctions.get(&JunctionId::try_from(junction_id.as_str())?) {
            Some(parameter_value) => {
              environment_variables.insert(configured_environment_variable, parameter_value.to_string());
            }
            None => {
              return Err(format!(
                "missing inbound junction setting '{}' for variable '{}'",
                junction_id, configured_environment_variable
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::OutboundJunction => match variable.id {
          Some(ref junction_id) => match outbound_junctions.get(&JunctionId::try_from(junction_id.as_str())?) {
            Some(parameter_value) => {
              environment_variables.insert(configured_environment_variable, parameter_value.to_string());
            }
            None => {
              return Err(format!(
                "missing outbound junction setting '{}' for variable '{}'",
                junction_id, configured_environment_variable
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::DeploymentParameter => match variable.id {
          Some(ref deployment_parameter_id) => match parameters.get(deployment_parameter_id) {
            Some(parameter_value) => {
              environment_variables.insert(configured_environment_variable, parameter_value.clone());
            }
            None => {
              return Err(format!(
                "missing deployment parameter '{}' for variable '{}'",
                deployment_parameter_id, configured_environment_variable
              ))
            }
          },
          None => unreachable!(),
        },
        VariableType::Template => match variable.value {
          Some(template) => {
            let resolved = template_resolver(template.as_str(), template_mapping)?;
            environment_variables.insert(configured_environment_variable, resolved);
          }
          None => unreachable!(),
        },
        VariableType::Value => match variable.value {
          Some(parameter_value) => {
            environment_variables.insert(configured_environment_variable, parameter_value);
          }
          None => unreachable!(),
        },
      }
    }
  }

  let api_app = ApiAppCatalogApp { configuration: None, manifest_urn: "".to_string(), name: "".to_string(), resources: Default::default() };
  Ok(api_app)
}
