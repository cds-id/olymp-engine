use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use olymp_core::AppError;

pub struct CertificateRepository;

impl CertificateRepository {
    // ─── Templates ───

    pub async fn create_template(
        pool: &PgPool,
        event_id: Uuid,
        req: &CreateTemplateRequest,
    ) -> Result<CertificateTemplate, AppError> {
        sqlx::query_as::<_, CertificateTemplate>(
            "INSERT INTO certificate_templates (event_id, name, template_url, stage_id)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(event_id)
        .bind(&req.name)
        .bind(&req.template_url)
        .bind(req.stage_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn list_templates(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<Vec<CertificateTemplate>, AppError> {
        sqlx::query_as::<_, CertificateTemplate>(
            "SELECT * FROM certificate_templates WHERE event_id = $1 ORDER BY created_at",
        )
        .bind(event_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get_template(
        pool: &PgPool,
        template_id: Uuid,
    ) -> Result<Option<CertificateTemplate>, AppError> {
        sqlx::query_as::<_, CertificateTemplate>(
            "SELECT * FROM certificate_templates WHERE id = $1",
        )
        .bind(template_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update_template(
        pool: &PgPool,
        template_id: Uuid,
        req: &UpdateTemplateRequest,
    ) -> Result<CertificateTemplate, AppError> {
        let current = Self::get_template(pool, template_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Template not found".into()))?;

        let name = req.name.as_deref().unwrap_or(&current.name);
        let url = req.template_url.as_deref().unwrap_or(&current.template_url);
        let stage = req.stage_id.or(current.stage_id);

        sqlx::query_as::<_, CertificateTemplate>(
            "UPDATE certificate_templates SET name = $2, template_url = $3, stage_id = $4, updated_at = now()
             WHERE id = $1 RETURNING *",
        )
        .bind(template_id)
        .bind(name)
        .bind(url)
        .bind(stage)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Certificate Generation ───

    /// Generate certificates for a stage using template.
    /// Creates certificate records with a generated certificate_number.
    /// Actual PDF generation would be handled by a worker (placeholder URL for now).
    pub async fn generate_for_stage(
        pool: &PgPool,
        stage_id: Uuid,
        req: &GenerateCertificatesRequest,
    ) -> Result<GenerationResult, AppError> {
        // Find template for this stage
        let template = sqlx::query_as::<_, CertificateTemplate>(
            "SELECT * FROM certificate_templates WHERE stage_id = $1 LIMIT 1",
        )
        .bind(stage_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("No template configured for this stage".into()))?;

        // Get eligible participant_stages (qualified/winner/finalist)
        let eligible_statuses = vec!["qualified", "winner", "finalist", "ranked"];

        let participant_stages: Vec<Uuid> = if let Some(ref ids) = req.participant_stage_ids {
            // Validate provided IDs belong to this stage and have eligible status
            let mut valid = Vec::new();
            for id in ids {
                let exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM participant_stages WHERE id = $1 AND stage_id = $2 AND status = ANY($3))",
                )
                .bind(id)
                .bind(stage_id)
                .bind(&eligible_statuses)
                .fetch_one(pool)
                .await
                .map_err(AppError::Database)?;
                if exists {
                    valid.push(*id);
                }
            }
            valid
        } else {
            // All eligible in stage
            sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM participant_stages WHERE stage_id = $1 AND status = ANY($2)",
            )
            .bind(stage_id)
            .bind(&eligible_statuses)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?
        };

        let mut generated = 0i32;
        let mut skipped = 0i32;

        for ps_id in &participant_stages {
            // Generate unique certificate number
            let cert_number = format!(
                "CERT-{}-{}",
                &stage_id.to_string()[..8],
                &Uuid::new_v4().to_string()[..8]
            );

            // Placeholder URL — real PDF generation via worker
            let cert_url = format!("/certificates/{}.pdf", cert_number);

            let result = sqlx::query(
                "INSERT INTO certificates (template_id, participant_stage_id, certificate_url, certificate_number, generated_at)
                 VALUES ($1, $2, $3, $4, now())
                 ON CONFLICT (template_id, participant_stage_id) DO NOTHING",
            )
            .bind(template.id)
            .bind(ps_id)
            .bind(&cert_url)
            .bind(&cert_number)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

            if result.rows_affected() > 0 {
                generated += 1;
            } else {
                skipped += 1;
            }
        }

        Ok(GenerationResult {
            stage_id,
            template_id: template.id,
            generated_count: generated,
            skipped_count: skipped,
        })
    }

    // ─── Certificate Queries ───

    pub async fn list_by_participant_stage(
        pool: &PgPool,
        participant_stage_id: Uuid,
    ) -> Result<Vec<Certificate>, AppError> {
        sqlx::query_as::<_, Certificate>(
            "SELECT * FROM certificates WHERE participant_stage_id = $1 ORDER BY created_at",
        )
        .bind(participant_stage_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn list_by_participant(
        pool: &PgPool,
        participant_id: Uuid,
    ) -> Result<Vec<Certificate>, AppError> {
        sqlx::query_as::<_, Certificate>(
            "SELECT c.* FROM certificates c
             JOIN participant_stages ps ON ps.id = c.participant_stage_id
             WHERE ps.participant_id = $1
             ORDER BY c.created_at",
        )
        .bind(participant_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get_certificate(
        pool: &PgPool,
        certificate_id: Uuid,
    ) -> Result<Option<Certificate>, AppError> {
        sqlx::query_as::<_, Certificate>("SELECT * FROM certificates WHERE id = $1")
            .bind(certificate_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    // ─── Event Finalization ───

    /// Finalize event: set status to 'finalized'. All stages must be in terminal state.
    pub async fn finalize_event(pool: &PgPool, event_id: Uuid) -> Result<(), AppError> {
        // Check all stages are in terminal state
        let non_final = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM stages WHERE event_id = $1 AND status NOT IN ('result_published', 'finalized', 'cancelled')",
        )
        .bind(event_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        if non_final > 0 {
            return Err(AppError::BadRequest(format!(
                "{} stage(s) not in terminal state. Finalize or cancel all stages first.",
                non_final
            )));
        }

        sqlx::query("UPDATE events SET status = 'finalized', updated_at = now() WHERE id = $1")
            .bind(event_id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }
}
