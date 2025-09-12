// Epic 1: Core functionality
pub use backtestr_core as core;
pub use backtestr_data as data;

// Epic 2+: Deferred features
#[cfg(feature = "epic_2")]
pub mod features;

// Epic 5: IPC for frontend (not needed yet)
// Will be enabled when epic_5 feature is activated and dependency is restored
// #[cfg(feature = "epic_5")]
// pub use backtestr_ipc as ipc;
