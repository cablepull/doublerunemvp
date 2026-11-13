use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub(crate) fn run(
    artifact: &Path,
    args: &[String],
    prefix: &Path,
    seconds: u64,
) -> Result<(), String> {
    let mut cmd = Command::new(artifact);
    cmd.args(args).env_clear().stdin(Stdio::null());
    // Rendering only. No model/API/SSH credentials or general user environment.
    for name in [
        "PATH",
        "SystemRoot",
        "WINDIR",
        "TMPDIR",
        "TEMP",
        "XDG_RUNTIME_DIR",
        "VK_ICD_FILENAMES",
        "VK_DRIVER_FILES",
        "LD_LIBRARY_PATH",
        "LIBGL_ALWAYS_SOFTWARE",
    ] {
        if let Some(value) = std::env::var_os(name) {
            cmd.env(name, value);
        }
    }
    cmd.stdout(File::create(prefix.with_extension("stdout.log")).map_err(|e| e.to_string())?);
    cmd.stderr(File::create(prefix.with_extension("stderr.log")).map_err(|e| e.to_string())?);
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let start = Instant::now();
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) if status.success() => return Ok(()),
            Some(status) => return Err(format!("renderer exited {status}; see stderr.log")),
            None if start.elapsed() >= Duration::from_secs(seconds) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("renderer timed out after {seconds}s"));
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn enforces_timeout_on_an_actual_child() {
        let d = tempfile::tempdir().unwrap();
        let now = Instant::now();
        assert!(run(
            Path::new("/bin/sleep"),
            &["10".into()],
            &d.path().join("run"),
            1
        )
        .unwrap_err()
        .contains("timed out"));
        assert!(now.elapsed() < Duration::from_secs(3));
    }
}
