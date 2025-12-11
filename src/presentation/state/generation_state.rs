//! Generation State - Track asset generation queue and status
//!
//! Manages the state of ComfyUI asset generation batches,
//! including queue tracking, progress updates, and ready batches.

use dioxus::prelude::*;

/// Status of a generation batch
#[derive(Debug, Clone, PartialEq)]
pub enum BatchStatus {
    /// Batch is waiting in queue
    Queued { position: u32 },
    /// Batch is currently generating
    Generating { progress: u8 },
    /// Batch is ready for selection
    Ready { asset_count: u32 },
    /// Batch generation failed
    Failed { error: String },
}

/// A generation batch in the queue
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationBatch {
    pub batch_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub asset_type: String,
    pub status: BatchStatus,
}

/// State for managing asset generation
#[derive(Clone, Copy)]
pub struct GenerationState {
    /// All tracked batches
    batches: Signal<Vec<GenerationBatch>>,
    /// Whether there are batches ready for selection
    has_ready_batches: Signal<bool>,
}

impl GenerationState {
    /// Create a new generation state
    pub fn new() -> Self {
        Self {
            batches: Signal::new(Vec::new()),
            has_ready_batches: Signal::new(false),
        }
    }

    /// Add a new batch to the queue
    pub fn add_batch(&mut self, batch: GenerationBatch) {
        self.batches.write().push(batch);
        self.update_ready_flag();
    }

    /// Update batch status when queued
    pub fn batch_queued(
        &mut self,
        batch_id: String,
        entity_type: String,
        entity_id: String,
        asset_type: String,
        position: u32,
    ) {
        let batch = GenerationBatch {
            batch_id,
            entity_type,
            entity_id,
            asset_type,
            status: BatchStatus::Queued { position },
        };
        self.add_batch(batch);
    }

    /// Update batch progress
    pub fn batch_progress(&mut self, batch_id: &str, progress: u8) {
        let mut batches = self.batches.write();
        if let Some(batch) = batches.iter_mut().find(|b| b.batch_id == batch_id) {
            batch.status = BatchStatus::Generating { progress };
        }
    }

    /// Mark batch as complete
    pub fn batch_complete(&mut self, batch_id: &str, asset_count: u32) {
        {
            let mut batches = self.batches.write();
            if let Some(batch) = batches.iter_mut().find(|b| b.batch_id == batch_id) {
                batch.status = BatchStatus::Ready { asset_count };
            }
        }
        self.update_ready_flag();
    }

    /// Mark batch as failed
    pub fn batch_failed(&mut self, batch_id: &str, error: String) {
        let mut batches = self.batches.write();
        if let Some(batch) = batches.iter_mut().find(|b| b.batch_id == batch_id) {
            batch.status = BatchStatus::Failed { error };
        }
    }

    /// Remove a batch (after selection or dismissal)
    pub fn remove_batch(&mut self, batch_id: &str) {
        self.batches.write().retain(|b| b.batch_id != batch_id);
        self.update_ready_flag();
    }

    /// Get all batches
    pub fn get_batches(&self) -> Vec<GenerationBatch> {
        self.batches.read().clone()
    }

    /// Get batches for a specific entity
    pub fn get_batches_for_entity(&self, entity_type: &str, entity_id: &str) -> Vec<GenerationBatch> {
        self.batches
            .read()
            .iter()
            .filter(|b| b.entity_type == entity_type && b.entity_id == entity_id)
            .cloned()
            .collect()
    }

    /// Get ready batches
    pub fn get_ready_batches(&self) -> Vec<GenerationBatch> {
        self.batches
            .read()
            .iter()
            .filter(|b| matches!(b.status, BatchStatus::Ready { .. }))
            .cloned()
            .collect()
    }

    /// Check if there are ready batches
    pub fn has_ready(&self) -> bool {
        *self.has_ready_batches.read()
    }

    /// Get count of active (queued or generating) batches
    pub fn active_count(&self) -> usize {
        self.batches
            .read()
            .iter()
            .filter(|b| {
                matches!(
                    b.status,
                    BatchStatus::Queued { .. } | BatchStatus::Generating { .. }
                )
            })
            .count()
    }

    fn update_ready_flag(&mut self) {
        let has_ready = self
            .batches
            .read()
            .iter()
            .any(|b| matches!(b.status, BatchStatus::Ready { .. }));
        self.has_ready_batches.set(has_ready);
    }
}

impl Default for GenerationState {
    fn default() -> Self {
        Self::new()
    }
}
