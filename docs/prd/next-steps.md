# Next Steps

## UX Expert Prompt

Please review this Windows-only PRD and create the front-end specification for BackTestr_ai. Focus on the 6-panel chart layout, Windows-native dark theme professional design, data-dense visualization requirements, and Windows desktop conventions. Use Lightweight Charts library specifications and ensure 60 FPS performance targets are achievable with the proposed UI architecture. Incorporate Windows-specific UI patterns including native menus, keyboard shortcuts, and system integration.

## Architect Prompt

Please create the technical architecture document for BackTestr_ai based on this Windows-only PRD. Priority focus areas: 1) Rust-Python integration via PyO3 for <10% performance overhead, 2) MTF state synchronization achieving <100μs tick processing, 3) DuckDB schema optimization for 10+ years of tick data, 4) IPC design between main process and Electron UI, 5) Windows-specific optimizations and native API integration. Validate the hybrid architecture approach for Windows-only deployment and provide detailed component interaction diagrams optimized for Windows performance.