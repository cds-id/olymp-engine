use redis::AsyncCommands;
use sqlx::PgPool;
use tokio::time::{interval, Duration};

pub mod error;

pub use error::*;

/// Start background worker tasks
pub fn start_workers(pool: PgPool, redis: redis::Client) {
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();
    let redis1 = redis.clone();
    let redis2 = redis.clone();

    // Expired session cleanup — every 5 minutes
    // Auto-submit exam sessions past their deadline
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(300));
        loop {
            ticker.tick().await;
            if let Err(e) = cleanup_expired_sessions(&pool1).await {
                tracing::error!("Session cleanup failed: {:?}", e);
            }
        }
    });

    // Email queue processor — every 10 seconds
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;
            if let Err(e) = process_email_queue(&pool2, &redis1).await {
                tracing::error!("Email queue processing failed: {:?}", e);
            }
        }
    });

    // Scoring queue processor — every 5 seconds
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;
            if let Err(e) = process_scoring_queue(&pool3, &redis2).await {
                tracing::error!("Scoring queue processing failed: {:?}", e);
            }
        }
    });

    tracing::info!("Background workers started");
}

/// Auto-submit exam sessions that exceeded their time limit.
/// Sets status to 'submitted' so they can be scored.
pub async fn cleanup_expired_sessions(pool: &PgPool) -> Result<usize, WorkerError> {
    let result = sqlx::query(
        r#"
        UPDATE exam_sessions es
        SET status = 'submitted',
            finished_at = NOW()
        FROM exams e
        WHERE es.exam_id = e.id
          AND es.status = 'in_progress'
          AND es.started_at IS NOT NULL
          AND es.started_at + (e.duration_minutes * INTERVAL '1 minute') < NOW()
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| WorkerError::Database(e.to_string()))?;

    let count = result.rows_affected() as usize;
    if count > 0 {
        tracing::info!("Auto-submitted {} expired exam sessions", count);
    }
    Ok(count)
}

/// Process email queue from Redis
pub async fn process_email_queue(
    _pool: &PgPool,
    redis: &redis::Client,
) -> Result<usize, WorkerError> {
    let mut conn = redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    let mut processed = 0;

    loop {
        let items: Vec<(String, f64)> = conn
            .zpopmin("olymp:email_queue", 1)
            .await
            .map_err(|e| WorkerError::Redis(e.to_string()))?;

        if items.is_empty() {
            break;
        }

        for (email_json, _score) in items {
            match serde_json::from_str::<EmailJob>(&email_json) {
                Ok(job) => {
                    // TODO: send via EmailService (needs Arc<EmailService> in worker state)
                    tracing::debug!("Processing email: {} -> {}", job.template, job.to);
                    processed += 1;
                }
                Err(e) => {
                    tracing::error!("Invalid email job JSON: {:?}", e);
                }
            }
        }
    }

    Ok(processed)
}

/// Process submitted sessions that need scoring.
/// Polls DB for sessions with status='submitted', scores them via ExamRepository::score_session.
pub async fn process_scoring_queue(
    pool: &PgPool,
    _redis: &redis::Client,
) -> Result<usize, WorkerError> {
    // Find submitted sessions pending scoring
    let pending: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM exam_sessions WHERE status = 'submitted' ORDER BY finished_at ASC LIMIT 20",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| WorkerError::Database(e.to_string()))?;

    if pending.is_empty() {
        return Ok(0);
    }

    let mut processed = 0;

    for (session_id,) in pending {
        tracing::info!("Scoring session: {}", session_id);
        match olymp_exam::repository::ExamRepository::score_session(pool, session_id).await {
            Ok(result) => {
                if result.needs_manual_grading {
                    tracing::info!(
                        "Session {} needs manual grading (essay questions)",
                        session_id
                    );
                } else {
                    tracing::info!(
                        "Session {} scored: {}/{}",
                        session_id,
                        result.total_score,
                        result.max_score
                    );
                }
                processed += 1;
            }
            Err(e) => {
                tracing::error!("Failed to score session {}: {:?}", session_id, e);
            }
        }
    }

    Ok(processed)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct EmailJob {
    to: String,
    template: String,
    data: serde_json::Value,
}

/// Enqueue email for async processing
pub async fn enqueue_email(
    redis: &redis::Client,
    to: &str,
    template: &str,
    data: serde_json::Value,
) -> Result<(), WorkerError> {
    let mut conn = redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    let job = EmailJob {
        to: to.to_string(),
        template: template.to_string(),
        data,
    };

    let json =
        serde_json::to_string(&job).map_err(|e| WorkerError::Internal(e.to_string()))?;

    let score = chrono::Utc::now().timestamp();

    let _: () = conn
        .zadd("olymp:email_queue", json, score)
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    Ok(())
}
