use crate::core::processing::{
    engine::Cancellation,
    errors::{err, ErrorCode, Result},
};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    ffi::OsString,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
pub struct ProcessOutput {
    pub success: bool,
    pub text: String,
}
fn drain(mut stream: impl Read) -> String {
    let mut tail = Vec::new();
    let mut buf = [0u8; 4096];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        tail.extend_from_slice(&buf[..n]);
        if tail.len() > 16384 {
            tail.drain(..tail.len() - 16384);
        }
    }
    String::from_utf8_lossy(&tail).into_owned()
}
pub fn run(
    exe: &Path,
    cwd: &Path,
    args: &[OsString],
    cancel: &Cancellation,
    timeout: Duration,
) -> Result<ProcessOutput> {
    cancel.check()?;
    let mut command = Command::new(exe);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Keep Windows loader/driver environment; remove application injection variables.
    for key in [
        "PYTHONPATH",
        "PYTHONHOME",
        "NODE_OPTIONS",
        "LD_PRELOAD",
        "VK_INSTANCE_LAYERS",
        "VK_LAYER_PATH",
    ] {
        command.env_remove(key);
    }
    #[cfg(windows)]
    command.creation_flags(0x08000000 | 0x00000004); // NO_WINDOW | SUSPENDED
    let mut child = command
        .spawn()
        .map_err(|_| err(ErrorCode::ProcessLaunchFailed))?;
    #[cfg(windows)]
    let job = super::containment::Job::attach_and_resume(&mut child)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| err(ErrorCode::ProcessLaunchFailed))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| err(ErrorCode::ProcessLaunchFailed))?;
    let out_thread = thread::spawn(move || drain(stdout));
    let err_thread = thread::spawn(move || drain(stderr));
    let start = Instant::now();
    let result = loop {
        if let Err(e) = cancel.check() {
            break Err(e);
        }
        if start.elapsed() > timeout {
            break Err(err(ErrorCode::TimedOut));
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                log::info!(
                    "event=engine_exit success={} exit_code={:?}",
                    status.success(),
                    status.code()
                );
                break Ok(status.success());
            }
            Ok(None) => thread::sleep(Duration::from_millis(40)),
            Err(_) => break Err(err(ErrorCode::InferenceFailed)),
        }
    };
    #[cfg(windows)]
    {
        job.terminate();
        drop(job);
    }
    if result.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    let stdout = out_thread.join().unwrap_or_default();
    let stderr = err_thread.join().unwrap_or_default();
    result.map(|success| ProcessOutput {
        success,
        text: format!("{stdout}\n{stderr}"),
    })
}
pub fn require_success(output: ProcessOutput) -> Result<()> {
    if output.success {
        Ok(())
    } else {
        Err(err(ErrorCode::InferenceFailed))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_exit_never_means_success() {
        assert_eq!(
            require_success(ProcessOutput {
                success: false,
                text: String::new()
            })
            .unwrap_err()
            .code,
            ErrorCode::InferenceFailed
        );
    }
    #[test]
    fn output_capture_is_bounded() {
        assert!(drain(std::io::Cursor::new(vec![b'x'; 100_000])).len() <= 16384);
    }
}
