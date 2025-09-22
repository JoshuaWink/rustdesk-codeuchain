use lazy_static::lazy_static;
use std::collections::HashMap;

/// Global migration tracker instance
lazy_static::lazy_static! {
    pub static ref MIGRATION_TRACKER: MigrationTracker = MigrationTracker::new();
}

#[derive(Debug, Clone, PartialEq)]
pub enum MigrationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

pub struct MigrationTracker {
    phases: HashMap<String, MigrationStatus>,
}

impl MigrationTracker {
    pub fn new() -> Self {
        let mut phases = HashMap::new();

        // Initialize all phases
        phases.insert("phase1_foundation".to_string(), MigrationStatus::Completed);
        phases.insert("phase2_ipc".to_string(), MigrationStatus::Completed);
        phases.insert("phase2_common".to_string(), MigrationStatus::NotStarted);
        phases.insert("phase2_platform".to_string(), MigrationStatus::NotStarted);
        phases.insert("phase3_client".to_string(), MigrationStatus::Completed);
        phases.insert("phase3_server".to_string(), MigrationStatus::Completed);
        phases.insert("phase4_ui".to_string(), MigrationStatus::NotStarted);
        phases.insert("phase4_core_main".to_string(), MigrationStatus::NotStarted);
        phases.insert("phase5_integration".to_string(), MigrationStatus::NotStarted);

        Self { phases }
    }

    pub fn get_status(&self, phase: &str) -> MigrationStatus {
        self.phases.get(phase).cloned().unwrap_or(MigrationStatus::NotStarted)
    }

    pub fn set_status(&mut self, phase: &str, status: MigrationStatus) {
        self.phases.insert(phase.to_string(), status);
    }

    pub fn is_completed(&self, phase: &str) -> bool {
        matches!(self.get_status(phase), MigrationStatus::Completed)
    }

    pub fn get_all_statuses(&self) -> &HashMap<String, MigrationStatus> {
        &self.phases
    }
}
