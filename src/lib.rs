// Epic 1: Core functionality
pub use backtestr_core as core;
pub use backtestr_data as data;

// Epic 2+: Deferred features
#[cfg(feature = "epic_2")]
pub mod features;

// Epic 5: IPC for frontend (not needed yet)
#[cfg(feature = "epic_5")]
pub use backtestr_ipc as ipc;
