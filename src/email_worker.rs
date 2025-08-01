use passwordless_auth::{
    config::Config,
    db::Database,
    email::Emailer,
    email_queue::{EmailQueue, EmailTask, QueueError},
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let cfg = Config::load("config.toml")?;
    let db = Database::open(&cfg.database_path)?;
    // run migrations if needed
    let migration_sql = std::fs::read_to_string("migrations/init.sql")?;
    db.migrate(&migration_sql)?;
    let extra_sql = std::fs::read_to_string("migrations/002_email_queue.sql")?;
    db.migrate(&extra_sql)?;

    let emailer = Emailer::new(&cfg);
    let db = Arc::new(db);
    loop {
        match EmailQueue::fetch_due(&db, 10) {
            Ok(tasks) => {
                for t in tasks {
                    let db_clone = db.clone();
                    let emailer_clone = emailer.clone();
                    tokio::spawn(async move {
                        if let Err(e) = process(&db_clone, &emailer_clone, &t).await {
                            error!("error processing email {}: {}", t.id, e);
                        }
                    });
                }
            }
            Err(e) => error!("failed to fetch due emails: {}", e),
        }
        sleep(Duration::from_secs(5)).await;
    }
}

async fn process(db: &Database, emailer: &Emailer, task: &EmailTask) -> Result<(), anyhow::Error> {
    EmailQueue::mark_sending(db, &task.id)?;
    let send_result = emailer.send_magic_link(&task.to_email, &format!("{}", task.body_text));
    match send_result {
        Ok(_) => {
            info!("sent queued email to {}", task.to_email);
            EmailQueue::mark_sent(db, &task.id)?;
        }
        Err(e) => {
            error!("sending failed: {}", e);
            let next_attempts = task.attempts + 1;
            EmailQueue::mark_failed(db, &task.id, &format!("{}", e), next_attempts)?;
        }
    }
    Ok(())
}
