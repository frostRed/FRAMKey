use std::collections::VecDeque;

use anyhow::{Result, bail};
use framkey_simulation::TransactionReviewReport;
use serde_json::{Value, json};

use super::*;

#[derive(Debug)]
pub struct ReviewQueue {
    pub(crate) broker_session_id: String,
    pub(crate) next_id: u64,
    pub(crate) items: VecDeque<ReviewRequest>,
}

impl ReviewQueue {
    pub fn new() -> Self {
        Self {
            broker_session_id: new_broker_session_id(),
            next_id: 1,
            items: VecDeque::new(),
        }
    }

    #[cfg(test)]
    pub fn capture(
        &mut self,
        provider_request_id: String,
        method: String,
        origin: Option<String>,
        params: &Value,
        chain_id: &str,
        transaction_review: Option<TransactionReviewReport>,
    ) -> Result<ReviewRequest> {
        self.capture_with_asset_context(
            provider_request_id,
            method,
            origin,
            params,
            chain_id,
            transaction_review,
            None,
        )
    }

    pub fn capture_with_asset_context(
        &mut self,
        provider_request_id: String,
        method: String,
        origin: Option<String>,
        params: &Value,
        chain_id: &str,
        transaction_review: Option<TransactionReviewReport>,
        transaction_asset_context: Option<Value>,
    ) -> Result<ReviewRequest> {
        let kind = dangerous_method_kind(&method).expect("capture only called for review methods");
        let received_at_unix_ms = now_unix_ms();
        let request = ReviewRequest {
            id: self.next_review_id(),
            broker_session_id: self.broker_session_id.clone(),
            provider_request_id,
            method: method.clone(),
            kind,
            origin,
            received_at_unix_ms,
            expires_at_unix_ms: received_at_unix_ms.saturating_add(REVIEW_REQUEST_TTL_MS),
            status: ReviewStatus::Pending,
            decision: None,
            decision_token: new_decision_token()?,
            decision_token_consumed: false,
            execution: None,
            blocked_reason: blocked_reason(kind).to_owned(),
            summary: summarize_review_request(
                kind,
                &method,
                params,
                chain_id,
                transaction_review,
                transaction_asset_context,
            ),
            params_preview: truncate_value(params, 0),
        };

        self.items.push_front(request.clone());
        while self.items.len() > MAX_REVIEW_QUEUE_ITEMS {
            self.items.pop_back();
        }

        Ok(request)
    }

    pub fn snapshot(&mut self) -> Vec<ReviewRequest> {
        self.expire_pending(now_unix_ms());
        self.items.iter().cloned().collect()
    }

    pub fn decide(
        &mut self,
        review_id: &str,
        decision_token: &str,
        decision: ReviewDecision,
    ) -> Result<ReviewDecisionOutcome> {
        let now = now_unix_ms();
        self.expire_pending(now);

        let Some(request) = self.items.iter_mut().find(|item| item.id == review_id) else {
            bail!("review request {review_id} was not found");
        };

        if request.decision_token_consumed {
            bail!("review request {review_id} decision token was already consumed");
        }
        if request.decision_token != decision_token {
            bail!("review request {review_id} decision token mismatch");
        }
        if request.status == ReviewStatus::Expired {
            bail!("review request {review_id} is expired");
        }
        if request.status != ReviewStatus::Pending {
            bail!("review request {review_id} is not pending");
        }

        let broker_mode = decision_broker_mode(request, decision)?;

        request.status = match decision {
            ReviewDecision::Approve | ReviewDecision::ApproveWithRisk => ReviewStatus::Approved,
            ReviewDecision::Reject => ReviewStatus::Rejected,
        };
        request.decision_token_consumed = true;
        request.decision = Some(ReviewDecisionRecord {
            decision,
            decided_at_unix_ms: now,
        });

        let signing_enabled = matches!(
            broker_mode,
            "controlled_personal_sign"
                | "controlled_typed_data_signing"
                | "controlled_transaction_signing"
                | "controlled_transaction_high_risk_override"
        );

        Ok(ReviewDecisionOutcome {
            review_request: request.clone(),
            signing_enabled,
            broker_mode,
        })
    }

    pub fn get(&mut self, review_id: &str) -> Result<ReviewRequest> {
        self.expire_pending(now_unix_ms());
        self.items
            .iter()
            .find(|item| item.id == review_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("review request {review_id} was not found"))
    }

    pub fn mark_signed(
        &mut self,
        review_id: &str,
        address: String,
        message_hash: String,
    ) -> Result<ReviewRequest> {
        let now = now_unix_ms();
        let request = self
            .items
            .iter_mut()
            .find(|item| item.id == review_id)
            .ok_or_else(|| anyhow::anyhow!("review request {review_id} was not found"))?;
        if request.status != ReviewStatus::Approved {
            bail!("review request {review_id} is not approved");
        }
        request.status = ReviewStatus::Signed;
        request.execution = Some(ReviewExecutionRecord {
            completed_at_unix_ms: now,
            address: Some(address),
            message_hash: Some(message_hash),
            error: None,
        });
        Ok(request.clone())
    }

    pub fn mark_completed(
        &mut self,
        review_id: &str,
        address: Option<String>,
    ) -> Result<ReviewRequest> {
        let now = now_unix_ms();
        let request = self
            .items
            .iter_mut()
            .find(|item| item.id == review_id)
            .ok_or_else(|| anyhow::anyhow!("review request {review_id} was not found"))?;
        if request.status != ReviewStatus::Approved {
            bail!("review request {review_id} is not approved");
        }
        request.status = ReviewStatus::Completed;
        request.execution = Some(ReviewExecutionRecord {
            completed_at_unix_ms: now,
            address,
            message_hash: None,
            error: None,
        });
        Ok(request.clone())
    }

    pub fn mark_sign_failed(&mut self, review_id: &str, error: String) -> Result<ReviewRequest> {
        let now = now_unix_ms();
        let request = self
            .items
            .iter_mut()
            .find(|item| item.id == review_id)
            .ok_or_else(|| anyhow::anyhow!("review request {review_id} was not found"))?;
        if request.status != ReviewStatus::Approved {
            bail!("review request {review_id} is not approved");
        }
        request.status = ReviewStatus::SignFailed;
        request.execution = Some(ReviewExecutionRecord {
            completed_at_unix_ms: now,
            address: None,
            message_hash: None,
            error: Some(error),
        });
        Ok(request.clone())
    }

    pub fn dismiss(&mut self, review_id: &str) -> bool {
        self.expire_pending(now_unix_ms());
        if let Some(index) = self.items.iter().position(|item| item.id == review_id) {
            self.items.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) -> usize {
        let count = self.items.len();
        self.items.clear();
        count
    }

    pub(crate) fn expire_pending(&mut self, now_unix_ms: u64) {
        for item in &mut self.items {
            if item.status == ReviewStatus::Pending && item.expires_at_unix_ms <= now_unix_ms {
                item.status = ReviewStatus::Expired;
            }
        }
    }

    pub(crate) fn next_review_id(&mut self) -> String {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        format!("review-{id:06}")
    }
}

impl ReviewRequest {
    pub fn provider_view(&self) -> Value {
        json!({
            "id": self.id,
            "brokerSessionId": self.broker_session_id,
            "providerRequestId": self.provider_request_id,
            "method": self.method,
            "kind": self.kind,
            "origin": self.origin,
            "receivedAtUnixMs": self.received_at_unix_ms,
            "expiresAtUnixMs": self.expires_at_unix_ms,
            "status": self.status,
            "execution": self.execution,
            "blockedReason": self.blocked_reason,
            "summary": self.summary,
            "paramsPreview": self.params_preview,
        })
    }
}
