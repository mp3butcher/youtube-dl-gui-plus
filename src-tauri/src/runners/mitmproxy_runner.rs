use crate::paths::PathsManager;
use crate::runners::ytdlp_process::{
  configure_command, kill_platform_process, platform_process_from_child, PlatformProcess,
};
use crate::state::config_models::Config;
use crate::state::preferences_models::Preferences;
use crate::{SharedConfig, SharedPreferences};
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone)]
pub struct TerminatedPayload {
  pub code: Option<i32>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MitmproxyCommandEvent {
  Stderr(Vec<u8>),
  Stdout(Vec<u8>),
  Error(String),
  Terminated(TerminatedPayload),
}

#[derive(Debug)]
pub struct MitmproxyOutput {
  pub status: ExitStatus,
  pub stdout: Vec<u8>,
  pub stderr: Vec<u8>,
}

#[derive(Debug)]
pub struct MitmproxyChild {
  platform: PlatformProcess,
}

pub struct MitmproxyRunner<'a> {
  app: &'a AppHandle,
  cfg: Arc<Config>,
  prefs: Arc<Preferences>,
  args: Vec<String>,
  bin_dir: PathBuf,
}

impl<'a> MitmproxyRunner<'a> {
  pub fn new(app: &'a AppHandle) -> Self {
    let paths_manager = app.state::<PathsManager>();
    let bin_dir = paths_manager.bin_dir().clone();
    let args = vec![];//"--encoding".into(), "utf-8".into()];
    let cfg_handle = app.state::<SharedConfig>();
    let cfg = cfg_handle.load();
    let prefs_handle = app.state::<SharedPreferences>();
    let prefs = prefs_handle.load();

    Self {
      app,
      cfg,
      prefs,
      args,
      bin_dir,
    }
  }

  pub fn with_args<I, S>(mut self, args: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    self.args.extend(args.into_iter().map(Into::into));
    self.args.extend([ "-s".to_string() ]);
    let script_path = self.bin_dir.join("send_traffic_to_videodownloader.py");
    self.args.extend([ script_path.display().to_string() ]);
    self
  }

  pub async fn output(self) -> Result<MitmproxyOutput, String> {
    tracing::info!("Running command: mitmweb {}", self.args.join(" "));
    let mut command = self.build_command();

    configure_command(&mut command).map_err(|e| format!("mitmweb spawn setup failed: {e}"))?;

    tauri::async_runtime::spawn_blocking(move || {
      let output = command
        .output()
        .map_err(|e| format!("mitmweb failed to run: {e}"))?;
      Ok(MitmproxyOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
      })
    })
    .await
    .map_err(|e| format!("mitmweb task failed: {e}"))?
  }

  pub fn spawn(self) -> Result<(UnboundedReceiver<MitmproxyCommandEvent>, MitmproxyChild), String> {
    tracing::info!("Running command: mitmweb {}", self.args.join(" "));
    let mut command = self.build_command();
    command
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());

    configure_command(&mut command).map_err(|e| format!("mitmweb spawn setup failed: {e}"))?;

    let mut raw_child = command
      .spawn()
      .map_err(|e| format!("mitmweb failed to spawn: {e}"))?;
    let stdout = raw_child.stdout.take();
    let stderr = raw_child.stderr.take();

    let platform = match platform_process_from_child(&raw_child) {
      Ok(platform) => platform,
      Err(err) => {
        let _ = raw_child.kill();
        return Err(err);
      }
    };

    let (tx, rx) = unbounded_channel();

    if let Some(stdout) = stdout {
      spawn_reader(stdout, tx.clone(), true);
    }
    if let Some(stderr) = stderr {
      spawn_reader(stderr, tx.clone(), false);
    }

    let wait_tx = tx.clone();
    thread::spawn(move || {
      let status = raw_child.wait();
      match status {
        Ok(status) => {
          let payload = TerminatedPayload {
            code: status.code(),
          };
          let _ = wait_tx.send(MitmproxyCommandEvent::Terminated(payload));
        }
        Err(err) => {
          let _ = wait_tx.send(MitmproxyCommandEvent::Error(err.to_string()));
        }
      }
    });

    let child = MitmproxyChild { platform };

    Ok((rx, child))
  }

  fn build_command(&self) -> Command {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}{}{}", self.bin_dir.display(), separator, path_env);
    let mut command = Command::new("mitmweb");
    command.args(&self.args).env("PATH", new_path);
    command
  }
}

impl MitmproxyChild {
  pub fn kill_tree(&self) -> Result<(), String> {
    kill_platform_process(&self.platform);
    Ok(())
  }
}

fn spawn_reader<R: io::Read + Send + 'static>(
  reader: R,
  tx: UnboundedSender<MitmproxyCommandEvent>,
  is_stdout: bool,
) {
  thread::spawn(move || {
    let mut reader = BufReader::new(reader);
    let mut buf = Vec::new();
    loop {
      buf.clear();
      match reader.read_until(b'\n', &mut buf) {
        Ok(0) => break,
        Ok(_) => {
          let out = std::mem::take(&mut buf);
          let event = if is_stdout {
            MitmproxyCommandEvent::Stdout(out)
          } else {
            MitmproxyCommandEvent::Stderr(out)
          };
          let _ = tx.send(event);
        }
        Err(err) => {
          let _ = tx.send(MitmproxyCommandEvent::Error(err.to_string()));
          break;
        }
      }
    }
  });
}


#[cfg(test)]
mod tests {
  use super::build_subtitle_args;
  use crate::state::config_models::SubtitleSettings;
}