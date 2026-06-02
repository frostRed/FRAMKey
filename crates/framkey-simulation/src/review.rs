use serde_json::Value;

use crate::{
    assessment::{
        evaluate_transaction_impact, evaluate_transaction_policy, evaluate_transaction_risk,
        evaluate_transaction_trust,
    },
    clients::{LocalDecoderSimulationClient, TransactionSimulationClient},
    model::{TransactionReviewReport, TransactionSimulationRequest},
};

pub fn local_transaction_review(
    method: &str,
    params: &Value,
    default_chain_id: &str,
) -> TransactionReviewReport {
    simulate_transaction_review(
        &LocalDecoderSimulationClient,
        TransactionSimulationRequest {
            method,
            params,
            default_chain_id,
        },
    )
}

pub fn simulate_transaction_review(
    client: &impl TransactionSimulationClient,
    request: TransactionSimulationRequest<'_>,
) -> TransactionReviewReport {
    let simulation = client.simulate_transaction(request);
    let policy = evaluate_transaction_policy(&simulation);
    let risk = evaluate_transaction_risk(&simulation, &policy);
    let impact = evaluate_transaction_impact(&simulation);
    let trust = evaluate_transaction_trust(&simulation);
    TransactionReviewReport {
        simulation,
        policy,
        risk,
        impact,
        trust,
    }
}
