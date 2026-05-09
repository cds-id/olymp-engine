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

    // Reservation cleanup - every 5 minutes
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(300));
        loop {
            ticker.tick().await;
            if let Err(e) = cleanup_expired_reservations(&pool1, &redis1).await {
                tracing::error!("Reservation cleanup failed: {:?}", e);
            }
        }
    });

    // Email queue processor - every 10 seconds
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        loop {
            ticker.tick().await;
            if let Err(e) = process_email_queue(&pool2, &redis2).await {
                tracing::error!("Email queue processing failed: {:?}", e);
            }
        }
    });

    // Popularity score refresh - every 24 hours (run on startup + daily)
    tokio::spawn(async move {
        // Run immediately on startup
        if let Err(e) = refresh_popularity_scores(&pool3).await {
            tracing::error!("Initial popularity refresh failed: {:?}", e);
        }
        
        // Then every 24 hours
        let mut ticker = interval(Duration::from_secs(86400));
        loop {
            ticker.tick().await;
            if let Err(e) = refresh_popularity_scores(&pool3).await {
                tracing::error!("Popularity refresh failed: {:?}", e);
            }
        }
    });

    tracing::info!("Background workers started");
}

/// Clean up expired stock reservations
pub async fn cleanup_expired_reservations(
    pool: &PgPool,
    _redis: &redis::Client,
) -> Result<usize, WorkerError> {
    let result = sqlx::query(
        r#"
        DELETE FROM catalog.stock_reservations
        WHERE expires_at < NOW()
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| WorkerError::Database(e.to_string()))?;

    let count = result.rows_affected() as usize;
    if count > 0 {
        tracing::info!("Cleaned up {} expired stock reservations", count);
    }
    Ok(count)
}

/// Process email queue from Redis
pub async fn process_email_queue(
    pool: &PgPool,
    redis: &redis::Client,
) -> Result<usize, WorkerError> {
    let mut conn = redis.get_multiplexed_async_connection().await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    let mut processed = 0;

    loop {
        // Get next item from queue (sorted by score = timestamp)
        let items: Vec<(String, f64)> = conn.zpopmin("blurp:email_queue", 1)
            .await
            .map_err(|e| WorkerError::Redis(e.to_string()))?;

        if items.is_empty() {
            break;
        }

        for (email_json, _score) in items {
            match serde_json::from_str::<EmailJob>(&email_json) {
                Ok(job) => {
                    if let Err(e) = process_email(job, pool).await {
                        tracing::error!("Failed to process email job: {:?}", e);
                        // Re-queue with delay on failure
                        let _: () = conn.zadd(
                            "blurp:email_queue",
                            &email_json,
                            (chrono::Utc::now() + chrono::Duration::seconds(60)).timestamp() as f64
                        ).await.map_err(|e| WorkerError::Redis(e.to_string()))?;
                    } else {
                        processed += 1;
                    }
                }
                Err(e) => {
                    tracing::error!("Invalid email job JSON: {:?}", e);
                }
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

async fn process_email(job: EmailJob, _pool: &PgPool) -> Result<(), WorkerError> {
    // Get email service from pool (would need Arc<EmailService> in state)
    // For now, we just log - actual implementation needs EmailService in shared state
    tracing::debug!(
        "Processing email: {} -> {} (template: {:?})",
        job.to,
        job.template,
        job.data
    );

    // The actual email sending would be done here using EmailService
    // This requires passing Arc<EmailService> to the worker
    Ok(())
}

/// Enqueue email for async processing
pub async fn enqueue_email(
    redis: &redis::Client,
    to: &str,
    template: &str,
    data: serde_json::Value,
) -> Result<(), WorkerError> {
    let mut conn = redis.get_multiplexed_async_connection().await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    let job = EmailJob {
        to: to.to_string(),
        template: template.to_string(),
        data,
    };

    let json = serde_json::to_string(&job)
        .map_err(|e| WorkerError::Internal(e.to_string()))?;

    // Add to sorted set with timestamp as score
    let score = chrono::Utc::now().timestamp();

    let _: () = conn.zadd("blurp:email_queue", json, score)
        .await
        .map_err(|e| WorkerError::Redis(e.to_string()))?;

    Ok(())
}

/// Refresh popularity scores for all products
/// Weighted by recency: 7d = 3x, 30d = 2x, older = 1x
pub async fn refresh_popularity_scores(pool: &PgPool) -> Result<usize, WorkerError> {
    tracing::info!("Starting popularity score refresh");
    
    // Calculate weighted popularity scores from order history
    // Each order line item contributes to product popularity with recency weighting
    let result = sqlx::query(
        r#"
        WITH product_sales AS (
            SELECT 
                v.product_id,
                SUM(
                    CASE 
                        WHEN o.created_at > NOW() - INTERVAL '7 days' THEN 3 * oi.quantity
                        WHEN o.created_at > NOW() - INTERVAL '30 days' THEN 2 * oi.quantity
                        ELSE oi.quantity
                    END
                ) AS weighted_score
            FROM orders.order_items oi
            JOIN orders.orders o ON o.id = oi.order_id
            JOIN catalog.variants v ON v.id = oi.variant_id
            WHERE o.status IN ('processing', 'shipped', 'delivered')
            GROUP BY v.product_id
        )
        UPDATE catalog.products p
        SET 
            popularity_score = COALESCE(ps.weighted_score, 0),
            updated_at = NOW()
        FROM (
            SELECT id FROM catalog.products
        ) AS all_products
        LEFT JOIN product_sales ps ON ps.product_id = all_products.id
        WHERE p.id = all_products.id
        "#
    )
    .execute(pool)
    .await
    .map_err(|e| WorkerError::Database(e.to_string()))?;

    let count = result.rows_affected() as usize;
    tracing::info!("Updated popularity scores for {} products", count);
    Ok(count)
}