#![cfg(windows)]
use enhancece_lib::{
    core::processing::{engine::Cancellation, errors::ErrorCode},
    engines::realesrgan::runner,
};
use std::{
    ffi::OsString,
    path::PathBuf,
    time::{Duration, Instant},
};
fn powershell() -> PathBuf {
    PathBuf::from(std::env::var_os("SystemRoot").unwrap())
        .join("System32/WindowsPowerShell/v1.0/powershell.exe")
}
#[test]
fn process_launch_and_exit_failures_are_typed() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        runner::run(
            &d.path().join("missing.exe"),
            d.path(),
            &[],
            &Cancellation::default(),
            Duration::from_secs(2)
        )
        .err()
        .unwrap()
        .code,
        ErrorCode::ProcessLaunchFailed
    );
    let args: Vec<OsString> = ["-NoProfile", "-NonInteractive", "-Command", "exit 7"]
        .iter()
        .map(OsString::from)
        .collect();
    let output = runner::run(
        &powershell(),
        d.path(),
        &args,
        &Cancellation::default(),
        Duration::from_secs(10),
    )
    .unwrap();
    assert_eq!(
        runner::require_success(output).unwrap_err().code,
        ErrorCode::InferenceFailed
    );
}
#[test]
fn cancelling_windows_job_terminates_parent_and_owned_descendant() {
    let d = tempfile::tempdir().unwrap();
    let script = d.path().join("fixture.ps1");
    let parent = d.path().join("parent.pid");
    let descendant = d.path().join("child.pid");
    std::fs::write(&script,r#"param([string]$ParentFile,[string]$ChildFile)
[IO.File]::WriteAllText($ParentFile, [string]$PID)
$child = Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -ArgumentList '-NoProfile -NonInteractive -Command Start-Sleep -Seconds 60' -WindowStyle Hidden -PassThru
[IO.File]::WriteAllText($ChildFile, [string]$child.Id)
Start-Sleep -Seconds 60
"#).unwrap();
    let args = vec![
        "-NoProfile".into(),
        "-NonInteractive".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        script.into_os_string(),
        parent.clone().into_os_string(),
        descendant.clone().into_os_string(),
    ];
    let cancel = Cancellation::default();
    let token = cancel.clone();
    let cwd = d.path().to_path_buf();
    let worker = std::thread::spawn(move || {
        runner::run(&powershell(), &cwd, &args, &token, Duration::from_secs(15))
    });
    let start = Instant::now();
    while !descendant.exists() && start.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(20));
    }
    cancel.cancel();
    assert_eq!(
        worker.join().unwrap().err().unwrap().code,
        ErrorCode::Cancelled
    );
    assert!(
        descendant.exists(),
        "Fixture must have created a descendant before cancellation"
    );
    for pid_file in [parent, descendant] {
        let pid: u32 = std::fs::read_to_string(pid_file).unwrap().parse().unwrap();
        unsafe {
            use windows_sys::Win32::{
                Foundation::CloseHandle,
                System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE},
            };
            let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
            if !handle.is_null() {
                assert_eq!(
                    WaitForSingleObject(handle, 5000),
                    0,
                    "Owned process remained alive"
                );
                CloseHandle(handle);
            }
        }
    }
}
