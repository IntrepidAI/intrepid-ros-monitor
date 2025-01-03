use std::ffi::OsString;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use tokio::sync::broadcast::error::RecvError;

pub mod types;
pub mod state;

#[derive(Default, Clone)]
pub struct RosMonitor {
    state: Arc<Mutex<state::RosState>>,
    channel: Arc<Mutex<Option<tokio::sync::broadcast::Sender<types::DiscoveryEvent>>>>,
    task: Option<Arc<AbortJoinHandle>>,
}

struct AbortJoinHandle(pub tokio::task::JoinHandle<()>);

impl Drop for AbortJoinHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl RosMonitor {
    pub fn new(command: impl Into<OsString>) -> Self {
        use std::process::Stdio;
        use tokio::io::AsyncReadExt;
        use tokio::process::Command;

        let state_arc = Arc::new(Mutex::new(state::RosState::default()));
        let (channel, _rx) = tokio::sync::broadcast::channel(128);
        let command = command.into();
        let state_arc_ = state_arc.clone();
        let channel_arc = Arc::new(Mutex::new(Some(channel.clone())));
        let channel_arc_ = channel_arc.clone();

        let task = AbortJoinHandle(tokio::spawn(async move {
            let error: Result<(), RosMonitorError> = async {
                let channel_ = {
                    let channel = channel_arc_.lock().unwrap();
                    channel.as_ref().unwrap().clone()
                };

                let mut byte_buffer = Vec::new();
                let mut bitcode_buffer = bitcode::Buffer::new();

                loop {
                    let started_at = std::time::Instant::now();
                    let mut child = Command::new(&command)
                        .arg("-f")
                        .arg("bitcode")
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()?;

                    let stdout = child.stdout.take().ok_or(RosMonitorError::PipeError)?;
                    let mut reader = tokio::io::BufReader::new(stdout);

                    loop {
                        let Ok(size) = reader.read_u32_le().await else { break };
                        byte_buffer.resize(size as usize, 0);
                        let Ok(_) = reader.read_exact(&mut byte_buffer).await else { break };
                        let Ok(event) = bitcode_buffer.decode::<types::DiscoveryEventWrapper>(&byte_buffer) else { break };

                        {
                            let mut state = state_arc_.lock().unwrap();
                            let mut new_state = state.clone();
                            new_state.update(event.event);
                            for event in new_state.changes(&state) {
                                let _ = channel_.send(event);
                                // yield event;
                            }
                            *state = new_state;
                        }
                    }

                    {
                        let mut state = state_arc_.lock().unwrap();
                        let new_state = state::RosState::default();
                        for event in new_state.changes(&state) {
                            let _ = channel_.send(event);
                            // yield event;
                        }
                        *state = new_state;
                    }

                    let elapsed = started_at.elapsed();
                    if elapsed.as_millis() < 2000 {
                        let mut stderr = child.stderr.take().ok_or(RosMonitorError::PipeError)?;
                        let mut result = String::new();
                        stderr.read_to_string(&mut result).await?;
                        let status = child.try_wait()?;
                        Err(RosMonitorError::ProcessExited {
                            status: status.unwrap_or_default(),
                            stderr: result,
                        })?;
                    }
                }
            }.await;

            if let Err(error) = error {
                let mut spawn_error = false;
                if let RosMonitorError::SpawnError(err) = &error {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        spawn_error = true;
                        log::warn!("ROS discovery is not available on this platform");
                    }
                }

                if !spawn_error {
                    let mut reason = String::new();
                    if error.to_string().contains("error while loading shared libraries") {
                        reason.push_str("Please make sure that ROS is sourced, and try at least ROS jazzy.");
                    }
                    log::error!("ROS discovery is not available:\n{}{}", error, reason);
                }
            }

            channel_arc_.lock().unwrap().take().unwrap();
        }));

        Self {
            state: state_arc,
            channel: channel_arc,
            task: Some(Arc::new(task)),
        }
    }

    pub fn subscribe(&self) -> Result<impl futures::TryStream<Item = Result<types::DiscoveryEvent, RecvError>>, RecvError> {
        let is_finished = self.task.as_ref().map(|task| task.0.is_finished()).unwrap_or(true);

        if is_finished {
            return Err(RecvError::Closed);
        }

        let (initial, receiver) = {
            let channel = self.channel.lock().unwrap();
            let state = self.state.lock().unwrap();
            (
                state.changes(&Default::default()),
                channel.as_ref().map(|channel| channel.subscribe())
            )
        };

        Ok(async_stream::try_stream! {
            let mut receiver = receiver.ok_or(RecvError::Closed)?;
            for event in initial {
                yield event;
            }
            loop {
                let event = receiver.recv().await?;
                yield event;
            }
        })
    }
}

#[derive(Debug, Error)]
enum RosMonitorError {
    #[error("unable to spawn process: {0}")]
    SpawnError(#[from] tokio::io::Error),
    #[error("unable to read stdout/stderr")]
    PipeError,
    #[error("process exited: {status}\n{stderr}")]
    ProcessExited {
        status: std::process::ExitStatus,
        stderr: String,
    },
}
