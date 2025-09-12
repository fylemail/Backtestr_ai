# Epic 7: Statistical Analysis & Reporting

**Goal:** Provide institutional-grade analysis of backtest results with comprehensive metrics, heatmaps, and professional reports.

## Story 7.1: Core Performance Metrics

**As a** trader,  
**I want** comprehensive performance statistics,  
**so that** I can evaluate strategy effectiveness.

### Acceptance Criteria
1: Calculate win rate, profit factor, Sharpe ratio
2: Maximum drawdown with duration and recovery
3: Average win/loss, risk-reward ratio
4: MAE/MFE statistics for all trades
5: Consecutive wins/losses tracking
6: Monthly and yearly returns table
7: Risk-adjusted returns (Sortino, Calmar)
8: All metrics calculate in <5 seconds

## Story 7.2: Performance Heatmaps

**As a** trader,  
**I want** visual heatmaps of performance,  
**so that** I can identify patterns in profitability.

### Acceptance Criteria
1: Hour-of-day heatmap showing average P&L
2: Day-of-week heatmap with win rates
3: Volatility-based heatmap (5 quintiles)
4: Session-based heatmap (Asian, London, NY)
5: Interactive tooltips with detailed stats
6: Color gradients (red to green) for intuitive reading
7: Filters to show different metrics
8: Export heatmaps as images

## Story 7.3: Trade Analysis Tables

**As a** trader,  
**I want** detailed trade lists,  
**so that** I can review individual trade performance.

### Acceptance Criteria
1: Sortable table with all trade details
2: Filters for winning/losing/all trades
3: Search by date range or position ID
4: Expandable rows showing tick-by-tick P&L
5: Export to CSV with all columns
6: Summary row with totals
7: Grouping by day/week/month
8: Column customization and persistence

## Story 7.4: Professional Reports

**As a** trader,  
**I want** professional PDF reports,  
**so that** I can document strategy performance.

### Acceptance Criteria
1: PDF generation with charts and tables
2: Executive summary on first page
3: Equity curve and drawdown chart
4: Trade distribution histograms
5: Monthly returns table
6: Risk metrics summary
7: Logo/branding customization option
8: Automated filename with strategy name and date
