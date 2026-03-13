use std::sync::Arc;

use shaperail_runtime::cache::create_redis_pool;
use shaperail_runtime::jobs::JobQueue;

/// Show job queue depth and recent failures, or inspect a specific job by ID.
pub fn run(job_id: Option<&str>) -> i32 {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    println!("Job Queue Status");
    println!("================");
    println!("Redis: {redis_url}");
    println!();

    // Try to connect to Redis and show queue status
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Failed to create runtime: {e}");
            return 1;
        }
    };

    rt.block_on(async {
        match check_redis(&redis_url, job_id).await {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("Failed to connect to Redis: {e}");
                eprintln!("Is Redis running? Start with: docker compose up -d redis");
                1
            }
        }
    })
}

async fn check_redis(redis_url: &str, job_id: Option<&str>) -> Result<(), String> {
    let pool = Arc::new(create_redis_pool(redis_url).map_err(|e| e.to_string())?);

    if let Some(job_id) = job_id {
        return print_job_status(pool, job_id).await;
    }

    print_queue_summary(pool).await
}

async fn print_queue_summary(pool: Arc<deadpool_redis::Pool>) -> Result<(), String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Connection failed: {e}"))?;

    let queues = ["critical", "high", "normal", "low"];

    println!("{:<12} DEPTH", "QUEUE");
    println!("{}", "-".repeat(30));

    for queue in &queues {
        let key = format!("shaperail:jobs:queue:{queue}");
        let len: i64 = redis::cmd("LLEN")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);
        println!("{:<12} {}", queue, len);
    }

    // Dead letter queue
    let dead_len: i64 = redis::cmd("LLEN")
        .arg("shaperail:jobs:dead")
        .query_async(&mut conn)
        .await
        .unwrap_or(0);

    println!();
    println!("Dead letter queue: {dead_len} job(s)");

    if dead_len > 0 {
        println!();
        println!("Recent failures");
        println!("---------------");

        let recent: Vec<String> = redis::cmd("LRANGE")
            .arg("shaperail:jobs:dead")
            .arg(-5)
            .arg(-1)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        if recent.is_empty() {
            println!("No dead letter entries available.");
        } else {
            for entry in recent {
                match serde_json::from_str::<serde_json::Value>(&entry) {
                    Ok(value) => {
                        let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("-");
                        let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                        let error = value.get("error").and_then(|v| v.as_str()).unwrap_or("-");
                        println!("- {name} ({id}): {error}");
                    }
                    Err(_) => println!("- {entry}"),
                }
            }
        }
    }

    Ok(())
}

async fn print_job_status(pool: Arc<deadpool_redis::Pool>, job_id: &str) -> Result<(), String> {
    let queue = JobQueue::new(pool);
    let info = queue.get_status(job_id).await.map_err(|e| e.to_string())?;

    println!("Job Status");
    println!("==========");
    println!("ID:          {}", info.id);
    println!("Name:        {}", info.name);
    println!("Status:      {}", info.status);
    println!("Attempt:     {}", info.attempt);
    println!("Max retries: {}", info.max_retries);
    println!("Created at:  {}", info.created_at);
    println!("Updated at:  {}", info.updated_at);
    if let Some(error) = info.error {
        println!("Last error:  {error}");
    }

    Ok(())
}
