use self::{
    cleanup::cleanup_job, finalize_contest::finalize_contest_job, notification::notification_job,
};

pub mod cleanup;
pub mod finalize_contest;
pub mod notification;

pub fn spawn_all_jobs() {
    tokio::spawn(async {
        cleanup_job().await;
    });

    tokio::spawn(async {
        notification_job().await;
    });

    tokio::spawn(async {
        finalize_contest_job().await;
    });
}
