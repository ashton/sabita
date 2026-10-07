use chrono::NaiveDateTime;
use diesel::{Insertable, Selectable, deserialize::Queryable};
use diesel_derive_enum::DbEnum;

use crate::schema::jobs;

#[derive(Clone, Copy, Debug, DbEnum, PartialEq, Eq)]
pub enum JobType {
    SyncIntegrationLibraries,
}

#[derive(Clone, Copy, Debug, DbEnum, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, PartialEq)]
#[diesel(table_name = jobs)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub integration_id: Option<String>,
    pub error: Option<String>,
    pub created_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
}
