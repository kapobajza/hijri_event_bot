use tokio_cron_scheduler::{
    JobSchedulerError, NotificationStore,
    job::job_data_prost::{JobIdAndNotification, NotificationData},
    store::{DataStore, InitStore},
};

use crate::db::tables::NotificationState;

pub struct PostgresNotificationStore {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl PostgresNotificationStore {
    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }
}

impl DataStore<NotificationData> for PostgresNotificationStore {
    fn get(
        &mut self,
        id: sqlx::types::Uuid,
    ) -> std::pin::Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<NotificationData>,
                        tokio_cron_scheduler::JobSchedulerError,
                    >,
                > + Send,
        >,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            let notification = sqlx::query!(
                "SELECT id, job_id, extra FROM notifications WHERE id = $1",
                id,
            )
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to fetch notification: {}", e);
                JobSchedulerError::FetchJob
            })?;

            let notification = match notification {
                Some(notification) => notification,
                None => {
                    log::error!("Notification with id {} not found", id);
                    Err(JobSchedulerError::FetchJob)?
                }
            };

            let job_states = {
                sqlx::query_as!(
                    NotificationState,
                    "SELECT state FROM notification_states WHERE id = $1",
                    notification.id
                )
                .fetch_all(&pool)
                .await
                .map_err(|e| {
                    log::error!("Failed to fetch notification states: {}", e);
                    JobSchedulerError::FetchJob
                })?
            };

            Ok(Some(NotificationData {
                job_id: Some(JobIdAndNotification {
                    job_id: Some(notification.job_id.into()),
                    notification_id: Some(notification.id.into()),
                }),
                extra: notification.extra.unwrap_or_default(),
                job_states: job_states.into_iter().map(|s| s.state).collect(),
            }))
        })
    }

    fn add_or_update(
        &mut self,
        data: NotificationData,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), JobSchedulerError>> + Send>> {
        let pool = self.pool.clone();

        Box::pin(async move {
            let (job_id, notification_id) = match data.job_id_and_notification_id_from_data() {
                Some((job_id, notification_id)) => (job_id, notification_id),
                None => return Err(JobSchedulerError::UpdateJobData),
            };

            sqlx::query!(
                "INSERT INTO notifications (id, job_id, extra) \
                 VALUES ($1, $2, $3) \
                 ON CONFLICT (id) DO UPDATE SET \
                 job_id = EXCLUDED.job_id, \
                 extra = EXCLUDED.extra",
                notification_id,
                job_id,
                data.extra
            )
            .execute(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to add or update notification: {}", e);
                JobSchedulerError::CantAdd
            })?;

            Ok(())
        })
    }

    fn delete(
        &mut self,
        guid: sqlx::types::Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), JobSchedulerError>> + Send>> {
        let pool = self.pool.clone();

        Box::pin(async move {
            sqlx::query("DELETE FROM notifications WHERE id = $1")
                .bind(guid)
                .execute(&pool)
                .await
                .map_err(|e| {
                    log::error!("Failed to delete notification: {}", e);
                    JobSchedulerError::CantRemove
                })?;

            Ok(())
        })
    }
}

impl InitStore for PostgresNotificationStore {
    fn init(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<(), tokio_cron_scheduler::JobSchedulerError>> + Send>,
    > {
        Box::pin(async { Ok(()) })
    }

    fn inited(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<bool, tokio_cron_scheduler::JobSchedulerError>> + Send>,
    > {
        Box::pin(async { Ok(true) })
    }
}

impl NotificationStore for PostgresNotificationStore {
    fn delete_for_job(
        &mut self,
        job_id: sqlx::types::Uuid,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), JobSchedulerError>> + Send>> {
        let pool = self.pool.clone();

        Box::pin(async move {
            sqlx::query!("DELETE FROM notifications WHERE job_id = $1", job_id)
                .execute(&pool)
                .await
                .map_err(|e| {
                    log::error!("Failed to delete notifications for job: {}", e);
                    JobSchedulerError::CantRemove
                })?;

            Ok(())
        })
    }

    fn delete_notification_for_state(
        &mut self,
        notification_id: sqlx::types::Uuid,
        state: tokio_cron_scheduler::JobNotification,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, JobSchedulerError>> + Send>> {
        let pool = self.pool.clone();

        Box::pin(async move {
            let result = sqlx::query!(
                "DELETE FROM notification_states WHERE id = $1 AND state = $2",
                notification_id,
                state as i32
            )
            .execute(&pool)
            .await;

            match result {
                Ok(res) if res.rows_affected() > 0 => Ok(true),
                Ok(_) => Ok(false),
                Err(e) => {
                    log::error!("Failed to delete notification state: {}", e);
                    Err(JobSchedulerError::CantRemove)
                }
            }
        })
    }

    fn list_notification_guids_for_job_and_state(
        &mut self,
        job: tokio_cron_scheduler::job::JobId,
        state: tokio_cron_scheduler::JobNotification,
    ) -> std::pin::Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<tokio_cron_scheduler::job::NotificationId>,
                        JobSchedulerError,
                    >,
                > + Send,
        >,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            let notifications = sqlx::query!(
                "
                    SELECT DISTINCT states.id
                    FROM notification_states AS states
                    RIGHT JOIN notifications AS n ON n.id = states.id
                    WHERE n.job_id = $1 AND states.state = $2
                ",
                job.into(),
                state as i32
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to list notifications for job and state: {}", e);
                JobSchedulerError::FetchJob
            })?;

            Ok(notifications.into_iter().map(|n| n.id).collect::<Vec<_>>())
        })
    }

    fn list_notification_guids_for_job_id(
        &mut self,
        job_id: sqlx::types::Uuid,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<Vec<sqlx::types::Uuid>, JobSchedulerError>> + Send>,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            let notifications = sqlx::query!(
                "SELECT DISTINCT id FROM notifications WHERE job_id = $1",
                job_id
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to list notifications for job id: {}", e);
                JobSchedulerError::FetchJob
            })?;

            Ok(notifications.into_iter().map(|n| n.id).collect())
        })
    }
}
