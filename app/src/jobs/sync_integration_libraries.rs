use crate::models::job::{JobStatus, JobType};
use crate::providers::kavita::provider::KavitaProvider;
use crate::repository::{
    integration as integration_repository, job as job_repository, library as library_repository,
};

/// Spawns a dedicated OS thread that runs the job to completion in the background,
/// independent of the UI's async runtime.
pub fn spawn(integration_id: String) {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build sync-integration-libraries job runtime");

        runtime.block_on(run(integration_id));
    });
}

async fn run(integration_id: String) {
    let job = match job_repository::create(
        JobType::SyncIntegrationLibraries,
        JobStatus::Running,
        Some(integration_id.clone()),
    )
    .await
    {
        Ok(job) => job,
        Err(_) => return,
    };

    match sync(&integration_id).await {
        Ok(()) => {
            let _ = job_repository::update_status(job.id, JobStatus::Completed, None).await;
        }
        Err(error) => {
            let _ = job_repository::update_status(job.id, JobStatus::Failed, Some(error)).await;
        }
    }
}

async fn sync(integration_id: &str) -> Result<(), String> {
    let integration = integration_repository::find(integration_id).await?;
    let url = integration
        .url
        .clone()
        .ok_or_else(|| "integration is missing a url".to_string())?;
    let api_key = integration
        .api_key
        .clone()
        .ok_or_else(|| "integration is missing an api key".to_string())?;

    let provider = KavitaProvider::authenticate(url, api_key).await?;
    let remote_libraries = provider.list_libraries().await?;
    let existing_libraries = library_repository::all().await?;

    for library in remote_libraries {
        let already_exists = existing_libraries
            .iter()
            .any(|existing| existing.external_id == library.external_id);

        if already_exists {
            continue;
        }

        library_repository::create(library).await?;
    }

    Ok(())
}
