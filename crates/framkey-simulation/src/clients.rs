use std::{fmt, time::Duration};

use serde_json::{Value, json};

use crate::{
    alchemy::{
        alchemy_response_evidence, alchemy_result_error, alchemy_rpc_payload,
        alchemy_transport_error_message, apply_alchemy_asset_changes, mark_provider_failed,
    },
    decoder::{local_transaction_report, warning},
    model::{
        SimulationMode, SimulationStatus, TransactionSimulationReport,
        TransactionSimulationRequest, WarningSeverity,
    },
};

pub trait TransactionSimulationClient {
    fn simulate_transaction(
        &self,
        request: TransactionSimulationRequest<'_>,
    ) -> TransactionSimulationReport;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LocalDecoderSimulationClient;

impl TransactionSimulationClient for LocalDecoderSimulationClient {
    fn simulate_transaction(
        &self,
        request: TransactionSimulationRequest<'_>,
    ) -> TransactionSimulationReport {
        local_transaction_report(request.method, request.params, request.default_chain_id)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct AlchemyRpcSimulationConfig {
    pub endpoint_url: String,
    pub timeout_ms: u64,
    pub default_gas: String,
}

impl fmt::Debug for AlchemyRpcSimulationConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AlchemyRpcSimulationConfig")
            .field("endpoint_url", &"<redacted>")
            .field("timeout_ms", &self.timeout_ms)
            .field("default_gas", &self.default_gas)
            .finish()
    }
}

impl AlchemyRpcSimulationConfig {
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        Self {
            endpoint_url: endpoint_url.into(),
            timeout_ms: 5_000,
            default_gas: "0x7a1200".to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlchemyRpcSimulationClient {
    config: AlchemyRpcSimulationConfig,
}

impl AlchemyRpcSimulationClient {
    pub fn new(config: AlchemyRpcSimulationConfig) -> Self {
        Self { config }
    }
}

impl TransactionSimulationClient for AlchemyRpcSimulationClient {
    fn simulate_transaction(
        &self,
        request: TransactionSimulationRequest<'_>,
    ) -> TransactionSimulationReport {
        let mut report =
            local_transaction_report(request.method, request.params, request.default_chain_id);
        report.mode = SimulationMode::AlchemyRpc;

        if report.status == SimulationStatus::InvalidRequest {
            report.warnings.push(warning(
                WarningSeverity::Error,
                "simulation_skipped_invalid_request",
                "live simulation was skipped because the transaction request is invalid",
            ));
            return report;
        }

        let payload = alchemy_rpc_payload(request.params, &self.config);
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .build();
        let client = match http {
            Ok(client) => client,
            Err(error) => {
                mark_provider_failed(
                    &mut report,
                    "simulation_client_error",
                    format!(
                        "failed to create Alchemy RPC client: {}",
                        error.without_url()
                    ),
                    None,
                );
                return report;
            }
        };

        let response = client
            .post(&self.config.endpoint_url)
            .header("content-type", "application/json")
            .json(&payload)
            .send();
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                mark_provider_failed(
                    &mut report,
                    "simulation_provider_unavailable",
                    alchemy_transport_error_message(&error),
                    None,
                );
                return report;
            }
        };

        let status = response.status();
        let text = match response.text() {
            Ok(text) => text,
            Err(_) => {
                mark_provider_failed(
                    &mut report,
                    "simulation_provider_response_unreadable",
                    "failed to read Alchemy RPC response",
                    None,
                );
                return report;
            }
        };
        let parsed = serde_json::from_str::<Value>(&text);
        let response_body = match parsed {
            Ok(value) => value,
            Err(error) => {
                mark_provider_failed(
                    &mut report,
                    "simulation_provider_response_malformed",
                    format!("Alchemy RPC response was not JSON: {error}"),
                    Some(json!({
                        "provider": "alchemy_simulateAssetChanges",
                        "httpStatus": status.as_u16(),
                        "bodyBytes": text.len(),
                    })),
                );
                return report;
            }
        };

        if !status.is_success() {
            mark_provider_failed(
                &mut report,
                "simulation_provider_http_error",
                format!("Alchemy RPC returned HTTP {}", status.as_u16()),
                Some(alchemy_response_evidence(&response_body, status.as_u16())),
            );
            return report;
        }

        if response_body.get("error").is_some() {
            mark_provider_failed(
                &mut report,
                "simulation_provider_error",
                "Alchemy RPC returned a JSON-RPC error",
                Some(alchemy_response_evidence(&response_body, status.as_u16())),
            );
            return report;
        }
        if alchemy_result_error(&response_body).is_some() {
            mark_provider_failed(
                &mut report,
                "simulation_provider_result_error",
                "Alchemy simulation result contained an error",
                Some(alchemy_response_evidence(&response_body, status.as_u16())),
            );
            return report;
        }
        if let Err(error) = apply_alchemy_asset_changes(&mut report, &response_body) {
            mark_provider_failed(
                &mut report,
                "simulation_provider_response_malformed",
                error,
                Some(alchemy_response_evidence(&response_body, status.as_u16())),
            );
            return report;
        }

        report.provider_evidence = Some(alchemy_response_evidence(&response_body, status.as_u16()));
        report.status = SimulationStatus::ProviderSimulated;
        report
    }
}
