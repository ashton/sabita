use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    database,
    models::job::{Job, JobStatus, JobType},
};

pub async fn create(
    job_type: JobType,
    status: JobStatus,
    integration_id: Option<String>,
) -> Result<Job, String> {
    use crate::schema::jobs::dsl::jobs;

    let now = Utc::now().naive_utc();
    let new_job = Job {
        id: Uuid::new_v4().to_string(),
        job_type,
        status,
        integration_id,
        error: None,
        created_at: now,
        started_at: (status == JobStatus::Running).then_some(now),
        finished_at: None,
    };

    let mut conn = database::connect().await.map_err(|e| e.to_string())?;
    diesel::insert_into(jobs)
        .values(&new_job)
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_job)
}

pub async fn update_status(
    job_id: String,
    new_status: JobStatus,
    new_error: Option<String>,
) -> Result<(), String> {
    use crate::schema::jobs::dsl::{error, finished_at, id, jobs, status};

    let finished = matches!(new_status, JobStatus::Completed | JobStatus::Failed)
        .then(|| Utc::now().naive_utc());

    let mut conn = database::connect().await.map_err(|e| e.to_string())?;
    diesel::update(jobs.filter(id.eq(job_id)))
        .set((
            status.eq(new_status),
            error.eq(new_error),
            finished_at.eq(finished),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
