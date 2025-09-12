# DevOps & Deployment

## Overview

The DevOps & Deployment architecture for BackTestr_ai is specifically designed for Windows-only desktop application distribution. This focused approach eliminates cross-platform complexity while maximizing Windows ecosystem integration, performance optimization, and simplified deployment workflows.

The deployment strategy emphasizes reliability, security, and seamless user experience through automated build pipelines, comprehensive testing, and sophisticated update mechanisms that handle the complexities of desktop application lifecycle management.

## 1. Build Pipeline

### Windows-Focused Build System

```yaml
# .github/workflows/build.yml
name: Build and Release

on:
  push:
    tags: [ 'v*' ]
  pull_request:
    branches: [ main, develop ]

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
      
      - name: Install Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      
      - name: Run Rust Tests
        run: |
          cargo test --workspace --all-features
          cargo clippy --all-targets --all-features -- -D warnings
          cargo fmt -- --check
      
      - name: Run Frontend Tests
        run: |
          pnpm install --frozen-lockfile
          pnpm run test:unit
          pnpm run test:integration
          pnpm run lint
          pnpm run type-check

  build:
    name: Build ${{ matrix.platform }}
    needs: test
    strategy:
      matrix:
        include:
          - platform: 'windows-latest'
            args: '--target x86_64-pc-windows-msvc'
            target: 'x86_64-pc-windows-msvc'
          - platform: 'windows-latest'
            args: '--target aarch64-pc-windows-msvc'
            target: 'aarch64-pc-windows-msvc'
    
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
      
      - name: Install Windows build tools
        run: |
          # MSVC tools and Windows SDK are pre-installed on windows-latest
          # Additional dependencies for Windows development
          choco install -y visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools"
      
      - name: Build application
        run: |
          pnpm install --frozen-lockfile
          pnpm run build:production
          pnpm run build:electron ${{ matrix.args }}
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: backtestr-${{ matrix.target }}
          path: |
            dist/
            !dist/node_modules/
```

### Build Configuration

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    // Configure build based on target platform
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Windows-specific optimizations
    match target.as_str() {
        "x86_64-pc-windows-msvc" => {
            configure_windows_x64_build(&out_dir);
        }
        "aarch64-pc-windows-msvc" => {
            configure_windows_arm64_build(&out_dir);
        }
        _ => {
            panic!("Unsupported target: {}. BackTestr_ai only supports Windows targets.", target);
        }
    }
    
    // Embed version information
    embed_version_info();
    
    // Configure Python embedding
    configure_python_embedding(&target);
}

fn configure_windows_x64_build(out_dir: &PathBuf) {
    // Windows x64-specific build configuration
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=ole32");
    
    // Embed Windows resources
    embed_windows_resources(out_dir);
}

fn configure_windows_arm64_build(out_dir: &PathBuf) {
    // Windows ARM64-specific build configuration
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=ole32");
    
    // ARM64-specific optimizations
    println!("cargo:rustc-env=TARGET_ARCH=aarch64");
    
    // Embed Windows resources
    embed_windows_resources(out_dir);
}

// Note: macOS and Linux build functions removed as BackTestr_ai is Windows-only
```

### Electron Build Configuration

```javascript
// electron-builder.config.js
module.exports = {
  appId: "com.backtestr.ai",
  productName: "BackTestr AI",
  directories: {
    output: "dist",
    buildResources: "build"
  },
  files: [
    "build/**/*",
    "node_modules/**/*",
    "package.json"
  ],
  extraResources: [
    {
      from: "target/release/backtestr-core",
      to: "bin/backtestr-core"
    }
  ],
  win: {
    target: [
      {
        target: "nsis",
        arch: ["x64", "arm64"]
      },
      {
        target: "portable",
        arch: ["x64", "arm64"]
      },
      {
        target: "msi",
        arch: ["x64", "arm64"]
      },
      {
        target: "appx",
        arch: ["x64", "arm64"]
      }
    ],
    icon: "build/icon.ico",
    publisherName: "BackTestr AI Inc.",
    certificateFile: process.env.WINDOWS_CERTIFICATE_FILE,
    certificatePassword: process.env.WINDOWS_CERTIFICATE_PASSWORD,
    signingHashAlgorithms: ["sha256"],
    verifyUpdateCodeSignature: true,
    requestedExecutionLevel: "asInvoker",
    artifactName: "${productName}-${version}-${arch}.${ext}"
  },
  // Note: macOS and Linux configurations removed - Windows-only application
  publish: [
    {
      provider: "github",
      owner: "backtestr-ai",
      repo: "backtestr-desktop"
    }
  ]
};
```

## 2. Testing Automation

### Test Strategy Overview

```rust
// tests/integration/mod.rs
use backtestr_core::*;
use tempfile::TempDir;
use std::sync::Arc;

pub struct TestEnvironment {
    temp_dir: TempDir,
    config: TestConfig,
    runtime: Arc<Runtime>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config = TestConfig::new(temp_dir.path());
        let runtime = Arc::new(Runtime::new_with_config(&config));
        
        Self {
            temp_dir,
            config,
            runtime,
        }
    }
    
    pub async fn load_test_data(&self, dataset: &str) -> Result<()> {
        let data_path = format!("tests/data/{}", dataset);
        self.runtime.data_manager().import_csv(&data_path).await
    }
    
    pub async fn run_backtest(&self, strategy: &str) -> Result<BacktestResults> {
        let strategy_code = std::fs::read_to_string(
            format!("tests/strategies/{}.py", strategy)
        )?;
        
        self.runtime.backtest_engine()
            .run_strategy(&strategy_code)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_backtest_pipeline() {
        let env = TestEnvironment::new();
        
        // Load test data
        env.load_test_data("eurusd_1m_sample.csv").await.unwrap();
        
        // Run simple moving average strategy
        let results = env.run_backtest("simple_ma_crossover").await.unwrap();
        
        // Verify results
        assert!(results.total_trades > 0);
        assert!(results.win_rate >= 0.0 && results.win_rate <= 1.0);
        assert!(!results.profit_factor.is_nan());
    }
    
    #[tokio::test]
    async fn test_mtf_state_consistency() {
        let env = TestEnvironment::new();
        env.load_test_data("tick_data_sample.csv").await.unwrap();
        
        let state_engine = env.runtime.mtf_engine();
        
        // Process some ticks and verify MTF consistency
        for timeframe in &[TimeFrame::M1, TimeFrame::M5, TimeFrame::H1] {
            let bars = state_engine.get_bars(*timeframe, 100);
            assert!(!bars.is_empty());
            
            // Verify bar integrity
            for window in bars.windows(2) {
                assert!(window[1].timestamp > window[0].timestamp);
                assert!(window[0].close == window[1].open || 
                        (window[1].timestamp - window[0].timestamp).num_minutes() > timeframe.minutes());
            }
        }
    }
}
```

### Frontend Testing

```typescript
// src/tests/integration/chart.test.ts
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ChartContainer } from '../components/ChartContainer';
import { MockIpcRenderer } from './mocks/ipc';

describe('Chart Integration Tests', () => {
  let mockIpc: MockIpcRenderer;
  
  beforeEach(() => {
    mockIpc = new MockIpcRenderer();
    (window as any).electronAPI = mockIpc;
  });
  
  test('chart updates with real-time data', async () => {
    render(<ChartContainer timeframe="M1" symbol="EURUSD" />);
    
    // Simulate incoming tick data
    const tickData = {
      timestamp: Date.now(),
      symbol: 'EURUSD',
      bid: 1.1234,
      ask: 1.1236,
      volume: 1000
    };
    
    mockIpc.simulateMessage('tick-data', tickData);
    
    await waitFor(() => {
      expect(screen.getByTestId('chart-canvas')).toBeInTheDocument();
    });
    
    // Verify chart updated
    const chart = screen.getByTestId('chart-canvas');
    expect(chart).toHaveAttribute('data-last-price', '1.1234');
  });
  
  test('backtest replay controls work correctly', async () => {
    const user = userEvent.setup();
    render(<ChartContainer timeframe="M1" symbol="EURUSD" />);
    
    // Start replay
    await user.click(screen.getByTestId('replay-play-button'));
    
    expect(mockIpc.lastSentMessage).toMatchObject({
      type: 'backtest-replay-start',
      speed: 1
    });
    
    // Change replay speed
    await user.selectOptions(screen.getByTestId('replay-speed-select'), '10');
    
    expect(mockIpc.lastSentMessage).toMatchObject({
      type: 'backtest-replay-speed',
      speed: 10
    });
  });
});
```

### End-to-End Testing

```typescript
// e2e/tests/full-workflow.spec.ts
import { test, expect } from '@playwright/test';
import { ElectronApplication, _electron as electron } from 'playwright';

test.describe('Full Workflow Tests', () => {
  let electronApp: ElectronApplication;
  
  test.beforeAll(async () => {
    electronApp = await electron.launch({
      args: ['dist/main.js'],
      env: {
        NODE_ENV: 'test'
      }
    });
  });
  
  test.afterAll(async () => {
    await electronApp.close();
  });
  
  test('complete backtest workflow', async () => {
    const window = await electronApp.firstWindow();
    
    // Import data
    await window.click('[data-testid="import-data-button"]');
    await window.setInputFiles('[data-testid="file-input"]', 'e2e/data/sample.csv');
    await window.click('[data-testid="import-confirm"]');
    
    // Wait for import to complete
    await expect(window.locator('[data-testid="import-status"]')).toContainText('Import completed');
    
    // Configure strategy
    await window.click('[data-testid="strategy-tab"]');
    await window.fill('[data-testid="strategy-editor"]', `
def initialize(context):
    context.ma_short = 10
    context.ma_long = 20

def handle_data(context, data):
    if data.can_trade(context.symbol):
        short_ma = data.history(context.symbol, 'close', context.ma_short).mean()
        long_ma = data.history(context.symbol, 'close', context.ma_long).mean()
        
        if short_ma > long_ma and not context.portfolio.positions[context.symbol]:
            order_target_percent(context.symbol, 1.0)
        elif short_ma < long_ma and context.portfolio.positions[context.symbol]:
            order_target_percent(context.symbol, 0.0)
    `);
    
    // Run backtest
    await window.click('[data-testid="run-backtest-button"]');
    
    // Wait for completion and verify results
    await expect(window.locator('[data-testid="backtest-status"]')).toContainText('Completed', { timeout: 30000 });
    
    const totalReturn = await window.textContent('[data-testid="total-return"]');
    expect(totalReturn).toMatch(/^-?\d+\.\d+%$/);
    
    const totalTrades = await window.textContent('[data-testid="total-trades"]');
    expect(parseInt(totalTrades!)).toBeGreaterThan(0);
  });
  
  test('data import and validation', async () => {
    const window = await electronApp.firstWindow();
    
    // Test invalid data format
    await window.click('[data-testid="import-data-button"]');
    await window.setInputFiles('[data-testid="file-input"]', 'e2e/data/invalid.csv');
    await window.click('[data-testid="import-confirm"]');
    
    await expect(window.locator('[data-testid="error-message"]')).toContainText('Invalid data format');
    
    // Test valid data
    await window.setInputFiles('[data-testid="file-input"]', 'e2e/data/valid.csv');
    await window.click('[data-testid="import-confirm"]');
    
    await expect(window.locator('[data-testid="import-status"]')).toContainText('Import completed');
    
    // Verify data appears in charts
    await expect(window.locator('[data-testid="chart-canvas"]')).toBeVisible();
    const candleCount = await window.locator('[data-testid="candle-count"]').textContent();
    expect(parseInt(candleCount!)).toBeGreaterThan(0);
  });
});
```

## 3. Release Management

### Versioning Strategy

```rust
// src/version.rs
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: Version,
    pub build_number: u64,
    pub commit_hash: String,
    pub build_date: String,
    pub release_channel: ReleaseChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseChannel {
    Stable,
    Beta,
    Alpha,
    Development,
}

impl VersionInfo {
    pub fn current() -> Self {
        Self {
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            build_number: option_env!("BUILD_NUMBER")
                .unwrap_or("0")
                .parse()
                .unwrap_or(0),
            commit_hash: option_env!("GIT_COMMIT_HASH")
                .unwrap_or("unknown")
                .to_string(),
            build_date: option_env!("BUILD_DATE")
                .unwrap_or("unknown")
                .to_string(),
            release_channel: if cfg!(debug_assertions) {
                ReleaseChannel::Development
            } else {
                ReleaseChannel::Stable
            },
        }
    }
    
    pub fn is_newer_than(&self, other: &VersionInfo) -> bool {
        match self.version.cmp(&other.version) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Equal => self.build_number > other.build_number,
            std::cmp::Ordering::Less => false,
        }
    }
    
    pub fn format_display(&self) -> String {
        format!(
            "{} (build {}) - {}",
            self.version,
            self.build_number,
            match self.release_channel {
                ReleaseChannel::Stable => "Stable",
                ReleaseChannel::Beta => "Beta",
                ReleaseChannel::Alpha => "Alpha", 
                ReleaseChannel::Development => "Development",
            }
        )
    }
}
```

### Automated Release Process

```yaml
# .github/workflows/release.yml
name: Create Release

on:
  push:
    tags: [ 'v*' ]

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
      release_id: ${{ steps.create_release.outputs.id }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Generate changelog
        id: changelog
        run: |
          # Generate changelog from git history
          PREVIOUS_TAG=$(git describe --tags --abbrev=0 HEAD~1 2>/dev/null || echo "")
          if [ -n "$PREVIOUS_TAG" ]; then
            CHANGELOG=$(git log --pretty=format:"- %s (%h)" $PREVIOUS_TAG..HEAD | grep -E "(feat|fix|perf|refactor):" | head -20)
          else
            CHANGELOG="Initial release"
          fi
          echo "changelog<<EOF" >> $GITHUB_OUTPUT
          echo "$CHANGELOG" >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT
      
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: BackTestr AI ${{ github.ref }}
          body: |
            ## What's New
            
            ${{ steps.changelog.outputs.changelog }}
            
            ## Downloads
            
            Choose the appropriate Windows installer:
            
            - **Windows x64**: `BackTestr-AI-Setup-x64.exe` (Recommended NSIS installer)
            - **Windows ARM64**: `BackTestr-AI-Setup-arm64.exe` (For ARM64 Windows devices)
            - **Portable x64**: `BackTestr-AI-Portable-x64.exe` (No installation required)
            - **Enterprise MSI**: `BackTestr-AI-x64.msi` (For Group Policy deployment)
            - **Microsoft Store**: Available via Windows Store (MSIX package)
            
            ## System Requirements
            
            - **Windows 10** (64-bit) version 1903 or later
            - **Windows 11** (64-bit or ARM64) - all versions supported
            - **Windows Server 2019** or later (for enterprise deployments)
            - **Memory**: 8GB RAM minimum, 16GB recommended
            - **Storage**: 2GB available disk space
            - **GPU**: DirectX 11 compatible for chart rendering acceleration
            
            ## Installation Notes
            
            - Windows users may see SmartScreen warnings on first run (click "More info" → "Run anyway")
            - For enterprise deployment, use the MSI package with Group Policy
            - Windows Defender may require exclusions for optimal performance
            - ARM64 version is available for Windows on ARM devices
          draft: false
          prerelease: ${{ contains(github.ref, 'alpha') || contains(github.ref, 'beta') }}

  build-and-upload:
    name: Build and Upload ${{ matrix.platform }}
    needs: create-release
    strategy:
      matrix:
        include:
          - platform: 'macos-latest'
            target: 'universal-apple-darwin'
          - platform: 'ubuntu-20.04'
            target: 'x86_64-unknown-linux-gnu'
          - platform: 'windows-latest'
            target: 'x86_64-pc-windows-msvc'
    
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup and Build
        # ... (same as build workflow)
      
      - name: Sign and Package (Windows)
        if: matrix.platform == 'windows-latest'
        env:
          WINDOWS_CERTIFICATE: ${{ secrets.WINDOWS_CERTIFICATE }}
          WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
        run: |
          # Decode certificate
          echo "$WINDOWS_CERTIFICATE" | base64 --decode > certificate.p12
          
          # Sign and create installer
          pnpm run dist:windows -- --certificateFile=certificate.p12 --certificatePassword="$WINDOWS_CERTIFICATE_PASSWORD"
      
      - name: Additional Windows Packaging
        run: |
          # Create MSI and MSIX packages
          pnpm run dist:windows:msi
          pnpm run dist:windows:msix
      
      - name: Upload Release Assets
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: dist/*
          asset_name: "*"
          asset_content_type: application/octet-stream
```

### Changelog Generation

```javascript
// scripts/generate-changelog.js
const { execSync } = require('child_process');
const fs = require('fs');

class ChangelogGenerator {
  constructor() {
    this.conventionalCommitTypes = {
      feat: 'Features',
      fix: 'Bug Fixes',
      perf: 'Performance Improvements',
      refactor: 'Code Refactoring',
      style: 'Styles',
      test: 'Tests',
      docs: 'Documentation',
      build: 'Build System',
      ci: 'Continuous Integration',
      chore: 'Chores'
    };
  }
  
  getCommitsSince(tag) {
    try {
      const command = tag 
        ? `git log --pretty=format:"%H|%s|%an|%ad" --date=short ${tag}..HEAD`
        : `git log --pretty=format:"%H|%s|%an|%ad" --date=short`;
      
      const output = execSync(command, { encoding: 'utf8' });
      return output.trim().split('\n').map(line => {
        const [hash, message, author, date] = line.split('|');
        return { hash, message, author, date };
      });
    } catch (error) {
      console.error('Error getting commits:', error);
      return [];
    }
  }
  
  categorizeCommits(commits) {
    const categories = {};
    
    commits.forEach(commit => {
      const match = commit.message.match(/^(\w+)(\(.+\))?: (.+)/);
      if (match) {
        const [, type, scope, description] = match;
        const category = this.conventionalCommitTypes[type] || 'Other';
        
        if (!categories[category]) {
          categories[category] = [];
        }
        
        categories[category].push({
          ...commit,
          type,
          scope: scope ? scope.slice(1, -1) : null,
          description
        });
      } else {
        // Handle non-conventional commits
        if (!categories['Other']) {
          categories['Other'] = [];
        }
        categories['Other'].push({
          ...commit,
          description: commit.message
        });
      }
    });
    
    return categories;
  }
  
  generateMarkdown(version, categories, releaseDate) {
    let markdown = `## [${version}] - ${releaseDate}\n\n`;
    
    // Order categories by importance
    const orderedCategories = [
      'Features',
      'Bug Fixes', 
      'Performance Improvements',
      'Code Refactoring',
      'Documentation',
      'Tests',
      'Build System',
      'Continuous Integration',
      'Chores',
      'Other'
    ];
    
    orderedCategories.forEach(categoryName => {
      const category = categories[categoryName];
      if (category && category.length > 0) {
        markdown += `### ${categoryName}\n\n`;
        
        category.forEach(commit => {
          const scope = commit.scope ? `**${commit.scope}**: ` : '';
          markdown += `- ${scope}${commit.description} ([${commit.hash.substring(0, 7)}](../../commit/${commit.hash}))\n`;
        });
        
        markdown += '\n';
      }
    });
    
    return markdown;
  }
  
  updateChangelogFile(version, newEntry) {
    const changelogPath = 'CHANGELOG.md';
    let existingContent = '';
    
    if (fs.existsSync(changelogPath)) {
      existingContent = fs.readFileSync(changelogPath, 'utf8');
    } else {
      existingContent = '# Changelog\n\nAll notable changes to BackTestr AI will be documented in this file.\n\n';
    }
    
    // Insert new entry after the header
    const lines = existingContent.split('\n');
    const headerEndIndex = lines.findIndex(line => line.startsWith('## '));
    
    if (headerEndIndex === -1) {
      // No existing entries
      existingContent += newEntry;
    } else {
      lines.splice(headerEndIndex, 0, newEntry);
      existingContent = lines.join('\n');
    }
    
    fs.writeFileSync(changelogPath, existingContent);
  }
  
  generate(version, previousTag = null) {
    const commits = this.getCommitsSince(previousTag);
    const categories = this.categorizeCommits(commits);
    const releaseDate = new Date().toISOString().split('T')[0];
    
    const markdownEntry = this.generateMarkdown(version, categories, releaseDate);
    
    // Update CHANGELOG.md
    this.updateChangelogFile(version, markdownEntry);
    
    console.log('Generated changelog entry:');
    console.log(markdownEntry);
    
    return markdownEntry;
  }
}

// CLI usage
if (require.main === module) {
  const [,, version, previousTag] = process.argv;
  
  if (!version) {
    console.error('Usage: node generate-changelog.js <version> [previousTag]');
    process.exit(1);
  }
  
  const generator = new ChangelogGenerator();
  generator.generate(version, previousTag);
}

module.exports = ChangelogGenerator;
```

## 4. Distribution Strategy

### Code Signing Configuration

```javascript
// scripts/codesign.js
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

class CodeSigner {
  constructor(platform) {
    this.platform = platform;
    this.setupPlatformConfig();
  }
  
  setupPlatformConfig() {
    // Windows-only application - simplified configuration
    if (this.platform !== 'win32') {
      throw new Error('BackTestr_ai only supports Windows platform');
    }
    
    this.signtool = 'signtool.exe';
    this.certFile = process.env.WINDOWS_CERTIFICATE_FILE;
    this.certPassword = process.env.WINDOWS_CERTIFICATE_PASSWORD;
    this.timestampUrl = 'http://timestamp.sectigo.com';
  }
  
  async signWindows(filePath) {
    if (!this.certFile || !this.certPassword) {
      throw new Error('Windows code signing certificate not configured');
    }
    
    const command = [
      this.signtool,
      'sign',
      '/f', `"${this.certFile}"`,
      '/p', `"${this.certPassword}"`,
      '/tr', 'http://timestamp.sectigo.com',
      '/td', 'sha256',
      '/fd', 'sha256',
      '/as',
      `"${filePath}"`
    ].join(' ');
    
    try {
      execSync(command, { stdio: 'inherit' });
      console.log(`Successfully signed: ${filePath}`);
    } catch (error) {
      throw new Error(`Failed to sign ${filePath}: ${error.message}`);
    }
  }
  
  // Note: signMacOS method removed - Windows-only application
  
  async signFile(filePath) {
    if (!fs.existsSync(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }
    
    // Windows-only application
    if (this.platform !== 'win32') {
      throw new Error('BackTestr_ai only supports Windows platform');
    }
    
    await this.signWindows(filePath);
  }
  
  async signDirectory(dirPath, extensions = ['.exe', '.dll', '.app', '.dmg']) {
    const files = this.findFilesToSign(dirPath, extensions);
    
    for (const file of files) {
      await this.signFile(file);
    }
  }
  
  findFilesToSign(dirPath, extensions) {
    const files = [];
    
    function scanDirectory(currentPath) {
      const items = fs.readdirSync(currentPath);
      
      for (const item of items) {
        const itemPath = path.join(currentPath, item);
        const stat = fs.statSync(itemPath);
        
        if (stat.isDirectory()) {
          scanDirectory(itemPath);
        } else if (extensions.some(ext => item.endsWith(ext))) {
          files.push(itemPath);
        }
      }
    }
    
    scanDirectory(dirPath);
    return files;
  }
}

module.exports = CodeSigner;
```

### Installer Configuration

```nsis
; Windows NSIS installer script
; installer.nsi

!include "MUI2.nsh"
!include "FileFunc.nsh"

; Application information
!define APP_NAME "BackTestr AI"
!define APP_VERSION "1.0.0"
!define APP_PUBLISHER "BackTestr AI Inc."
!define APP_URL "https://backtestr.ai"
!define APP_EXECUTABLE "BackTestr AI.exe"

; Installer information
Name "${APP_NAME}"
OutFile "BackTestr-AI-Setup-x64.exe"
InstallDir "$PROGRAMFILES64\${APP_NAME}"
InstallDirRegKey HKLM "Software\${APP_NAME}" "InstallDir"
RequestExecutionLevel admin

; Modern UI configuration
!define MUI_ABORTWARNING
!define MUI_ICON "build\icon.ico"
!define MUI_UNICON "build\icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "build\installer-banner.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "build\installer-banner.bmp"

; Installer pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXECUTABLE}"
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "English"

; Version information
VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey "ProductName" "${APP_NAME}"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"
VIAddVersionKey "CompanyName" "${APP_PUBLISHER}"
VIAddVersionKey "FileDescription" "${APP_NAME} Installer"
VIAddVersionKey "FileVersion" "${APP_VERSION}"

Section "Install"
  SetOutPath "$INSTDIR"
  
  ; Install application files
  File /r "dist\*"
  
  ; Create shortcuts
  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortCut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXECUTABLE}"
  CreateShortCut "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXECUTABLE}"
  
  ; Registry entries
  WriteRegStr HKLM "Software\${APP_NAME}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "Software\${APP_NAME}" "Version" "${APP_VERSION}"
  
  ; Uninstaller registry
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
    "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
    "UninstallString" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
    "Publisher" "${APP_PUBLISHER}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
    "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
    "URLInfoAbout" "${APP_URL}"
  
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" \
    "EstimatedSize" "$0"
  
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
  ; Remove files
  RMDir /r "$INSTDIR"
  
  ; Remove shortcuts
  RMDir /r "$SMPROGRAMS\${APP_NAME}"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  
  ; Remove registry entries
  DeleteRegKey HKLM "Software\${APP_NAME}"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
SectionEnd
```

## 5. Update Mechanism

### Auto-Update System

```rust
// src/updater/mod.rs
use serde::{Deserialize, Serialize};
use semver::Version;
use std::path::PathBuf;
use tokio::time::{interval, Duration};
use reqwest::Client;
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: Version,
    pub release_notes: String,
    pub download_url: String,
    pub signature: String,
    pub size: u64,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub check_interval_hours: u64,
    pub auto_download: bool,
    pub auto_install: bool,
    pub channel: String,
    pub update_server_url: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_interval_hours: 24,
            auto_download: true,
            auto_install: false,
            channel: "stable".to_string(),
            update_server_url: "https://api.backtestr.ai/updates".to_string(),
        }
    }
}

pub struct UpdateManager {
    config: UpdateConfig,
    client: Client,
    current_version: Version,
    update_cache_dir: PathBuf,
}

impl UpdateManager {
    pub fn new(config: UpdateConfig, current_version: Version) -> Self {
        let update_cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("./cache"))
            .join("backtestr-updates");
        
        std::fs::create_dir_all(&update_cache_dir).ok();
        
        Self {
            config,
            client: Client::new(),
            current_version,
            update_cache_dir,
        }
    }
    
    pub async fn start_periodic_checks(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(
            self.config.check_interval_hours * 3600
        ));
        
        loop {
            interval.tick().await;
            
            match self.check_for_updates().await {
                Ok(Some(update_info)) => {
                    if self.config.auto_download {
                        if let Err(e) = self.download_update(&update_info).await {
                            tracing::error!("Failed to download update: {}", e);
                        }
                    }
                    
                    // Notify UI about available update
                    self.notify_update_available(&update_info).await;
                }
                Ok(None) => {
                    tracing::debug!("No updates available");
                }
                Err(e) => {
                    tracing::error!("Update check failed: {}", e);
                }
            }
        }
    }
    
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        let url = format!(
            "{}/check?version={}&channel={}&platform={}",
            self.config.update_server_url,
            self.current_version,
            self.config.channel,
            std::env::consts::OS
        );
        
        let response = self.client
            .get(&url)
            .header("User-Agent", format!("BackTestr-AI/{}", self.current_version))
            .send()
            .await?;
        
        if response.status() == 204 {
            // No updates available
            return Ok(None);
        }
        
        let update_info: UpdateInfo = response.json().await?;
        
        if update_info.version > self.current_version {
            Ok(Some(update_info))
        } else {
            Ok(None)
        }
    }
    
    pub async fn download_update(&self, update_info: &UpdateInfo) -> Result<PathBuf> {
        let file_name = format!(
            "backtestr-{}-{}.update",
            update_info.version,
            std::env::consts::OS
        );
        let download_path = self.update_cache_dir.join(&file_name);
        
        // Check if already downloaded
        if download_path.exists() {
            if self.verify_update_signature(&download_path, &update_info.signature).await? {
                return Ok(download_path);
            } else {
                std::fs::remove_file(&download_path)?;
            }
        }
        
        tracing::info!("Downloading update: {}", update_info.version);
        
        let response = self.client
            .get(&update_info.download_url)
            .send()
            .await?;
        
        let bytes = response.bytes().await?;
        tokio::fs::write(&download_path, &bytes).await?;
        
        // Verify signature
        if !self.verify_update_signature(&download_path, &update_info.signature).await? {
            std::fs::remove_file(&download_path)?;
            return Err(anyhow!("Update signature verification failed"));
        }
        
        tracing::info!("Update downloaded successfully: {}", download_path.display());
        Ok(download_path)
    }
    
    async fn verify_update_signature(&self, file_path: &PathBuf, signature: &str) -> Result<bool> {
        // Implement signature verification using your preferred method
        // This is a placeholder - implement proper cryptographic verification
        use sha2::{Sha256, Digest};
        
        let contents = tokio::fs::read(file_path).await?;
        let hash = Sha256::digest(&contents);
        let computed_signature = format!("{:x}", hash);
        
        Ok(computed_signature == signature)
    }
    
    pub async fn install_update(&self, update_path: &PathBuf) -> Result<()> {
        tracing::info!("Installing update from: {}", update_path.display());
        
        #[cfg(target_os = "windows")]
        {
            self.install_windows_update(update_path).await
        }
        
        #[cfg(target_os = "macos")]
        {
            self.install_macos_update(update_path).await
        }
        
        #[cfg(target_os = "linux")]
        {
            self.install_linux_update(update_path).await
        }
    }
    
    #[cfg(target_os = "windows")]
    async fn install_windows_update(&self, update_path: &PathBuf) -> Result<()> {
        use std::process::Command;
        
        // Run the installer with elevated permissions
        let status = Command::new("cmd")
            .args(["/C", "start", "", "/wait", update_path.to_str().unwrap(), "/S"])
            .status()?;
        
        if status.success() {
            // Schedule application restart
            self.schedule_restart().await?;
            Ok(())
        } else {
            Err(anyhow!("Update installation failed"))
        }
    }
    
    #[cfg(target_os = "macos")]
    async fn install_macos_update(&self, update_path: &PathBuf) -> Result<()> {
        use std::process::Command;
        
        // Mount DMG and copy app bundle
        let mount_point = "/tmp/backtestr-update";
        
        // Mount the DMG
        Command::new("hdiutil")
            .args(["attach", update_path.to_str().unwrap(), "-mountpoint", mount_point])
            .status()?;
        
        // Copy the app bundle
        let current_app_path = std::env::current_exe()?
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        
        Command::new("cp")
            .args(["-R", &format!("{}/BackTestr AI.app", mount_point), 
                   current_app_path.to_str().unwrap()])
            .status()?;
        
        // Unmount DMG
        Command::new("hdiutil")
            .args(["detach", mount_point])
            .status()?;
        
        self.schedule_restart().await?;
        Ok(())
    }
    
    #[cfg(target_os = "linux")]
    async fn install_linux_update(&self, update_path: &PathBuf) -> Result<()> {
        // For AppImage, replace the current executable
        let current_exe = std::env::current_exe()?;
        
        // Make backup
        let backup_path = format!("{}.backup", current_exe.to_str().unwrap());
        std::fs::copy(&current_exe, &backup_path)?;
        
        // Replace with new version
        std::fs::copy(update_path, &current_exe)?;
        
        // Make executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&current_exe, perms)?;
        
        self.schedule_restart().await?;
        Ok(())
    }
    
    async fn schedule_restart(&self) -> Result<()> {
        // Notify the main application to restart after a delay
        tracing::info!("Scheduling application restart");
        
        // Implementation depends on your IPC mechanism
        // This is a placeholder
        Ok(())
    }
    
    async fn notify_update_available(&self, update_info: &UpdateInfo) {
        // Notify UI about available update
        tracing::info!("Update available: {}", update_info.version);
        
        // Send IPC message to UI process
        // Implementation depends on your IPC mechanism
    }
}

// Delta update support for smaller downloads
pub struct DeltaUpdater {
    base_version: Version,
    target_version: Version,
}

impl DeltaUpdater {
    pub fn new(base_version: Version, target_version: Version) -> Self {
        Self {
            base_version,
            target_version,
        }
    }
    
    pub async fn create_delta_patch(&self, base_path: &PathBuf, target_path: &PathBuf) -> Result<Vec<u8>> {
        // Implement binary delta creation using bsdiff or similar
        // This is a simplified placeholder
        let base_data = tokio::fs::read(base_path).await?;
        let target_data = tokio::fs::read(target_path).await?;
        
        // In a real implementation, use a proper binary diff algorithm
        let delta = self.compute_binary_diff(&base_data, &target_data)?;
        Ok(delta)
    }
    
    pub async fn apply_delta_patch(&self, base_path: &PathBuf, delta: &[u8]) -> Result<Vec<u8>> {
        // Implement delta patch application
        let base_data = tokio::fs::read(base_path).await?;
        let patched_data = self.apply_binary_diff(&base_data, delta)?;
        Ok(patched_data)
    }
    
    fn compute_binary_diff(&self, _base: &[u8], target: &[u8]) -> Result<Vec<u8>> {
        // Placeholder - implement proper binary diffing
        Ok(target.to_vec())
    }
    
    fn apply_binary_diff(&self, _base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
        // Placeholder - implement proper patch application
        Ok(delta.to_vec())
    }
}
```

## 6. Monitoring & Analytics

### Crash Reporting System

```rust
// src/monitoring/crash_reporter.rs
use serde::{Deserialize, Serialize};
use std::panic::{catch_unwind, set_hook};
use std::sync::Mutex;
use backtrace::Backtrace;
use sysinfo::{System, SystemExt};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrashReport {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub platform: PlatformInfo,
    pub crash_info: CrashInfo,
    pub system_info: SystemInfo,
    pub user_info: Option<UserInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrashInfo {
    pub crash_type: CrashType,
    pub message: String,
    pub backtrace: Vec<StackFrame>,
    pub thread_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CrashType {
    Panic,
    SegmentationFault,
    AccessViolation,
    OutOfMemory,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackFrame {
    pub function: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_count: usize,
    pub cpu_usage: f32,
    pub uptime: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub session_id: String,
    pub last_action: Option<String>,
}

pub struct CrashReporter {
    endpoint: String,
    user_consent: bool,
    crash_dir: std::path::PathBuf,
}

impl CrashReporter {
    pub fn new(endpoint: String, user_consent: bool) -> Self {
        let crash_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("./cache"))
            .join("crash-reports");
        
        std::fs::create_dir_all(&crash_dir).ok();
        
        Self {
            endpoint,
            user_consent,
            crash_dir,
        }
    }
    
    pub fn initialize(&self) {
        if !self.user_consent {
            return;
        }
        
        // Set up panic hook
        let crash_dir = self.crash_dir.clone();
        set_hook(Box::new(move |panic_info| {
            let crash_report = CrashReporter::create_panic_report(panic_info);
            CrashReporter::save_crash_report(&crash_dir, &crash_report);
        }));
        
        // Set up signal handlers for native crashes
        #[cfg(unix)]
        self.setup_signal_handlers();
        
        #[cfg(windows)]
        self.setup_exception_handlers();
    }
    
    fn create_panic_report(panic_info: &std::panic::PanicInfo) -> CrashReport {
        let backtrace = Backtrace::new();
        let stack_frames = Self::parse_backtrace(&backtrace);
        
        let crash_info = CrashInfo {
            crash_type: CrashType::Panic,
            message: panic_info.to_string(),
            backtrace: stack_frames,
            thread_name: std::thread::current().name().map(|s| s.to_string()),
        };
        
        CrashReport {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: Self::get_platform_info(),
            crash_info,
            system_info: Self::get_system_info(),
            user_info: Self::get_user_info(),
        }
    }
    
    fn parse_backtrace(backtrace: &Backtrace) -> Vec<StackFrame> {
        backtrace
            .frames()
            .iter()
            .flat_map(|frame| frame.symbols())
            .map(|symbol| StackFrame {
                function: symbol.name().map(|n| n.to_string()),
                file: symbol.filename().map(|f| f.to_string_lossy().to_string()),
                line: symbol.lineno(),
                address: format!("{:p}", symbol.addr().unwrap_or(std::ptr::null())),
            })
            .collect()
    }
    
    fn get_platform_info() -> PlatformInfo {
        PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            version: System::new_all().long_os_version().unwrap_or_default(),
        }
    }
    
    fn get_system_info() -> SystemInfo {
        let mut system = System::new_all();
        system.refresh_all();
        
        SystemInfo {
            total_memory: system.total_memory(),
            available_memory: system.available_memory(),
            cpu_count: system.cpus().len(),
            cpu_usage: system.global_cpu_info().cpu_usage(),
            uptime: system.uptime(),
        }
    }
    
    fn get_user_info() -> Option<UserInfo> {
        // Get user info from application state if available
        // This would integrate with your user session management
        None
    }
    
    fn save_crash_report(crash_dir: &std::path::Path, report: &CrashReport) {
        let file_path = crash_dir.join(format!("crash-{}.json", report.id));
        
        if let Ok(json) = serde_json::to_string_pretty(report) {
            if let Err(e) = std::fs::write(&file_path, json) {
                eprintln!("Failed to save crash report: {}", e);
            } else {
                println!("Crash report saved: {}", file_path.display());
            }
        }
    }
    
    pub async fn upload_pending_reports(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.user_consent {
            return Ok(());
        }
        
        let client = reqwest::Client::new();
        
        for entry in std::fs::read_dir(&self.crash_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match self.upload_report(&client, &content).await {
                        Ok(_) => {
                            std::fs::remove_file(&path).ok();
                            tracing::info!("Uploaded crash report: {}", path.display());
                        }
                        Err(e) => {
                            tracing::error!("Failed to upload crash report {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn upload_report(&self, client: &reqwest::Client, report_json: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .body(report_json.to_string())
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Upload failed with status: {}", response.status()).into())
        }
    }
    
    #[cfg(unix)]
    fn setup_signal_handlers(&self) {
        use signal_hook::{consts::*, iterator::Signals};
        use std::thread;
        
        let crash_dir = self.crash_dir.clone();
        thread::spawn(move || {
            let mut signals = Signals::new(&[SIGSEGV, SIGABRT, SIGBUS, SIGFPE]).unwrap();
            
            for signal in signals.forever() {
                let crash_type = match signal {
                    SIGSEGV => CrashType::SegmentationFault,
                    SIGABRT => CrashType::Other("SIGABRT".to_string()),
                    SIGBUS => CrashType::Other("SIGBUS".to_string()),
                    SIGFPE => CrashType::Other("SIGFPE".to_string()),
                    _ => CrashType::Other(format!("Signal {}", signal)),
                };
                
                let crash_report = CrashReport {
                    id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    platform: Self::get_platform_info(),
                    crash_info: CrashInfo {
                        crash_type,
                        message: format!("Signal {} received", signal),
                        backtrace: vec![], // Would need more sophisticated backtrace capture
                        thread_name: None,
                    },
                    system_info: Self::get_system_info(),
                    user_info: Self::get_user_info(),
                };
                
                Self::save_crash_report(&crash_dir, &crash_report);
                std::process::exit(1);
            }
        });
    }
    
    #[cfg(windows)]
    fn setup_exception_handlers(&self) {
        // Windows-specific exception handling would go here
        // Using SetUnhandledExceptionFilter or similar
    }
}
```

### Usage Analytics

```rust
// src/monitoring/analytics.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticsEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
    pub session_id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub platform: String,
    pub version: String,
    pub first_session: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub daily_active_users: u64,
    pub session_duration_avg: f64,
    pub feature_usage: HashMap<String, u64>,
    pub error_rate: f64,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub startup_time: f64,
    pub backtest_completion_time: f64,
    pub memory_usage_avg: f64,
    pub cpu_usage_avg: f64,
}

pub struct AnalyticsCollector {
    events: Arc<Mutex<Vec<AnalyticsEvent>>>,
    session_info: SessionInfo,
    user_consent: bool,
    endpoint: String,
    batch_size: usize,
    flush_interval: Duration,
}

impl AnalyticsCollector {
    pub fn new(user_id: String, user_consent: bool, endpoint: String) -> Self {
        let session_info = SessionInfo {
            session_id: Uuid::new_v4().to_string(),
            user_id,
            start_time: chrono::Utc::now(),
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            version: env!("CARGO_PKG_VERSION").to_string(),
            first_session: false, // Would be determined from local storage
        };
        
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            session_info,
            user_consent,
            endpoint,
            batch_size: 50,
            flush_interval: Duration::from_secs(300), // 5 minutes
        }
    }
    
    pub fn track_event(&self, event_type: &str, properties: HashMap<String, serde_json::Value>) {
        if !self.user_consent {
            return;
        }
        
        let event = AnalyticsEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now(),
            user_id: self.session_info.user_id.clone(),
            session_id: self.session_info.session_id.clone(),
            properties,
        };
        
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
            
            if events.len() >= self.batch_size {
                // Trigger immediate flush for large batches
                let events_to_send = events.clone();
                events.clear();
                
                let endpoint = self.endpoint.clone();
                tokio::spawn(async move {
                    Self::send_events(&endpoint, &events_to_send).await;
                });
            }
        }
    }
    
    pub fn track_screen_view(&self, screen_name: &str) {
        let mut properties = HashMap::new();
        properties.insert("screen_name".to_string(), serde_json::Value::String(screen_name.to_string()));
        self.track_event("screen_view", properties);
    }
    
    pub fn track_feature_usage(&self, feature_name: &str, duration_ms: Option<u64>) {
        let mut properties = HashMap::new();
        properties.insert("feature_name".to_string(), serde_json::Value::String(feature_name.to_string()));
        
        if let Some(duration) = duration_ms {
            properties.insert("duration_ms".to_string(), serde_json::Value::Number(duration.into()));
        }
        
        self.track_event("feature_usage", properties);
    }
    
    pub fn track_backtest_completion(&self, duration_ms: u64, symbol: &str, timeframe: &str, trades_count: u32) {
        let mut properties = HashMap::new();
        properties.insert("duration_ms".to_string(), serde_json::Value::Number(duration_ms.into()));
        properties.insert("symbol".to_string(), serde_json::Value::String(symbol.to_string()));
        properties.insert("timeframe".to_string(), serde_json::Value::String(timeframe.to_string()));
        properties.insert("trades_count".to_string(), serde_json::Value::Number(trades_count.into()));
        
        self.track_event("backtest_completion", properties);
    }
    
    pub fn track_error(&self, error_type: &str, error_message: &str, context: Option<&str>) {
        let mut properties = HashMap::new();
        properties.insert("error_type".to_string(), serde_json::Value::String(error_type.to_string()));
        properties.insert("error_message".to_string(), serde_json::Value::String(error_message.to_string()));
        
        if let Some(ctx) = context {
            properties.insert("context".to_string(), serde_json::Value::String(ctx.to_string()));
        }
        
        self.track_event("error", properties);
    }
    
    pub fn track_performance_metric(&self, metric_name: &str, value: f64, unit: &str) {
        let mut properties = HashMap::new();
        properties.insert("metric_name".to_string(), serde_json::Value::String(metric_name.to_string()));
        properties.insert("value".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap()));
        properties.insert("unit".to_string(), serde_json::Value::String(unit.to_string()));
        
        self.track_event("performance_metric", properties);
    }
    
    pub async fn start_periodic_flush(&self) {
        if !self.user_consent {
            return;
        }
        
        let events = Arc::clone(&self.events);
        let endpoint = self.endpoint.clone();
        let flush_interval = self.flush_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(flush_interval);
            
            loop {
                interval.tick().await;
                
                let events_to_send = {
                    if let Ok(mut events_lock) = events.lock() {
                        if events_lock.is_empty() {
                            continue;
                        }
                        
                        let events_to_send = events_lock.clone();
                        events_lock.clear();
                        events_to_send
                    } else {
                        continue;
                    }
                };
                
                Self::send_events(&endpoint, &events_to_send).await;
            }
        });
    }
    
    async fn send_events(endpoint: &str, events: &[AnalyticsEvent]) {
        if events.is_empty() {
            return;
        }
        
        let client = reqwest::Client::new();
        
        let payload = serde_json::json!({
            "events": events,
            "batch_id": Uuid::new_v4().to_string(),
            "timestamp": chrono::Utc::now(),
        });
        
        match client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::debug!("Sent {} analytics events", events.len());
                } else {
                    tracing::error!("Failed to send analytics events: {}", response.status());
                }
            }
            Err(e) => {
                tracing::error!("Network error sending analytics events: {}", e);
            }
        }
    }
    
    pub async fn flush_all_events(&self) {
        if !self.user_consent {
            return;
        }
        
        let events_to_send = {
            if let Ok(mut events) = self.events.lock() {
                let events_to_send = events.clone();
                events.clear();
                events_to_send
            } else {
                return;
            }
        };
        
        Self::send_events(&self.endpoint, &events_to_send).await;
    }
    
    pub fn get_session_info(&self) -> &SessionInfo {
        &self.session_info
    }
}

// Privacy-focused analytics that respect user preferences
pub struct PrivacyAnalytics {
    collector: Option<AnalyticsCollector>,
    local_metrics: Arc<Mutex<UsageMetrics>>,
}

impl PrivacyAnalytics {
    pub fn new(user_consent: bool, endpoint: String) -> Self {
        let collector = if user_consent {
            Some(AnalyticsCollector::new(
                Self::generate_anonymous_user_id(),
                user_consent,
                endpoint,
            ))
        } else {
            None
        };
        
        Self {
            collector,
            local_metrics: Arc::new(Mutex::new(UsageMetrics {
                daily_active_users: 0,
                session_duration_avg: 0.0,
                feature_usage: HashMap::new(),
                error_rate: 0.0,
                performance_metrics: PerformanceMetrics {
                    startup_time: 0.0,
                    backtest_completion_time: 0.0,
                    memory_usage_avg: 0.0,
                    cpu_usage_avg: 0.0,
                },
            })),
        }
    }
    
    fn generate_anonymous_user_id() -> String {
        // Generate a stable but anonymous user ID
        // Could be based on machine ID + salt, ensuring privacy
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let machine_id = machine_uid::get().unwrap_or_else(|_| "unknown".to_string());
        let mut hasher = DefaultHasher::new();
        machine_id.hash(&mut hasher);
        format!("anon_{}", hasher.finish())
    }
    
    pub fn track_event(&self, event_type: &str, properties: HashMap<String, serde_json::Value>) {
        if let Some(collector) = &self.collector {
            collector.track_event(event_type, properties);
        }
        
        // Always update local metrics regardless of consent
        self.update_local_metrics(event_type);
    }
    
    fn update_local_metrics(&self, event_type: &str) {
        if let Ok(mut metrics) = self.local_metrics.lock() {
            let count = metrics.feature_usage.get(event_type).unwrap_or(&0) + 1;
            metrics.feature_usage.insert(event_type.to_string(), count);
        }
    }
    
    pub fn get_local_metrics(&self) -> UsageMetrics {
        self.local_metrics.lock().unwrap().clone()
    }
}
```

## Summary

The DevOps & Deployment architecture for BackTestr_ai provides a comprehensive solution for desktop application distribution and lifecycle management:

1. **Build Pipeline**: Multi-platform automated builds with comprehensive testing and quality gates
2. **Testing Automation**: Multi-layered testing strategy including unit, integration, and E2E tests
3. **Release Management**: Automated versioning, changelog generation, and release creation
4. **Distribution Strategy**: Code signing, notarization, and platform-specific installers
5. **Update Mechanism**: Secure auto-update system with delta updates and signature verification
6. **Monitoring & Analytics**: Privacy-focused crash reporting and usage analytics

This architecture ensures reliable, secure, and maintainable distribution of BackTestr_ai across all supported platforms while providing valuable insights for continuous improvement and user support.
