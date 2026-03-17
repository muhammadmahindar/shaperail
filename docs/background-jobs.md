---
title: Background jobs
parent: Guides
nav_order: 7
---

Shaperail includes a Redis-backed job queue and worker primitives. The runtime
can enqueue jobs from endpoint declarations, but the scaffolded app does not
start a worker or register job handlers for you.

## What the scaffold does today

When Redis is configured, the generated app creates a `JobQueue` and write
endpoints that declare `jobs:` will enqueue named jobs after a successful write.

Example:

```yaml
endpoints:
  create:
    method: POST
    path: /users
    auth: [admin]
    input: [email, name, role, org_id]
    jobs: [send_welcome_email]
```

That enqueue step is automatic.

What is **not** automatic:

- registering Rust job handlers
- starting `shaperail_runtime::jobs::Worker`
- processing retries and dead-letter transitions

## Wiring a worker manually

To actually execute queued jobs, create a registry and spawn a worker from your
app bootstrap:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use shaperail_runtime::jobs::{JobRegistry, Worker};

let mut handlers = HashMap::new();
handlers.insert(
    "send_welcome_email".to_string(),
    Arc::new(|payload| Box::pin(async move {
        println!("job payload: {payload}");
        Ok(())
    })),
);

let registry = JobRegistry::from_handlers(handlers);
let worker = Worker::new(job_queue.clone(), registry, Duration::from_secs(1));
let (_tx, rx) = tokio::sync::watch::channel(false);
let _handle = worker.spawn(rx);
```

## Queue behavior

The runtime worker polls queues in strict priority order:

| Priority | Redis key |
| --- | --- |
| `critical` | `shaperail:jobs:queue:critical` |
| `high` | `shaperail:jobs:queue:high` |
| `normal` | `shaperail:jobs:queue:normal` |
| `low` | `shaperail:jobs:queue:low` |

Jobs declared via endpoint `jobs:` are enqueued at `normal` priority.

## Retry and dead-letter behavior

Retries and dead-letter handling are worker features. Once a worker is running:

- failed jobs are retried with exponential backoff
- timed-out or exhausted jobs move to `shaperail:jobs:dead`
- metadata is stored under `shaperail:jobs:meta:{job_id}`

Without a worker, jobs remain queued and never transition beyond `pending`.

## Monitoring

The CLI can inspect queue state whether or not a worker is running:

```bash
shaperail jobs:status
shaperail jobs:status <job_id>
```

The summary view prints queue depth by priority, dead-letter count, and recent
dead-letter entries. Passing a job ID shows the stored metadata hash for that
job.

## Practical guidance

- Use `jobs:` when the side effect can happen after the HTTP response.
- Do not assume declaring `jobs:` also creates or registers the handler.
- In a scaffolded app, "jobs stuck in pending" usually means no worker was
  started.
- Keep handlers idempotent because retries are part of the queue model.
