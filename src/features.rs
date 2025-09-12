//! Feature flag management for progressive epic development
//! 
//! This module provides centralized feature flag control to enable
//! safe integration of incomplete epic functionality during development.

use std::env;
use serde::{Deserialize, Serialize};

/// Feature flags for controlling epic functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Features {
    /// Epic 2: Data Pipeline & DuckDB Integration
    pub epic_2_data_pipeline: bool,
    
    /// Epic 3: Multi-Timeframe State Engine
    pub epic_3_mtf_engine: bool,
    
    /// Epic 4: Python Algorithm Bridge
    pub epic_4_python_bridge: bool,
    
    /// Epic 5: Electron/React Frontend
    pub epic_5_frontend: bool,
    
    /// Epic 6: Advanced Charting
    pub epic_6_charting: bool,
    
    /// Epic 7: Statistical Analysis & Reporting
    pub epic_7_analytics: bool,
}

impl Features {
    /// Load features from environment variables
    pub fn from_env() -> Self {
        Self {
            epic_2_data_pipeline: Self::is_enabled("FEATURE_EPIC_2"),
            epic_3_mtf_engine: Self::is_enabled("FEATURE_EPIC_3"),
            epic_4_python_bridge: Self::is_enabled("FEATURE_EPIC_4"),
            epic_5_frontend: Self::is_enabled("FEATURE_EPIC_5"),
            epic_6_charting: Self::is_enabled("FEATURE_EPIC_6"),
            epic_7_analytics: Self::is_enabled("FEATURE_EPIC_7"),
        }
    }
    
    /// Create features for testing with all flags enabled
    #[cfg(test)]
    pub fn all_enabled() -> Self {
        Self {
            epic_2_data_pipeline: true,
            epic_3_mtf_engine: true,
            epic_4_python_bridge: true,
            epic_5_frontend: true,
            epic_6_charting: true,
            epic_7_analytics: true,
        }
    }
    
    /// Create features for testing with all flags disabled
    #[cfg(test)]
    pub fn all_disabled() -> Self {
        Self {
            epic_2_data_pipeline: false,
            epic_3_mtf_engine: false,
            epic_4_python_bridge: false,
            epic_5_frontend: false,
            epic_6_charting: false,
            epic_7_analytics: false,
        }
    }
    
    /// Check if a feature is enabled via environment variable
    fn is_enabled(env_var: &str) -> bool {
        env::var(env_var)
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false)
    }
    
    /// Check if the data pipeline is fully functional
    pub fn is_data_ready(&self) -> bool {
        self.epic_2_data_pipeline && self.epic_3_mtf_engine
    }
    
    /// Check if algorithmic trading is available
    pub fn is_algo_trading_ready(&self) -> bool {
        self.is_data_ready() && self.epic_4_python_bridge
    }
    
    /// Check if the UI is available
    pub fn is_ui_ready(&self) -> bool {
        self.epic_5_frontend
    }
    
    /// Check if full analytics are available
    pub fn is_analytics_ready(&self) -> bool {
        self.is_data_ready() && self.epic_7_analytics
    }
}

impl Default for Features {
    fn default() -> Self {
        // In production, default to only stable features
        if cfg!(debug_assertions) {
            // Development mode - load from environment
            Self::from_env()
        } else {
            // Production mode - only enable completed epics
            Self {
                epic_2_data_pipeline: false,  // Enable when Epic 2 complete
                epic_3_mtf_engine: false,      // Enable when Epic 3 complete
                epic_4_python_bridge: false,   // Enable when Epic 4 complete
                epic_5_frontend: false,        // Enable when Epic 5 complete
                epic_6_charting: false,        // Enable when Epic 6 complete
                epic_7_analytics: false,       // Enable when Epic 7 complete
            }
        }
    }
}

/// Global feature flag instance
static mut FEATURES: Option<Features> = None;
static FEATURES_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global features instance
pub fn features() -> &'static Features {
    unsafe {
        FEATURES_INIT.call_once(|| {
            FEATURES = Some(Features::from_env());
        });
        FEATURES.as_ref().expect("Features not initialized")
    }
}

/// Initialize features with custom configuration (for testing)
#[cfg(test)]
pub fn init_features_with(features: Features) {
    unsafe {
        FEATURES = Some(features);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_flags_from_env() {
        // Set test environment
        env::set_var("FEATURE_EPIC_2", "true");
        env::set_var("FEATURE_EPIC_3", "false");
        
        let features = Features::from_env();
        assert!(features.epic_2_data_pipeline);
        assert!(!features.epic_3_mtf_engine);
        
        // Cleanup
        env::remove_var("FEATURE_EPIC_2");
        env::remove_var("FEATURE_EPIC_3");
    }
    
    #[test]
    fn test_feature_dependencies() {
        let mut features = Features::all_disabled();
        
        // Data not ready without both epics
        assert!(!features.is_data_ready());
        
        features.epic_2_data_pipeline = true;
        assert!(!features.is_data_ready());
        
        features.epic_3_mtf_engine = true;
        assert!(features.is_data_ready());
        
        // Algo trading needs data + python
        assert!(!features.is_algo_trading_ready());
        features.epic_4_python_bridge = true;
        assert!(features.is_algo_trading_ready());
    }
}