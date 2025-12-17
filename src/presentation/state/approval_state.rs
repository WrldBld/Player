//! Approval state management using Dioxus signals
//!
//! Tracks pending approvals, decision history, and conversation log for DM view.

use dioxus::prelude::*;
use std::sync::Arc;

use crate::application::dto::{ProposedTool, ChallengeSuggestionInfo, NarrativeEventSuggestionInfo};
use crate::application::ports::outbound::{ApprovalDecision, GameConnectionPort, Platform};

/// A pending approval request from the LLM that the DM needs to review
#[derive(Debug, Clone, PartialEq)]
pub struct PendingApproval {
    /// Unique request ID for this approval
    pub request_id: String,
    /// Name of the NPC responding
    pub npc_name: String,
    /// The proposed dialogue from the LLM
    pub proposed_dialogue: String,
    /// Internal reasoning from the LLM
    pub internal_reasoning: String,
    /// Proposed tool calls
    pub proposed_tools: Vec<ProposedTool>,
    /// Optional challenge suggestion from the Engine
    pub challenge_suggestion: Option<ChallengeSuggestionInfo>,
    /// Optional narrative event suggestion from the Engine
    pub narrative_event_suggestion: Option<NarrativeEventSuggestionInfo>,
}

/// A past approval decision for lightweight decision history in the DM view
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalHistoryEntry {
    /// Request ID this decision relates to
    pub request_id: String,
    /// NPC name involved in the decision
    pub npc_name: String,
    /// Outcome label (e.g. "accepted", "modified", "rejected", "takeover")
    pub outcome: String,
    /// Unix timestamp (seconds) when the decision was made
    pub timestamp: u64,
}

/// A log entry for the conversation
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationLogEntry {
    /// Speaker name (or "System" for system messages)
    pub speaker: String,
    /// The message text
    pub text: String,
    /// Whether this is a system message
    pub is_system: bool,
    /// Timestamp (for ordering)
    pub timestamp: u64,
}

/// Approval state for DM approval workflow
#[derive(Clone)]
pub struct ApprovalState {
    /// Pending approval requests (for DM)
    pub pending_approvals: Signal<Vec<PendingApproval>>,
    /// Recent approval decisions for DM decision history
    pub decision_history: Signal<Vec<ApprovalHistoryEntry>>,
    /// Conversation log (for DM view)
    pub conversation_log: Signal<Vec<ConversationLogEntry>>,
}

impl ApprovalState {
    /// Create a new ApprovalState
    pub fn new() -> Self {
        Self {
            pending_approvals: Signal::new(Vec::new()),
            decision_history: Signal::new(Vec::new()),
            conversation_log: Signal::new(Vec::new()),
        }
    }

    /// Add a pending approval request
    pub fn add_pending_approval(&mut self, approval: PendingApproval) {
        self.pending_approvals.write().push(approval);
    }

    /// Remove a pending approval by request_id
    pub fn remove_pending_approval(&mut self, request_id: &str) {
        self.pending_approvals.write().retain(|a| a.request_id != request_id);
    }

    /// Add an entry to the approval decision history
    pub fn add_approval_history_entry(&mut self, entry: ApprovalHistoryEntry) {
        self.decision_history.write().push(entry);
    }

    /// Get a snapshot of the approval decision history
    pub fn get_approval_history(&self) -> Vec<ApprovalHistoryEntry> {
        self.decision_history.read().clone()
    }

    /// Add a conversation log entry
    pub fn add_log_entry(&mut self, speaker: String, text: String, is_system: bool, platform: &Platform) {
        let timestamp = platform.now_unix_secs();
        self.conversation_log.write().push(ConversationLogEntry {
            speaker,
            text,
            is_system,
            timestamp,
        });
    }

    /// Record an approval decision: send it to the Engine, log it locally with
    /// a real timestamp, and remove it from the pending queue.
    pub fn record_approval_decision(
        &mut self,
        request_id: String,
        decision: &ApprovalDecision,
        platform: &Platform,
        engine_client: &Option<Arc<dyn GameConnectionPort>>,
    ) {
        // Send to Engine if we have a client
        if let Some(client) = engine_client.as_ref() {
            let svc = crate::application::services::SessionCommandService::new(Arc::clone(client));
            if let Err(e) = svc.send_approval_decision(&request_id, decision.clone()) {
                tracing::error!("Failed to send approval decision: {}", e);
            }
        }

        // Normalize outcome label
        let outcome_label = match decision {
            ApprovalDecision::Accept => "accepted",
            ApprovalDecision::AcceptWithModification { .. } => "modified",
            ApprovalDecision::Reject { .. } => "rejected",
            ApprovalDecision::TakeOver { .. } => "takeover",
        }
        .to_string();

        // Resolve NPC name from current pending approvals
        let npc_name = self
            .pending_approvals
            .read()
            .iter()
            .find(|a| a.request_id == request_id)
            .map(|a| a.npc_name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Use Platform to get a real timestamp
        let timestamp = platform.now_unix_secs();

        let entry = ApprovalHistoryEntry {
            request_id: request_id.clone(),
            npc_name,
            outcome: outcome_label,
            timestamp,
        };
        self.add_approval_history_entry(entry);

        // Remove from pending approvals
        self.remove_pending_approval(&request_id);
    }

    /// Clear all approval state
    pub fn clear(&mut self) {
        self.pending_approvals.set(Vec::new());
        self.decision_history.set(Vec::new());
        self.conversation_log.set(Vec::new());
    }
}

impl Default for ApprovalState {
    fn default() -> Self {
        Self::new()
    }
}
