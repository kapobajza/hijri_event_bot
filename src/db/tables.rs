use sqlx::types::uuid;
use tokio_cron_scheduler::job::job_data_prost::{
    CronJob, JobStoredData, JobType, NonCronJob, job_stored_data,
};

#[derive(Debug)]
pub struct Job {
    pub id: uuid::Uuid,
    pub last_updated: Option<i64>,
    pub next_tick: Option<i64>,
    pub last_tick: Option<i64>,
    pub job_type: i32,
    pub count: Option<i32>,
    pub ran: Option<bool>,
    pub stopped: Option<bool>,
    pub time_offset_seconds: Option<i32>,
    pub extra: Option<Vec<u8>>,
    pub schedule: Option<String>,
    pub repeated_every: Option<i32>,
    pub repeating: Option<bool>,
}

impl From<JobStoredData> for Job {
    fn from(data: JobStoredData) -> Self {
        let repeated_every = data.repeated_every();
        let repeating = repeated_every.as_ref().is_some();

        Job {
            id: {
                let prost_uuid = data.id.expect("JobStoredData.id is None");
                let mut bytes = [0u8; 16];
                bytes[0..8].copy_from_slice(&prost_uuid.id1.to_be_bytes());
                bytes[8..16].copy_from_slice(&prost_uuid.id2.to_be_bytes());

                uuid::Uuid::from_bytes(bytes)
            },
            last_updated: data.last_updated.map(|dt| dt as i64),
            next_tick: Some(data.next_tick as i64),
            last_tick: data.last_tick.map(|lt| lt as i64),
            job_type: data.job_type,
            count: Some(data.count as i32),
            ran: Some(data.ran),
            stopped: Some(data.stopped),
            time_offset_seconds: Some(data.time_offset_seconds),
            extra: Some(data.extra.clone()),
            schedule: data.schedule().map(|s| s.to_string()),
            repeated_every: repeated_every.map(|i| i as i32),
            repeating: Some(repeating),
        }
    }
}

impl From<Job> for Option<JobStoredData> {
    fn from(job: Job) -> Self {
        let job_type = JobType::try_from(job.job_type);

        match job_type {
            Ok(JobType::Cron) => {
                let job_stored_data = JobStoredData {
                    id: Some(job.id.into()),
                    last_updated: Some(job.last_updated.map(|dt| dt as u64).unwrap_or(0)),
                    next_tick: job.next_tick.map(|nt| nt as u64).unwrap_or(0),
                    last_tick: Some(job.last_tick.map(|lt| lt as u64).unwrap_or(0)),
                    job_type: job.job_type,
                    count: job.count.map(|c| c as u32).unwrap_or(0),
                    ran: job.ran.unwrap_or(false),
                    stopped: job.stopped.unwrap_or(false),
                    time_offset_seconds: job.time_offset_seconds.unwrap_or(0),
                    extra: job.extra.unwrap_or_default(),
                    job: Some(job_stored_data::Job::CronJob(CronJob {
                        schedule: job.schedule.expect("Schedule should not be None"),
                    })),
                };
                Some(job_stored_data)
            }
            Ok(JobType::OneShot) | Ok(JobType::Repeated) => {
                let job_stored_data = JobStoredData {
                    id: Some(job.id.into()),
                    last_updated: job.last_updated.map(|dt| dt as u64),
                    next_tick: job.next_tick.map(|nt| nt as u64).unwrap_or_default(),
                    last_tick: job.last_tick.map(|lt| lt as u64),
                    job_type: job.job_type,
                    count: job.count.map(|c| c as u32).unwrap_or_default(),
                    ran: job.ran.unwrap_or_default(),
                    stopped: job.stopped.unwrap_or_default(),
                    time_offset_seconds: job.time_offset_seconds.unwrap_or_default(),
                    extra: job.extra.unwrap_or_default(),
                    job: Some(job_stored_data::Job::NonCronJob(NonCronJob {
                        repeating: job.repeating.unwrap_or_default(),
                        repeated_every: job.repeated_every.map(|re| re as u64).unwrap_or_default(),
                    })),
                };
                Some(job_stored_data)
            }
            Err(error) => {
                log::error!("Failed to convert job type: {}", error);
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct NotificationState {
    pub state: i32,
}

#[derive(sqlx::Type)]
pub enum JobExtensionType {
    WhiteDaysMessage = 1,
}

impl From<i32> for JobExtensionType {
    fn from(value: i32) -> Self {
        match value {
            1 => JobExtensionType::WhiteDaysMessage,
            _ => panic!("Unknown JobExtensionType value: {}", value),
        }
    }
}
