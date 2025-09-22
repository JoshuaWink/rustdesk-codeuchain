use lazy_static::lazy_static;

/// Global migration tracker instance
lazy_static::lazy_static! {
    pub static ref MIGRATION_TRACKER: MigrationTracker = MigrationTracker::new();
}

pub struct MigrationTracker;

impl MigrationTracker {
    pub fn new() -> Self {
        Self
    }
}
