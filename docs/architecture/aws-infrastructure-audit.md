# AWS Infrastructure Audit for Backtesting Platform

## Executive Summary
Analyzing AWS deployment for 7 years of tick data (2.3B ticks, ~30GB compressed) backtesting platform.

---

## Option 1: Serverless Architecture (Most Cost-Effective)

### Architecture
```
┌─────────────────┐
│   S3 Storage    │ ← $0.69/month for 30GB
│  (Parquet files)│
└────────┬────────┘
         │
┌────────▼────────┐
│ Athena Queries  │ ← $5 per TB scanned
│  (SQL on S3)    │
└────────┬────────┘
         │
┌────────▼────────┐
│ Lambda Functions│ ← $0.20 per 1M requests
│  (Backtesting)  │
└─────────────────┘
```

### Components
- **S3**: Store Parquet files
- **Athena**: SQL queries directly on S3
- **Lambda**: Run backtests
- **Step Functions**: Orchestrate complex backtests

### Cost Breakdown (Monthly)
```
Storage:
- S3 Standard (30GB):           $0.69
- S3 Intelligent-Tiering:       $0.39

Compute:
- Athena (100 queries @ 1GB):   $0.50
- Lambda (1000 backtests):       $50.00
  (3GB RAM, 5 min each)

Total: ~$51/month
```

### Pros & Cons
✅ No servers to manage
✅ Infinite scalability
✅ Pay per use
✅ Built-in partitioning support
❌ 15-minute Lambda timeout
❌ Cold starts (3-5 seconds)
❌ Complex for real-time data

---

## Option 2: Container-Based (ECS/Fargate)

### Architecture
```
┌─────────────────┐
│   S3 Storage    │
│  (Parquet files)│
└────────┬────────┘
         │
┌────────▼────────┐
│  ECS Fargate    │ ← $36/month (1 vCPU, 2GB)
│  with DuckDB    │
└────────┬────────┘
         │
┌────────▼────────┐
│  API Gateway    │
└─────────────────┘
```

### Cost Breakdown (Monthly)
```
Storage:
- S3:                           $0.69

Compute (Always-On):
- Fargate (1 vCPU, 2GB):       $36.00
- Or Spot Instances:           $11.00

Compute (On-Demand):
- Fargate (8 hours/day):       $12.00

Database Option:
- RDS for PostgreSQL:          $15.00 (db.t4g.micro)
- Or Aurora Serverless v2:    $45.00 (0.5 ACU minimum)

Total: $13-50/month
```

### Pros & Cons
✅ Full control over environment
✅ Can run DuckDB natively
✅ No timeout limits
❌ Need to manage containers
❌ Idle costs if always-on

---

## Option 3: EC2 Instance (Traditional)

### Architecture
```
┌─────────────────┐
│  EC2 Instance   │ ← $85/month (m5.xlarge)
│  - DuckDB       │
│  - 100GB SSD    │
└─────────────────┘
```

### Instance Options
```
Development:
- t3.medium (2 vCPU, 4GB):     $30/month
- 100GB GP3 SSD:                $8/month

Production:
- m5.xlarge (4 vCPU, 16GB):    $85/month
- Or Spot Instance:             $26/month
- 100GB GP3 SSD:                $8/month

Heavy Compute:
- c5.4xlarge (16 vCPU, 32GB): $340/month
- Or Spot:                     $102/month
```

### Pros & Cons
✅ Full control
✅ Predictable performance
✅ Can use any database
❌ Most expensive
❌ Need to manage servers
❌ Paying for idle time

---

## Option 4: AWS Native Analytics Stack

### Architecture
```
┌─────────────────┐
│   S3 Storage    │
└────────┬────────┘
         │
┌────────▼────────┐
│   Redshift      │ ← $180/month (dc2.large)
│   Serverless    │    or $90 on-demand
└────────┬────────┘
         │
┌────────▼────────┐
│   SageMaker     │ ← $50/month for notebooks
└─────────────────┘
```

### Cost Breakdown
```
Storage:
- S3:                          $0.69

Analytics:
- Redshift Serverless:         $90/month (8 hours/day)
- Or Timestream:               $120/month
- Or Redshift (always-on):     $180/month

ML/Compute:
- SageMaker Notebooks:         $50/month
- SageMaker Processing:        $0.05/hour

Total: $140-230/month
```

### Pros & Cons
✅ Purpose-built for analytics
✅ Petabyte scale
✅ Integrated ML tools
❌ Expensive
❌ Overkill for 30GB

---

## Option 5: Hybrid Local/Cloud

### Architecture
```
Local Development:
├── SQLite/DuckDB
├── Sample data (1 month)
└── Full backtesting engine

Cloud (CI/CD + Historical Data):
├── GitHub Actions (CI/CD)
├── S3 (Historical Parquet)
└── Lambda (Scheduled backtests)
```

### Cost Breakdown
```
Storage:
- S3 (30GB):                   $0.69

CI/CD:
- GitHub Actions:              FREE (2000 min/month)
- Or AWS CodeBuild:            $5/month

Occasional Compute:
- Lambda (100 backtests):      $5/month

Total: $6-11/month
```

### Pros & Cons
✅ Cheapest option
✅ Fast local development
✅ Cloud for scaling
❌ Complex deployment
❌ Data sync challenges

---

## Specialized Options

### AWS Timestream (Time-Series Database)
```
Costs:
- Storage: $0.03/GB-hour = $21/month for 30GB
- Writes: $0.50 per million writes
- Queries: $0.01 per GB scanned

Total: ~$100/month
```
✅ Purpose-built for time-series
❌ More expensive than S3+Athena

### Amazon Managed Streaming for Kafka (MSK)
```
Costs:
- Smallest instance: $140/month
- Plus storage and transfer

Total: ~$200/month
```
✅ Real-time streaming
❌ Overkill for historical backtesting

---

## Cost Comparison Summary

| Solution | Monthly Cost | Setup Complexity | Performance | Scalability |
|----------|-------------|------------------|-------------|-------------|
| **S3 + Athena + Lambda** | $51 | Medium | Good | Infinite |
| **ECS Fargate (On-Demand)** | $13 | Medium | Very Good | High |
| **EC2 Spot Instance** | $34 | High | Excellent | Limited |
| **Redshift Serverless** | $90 | Low | Excellent | Very High |
| **Hybrid Local/Cloud** | $6 | High | Good | Medium |

---

## Recommendation for Your Use Case

### For Development/MVP (Now):
**Hybrid Local/Cloud** - $6/month
- Develop locally with SQLite
- Use S3 for historical data
- GitHub Actions for CI/CD

### For Production (Later):
**S3 + Athena + Fargate** - $15-50/month
- Store all data in S3 as Parquet
- Use Athena for ad-hoc queries
- Run backtests on Fargate (on-demand)

### Why This Works:
1. **Cost-Effective**: Only pay when running backtests
2. **Scalable**: Can run 100 parallel backtests
3. **Simple**: No database to manage
4. **Fast Enough**: Athena queries complete in seconds
5. **CI/CD Friendly**: No compilation issues

---

## Implementation Path

### Phase 1 (Current): Local Development
```bash
# Everything runs locally
cargo test  # SQLite for tests
cargo run   # SQLite with sample data
```

### Phase 2: Add S3 Storage
```rust
// Download historical data from S3
aws s3 sync s3://backtest-data/historical ./data/

// Or query directly with Athena
let results = athena.query("
    SELECT * FROM ticks 
    WHERE symbol = 'AUDUSD' 
    AND date BETWEEN '2023-01-01' AND '2023-12-31'
").await?;
```

### Phase 3: Containerize for Cloud
```dockerfile
FROM rust:1.75-slim
COPY . .
RUN cargo build --release
CMD ["./target/release/backtestr"]
```

```bash
# Deploy to Fargate
aws ecs run-task --cluster backtesting \
  --task-definition backtest:latest \
  --overrides '{"containerOverrides":[{"name":"backtest","environment":[{"name":"STRATEGY","value":"ma_crossover"}]}]}'
```

---

## Decision Matrix

| Criteria | Local Only | AWS Serverless | AWS Containers | Hybrid |
|----------|------------|----------------|----------------|--------|
| Dev Speed | Excellent | Poor | Good | Excellent |
| Cost | $0 | $51/mo | $15-50/mo | $6/mo |
| Scalability | Poor | Excellent | Good | Good |
| Maintenance | Low | Medium | High | Medium |
| CI/CD Speed | Poor (20min) | N/A | Good | Excellent |
| **Best For** | **POC** | **SaaS Product** | **Power Users** | **Your Use Case** |

---

## Final Recommendation

**Start with Hybrid ($6/month):**
1. Keep SQLite for local dev and CI/CD
2. Store historical Parquet files in S3
3. Use GitHub Actions for CI/CD (free tier)
4. Add Fargate for production backtests when needed

**This gives you:**
- ✅ Fast 2-minute CI builds (SQLite)
- ✅ Cheap storage for 7 years data ($0.69/month)
- ✅ Ability to scale when needed
- ✅ No lock-in to expensive services
- ✅ Progressive enhancement path