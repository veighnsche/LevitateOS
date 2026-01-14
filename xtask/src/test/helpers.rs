use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// Represents a running test VM instance
pub struct TestVm {
    pub process: Option<Child>,
    pub output_file: String,
    pub qmp_socket: String,
    pub serial_socket: String,
    pub test_name: Option<String>,
}

impl TestVm {
    /// Start VM with Alpine ISO
    pub fn start_alpine(iso_path: &str) -> Result<Self> {
        if !Path::new(iso_path).exists() {
            bail!("Alpine ISO not found: {}", iso_path);
        }

        // Use unique socket paths for parallel test isolation
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let output_file = format!("build/test-alpine-{}.log", timestamp);
        let qmp_socket = format!("/tmp/levitate-test-qmp-{}.sock", timestamp);
        let serial_socket = format!("/tmp/levitate-test-serial-{}.sock", timestamp);

        // Clean up any stale sockets
        for socket in [&qmp_socket, &serial_socket] {
            if Path::new(socket).exists() {
                std::fs::remove_file(socket)?;
            }
        }

        // Ensure build directory exists
        std::fs::create_dir_all("build")?;
        std::fs::write(&output_file, "")?;

        // Start QEMU with Alpine ISO
        let child = Command::new("qemu-system-x86_64")
            .args(&[
                "-cdrom",
                iso_path,
                "-boot",
                "d", // Boot from CD-ROM
                "-m",
                "512M",
                "-display",
                "none",
                "-no-reboot",
                "-chardev",
                &format!(
                    "socket,id=serial0,path={},server=on,wait=off,logfile={}",
                    serial_socket, output_file
                ),
                "-serial",
                "chardev:serial0",
                "-qmp",
                &format!("unix:{},server,nowait", qmp_socket),
            ])
            .spawn()
            .context("Failed to start QEMU with Alpine ISO")?;

        // Wait for QEMU to initialize sockets
        std::thread::sleep(Duration::from_millis(500));

        Ok(TestVm {
            process: Some(child),
            output_file,
            qmp_socket,
            serial_socket,
            test_name: None,
        })
    }

    /// Wait for pattern to appear in output log
    pub fn wait_for_pattern(&self, pattern: &str, timeout_secs: u64) -> Result<bool> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        let regex = regex::Regex::new(pattern)?;

        loop {
            if Instant::now() > deadline {
                return Ok(false);
            }

            if Path::new(&self.output_file).exists() {
                if let Ok(contents) = std::fs::read_to_string(&self.output_file) {
                    if regex.is_match(&contents) {
                        return Ok(true);
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(200));
        }
    }

    /// Read output log contents
    pub fn read_output(&self) -> Result<String> {
        std::fs::read_to_string(&self.output_file).context("Failed to read output log")
    }

    /// Send text to serial console (with carriage return)
    pub fn send_line(&mut self, text: &str) -> Result<()> {
        use std::io::Write;
        use std::os::unix::net::UnixStream;

        let mut stream = UnixStream::connect(&self.serial_socket)
            .context("Failed to connect to serial socket")?;

        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        // Terminal expects carriage return, not just newline
        let data = format!("{}\r", text);
        stream
            .write_all(data.as_bytes())
            .context("Failed to write to serial")?;
        stream.flush()?;

        // Keep connection open briefly to ensure delivery
        std::thread::sleep(Duration::from_millis(100));

        Ok(())
    }

    /// Set test name for artifact generation
    pub fn set_test_name(&mut self, name: &str) {
        self.test_name = Some(name.to_string());
    }

    /// Save output as artifact file for manual validation
    pub fn save_artifact(&self) -> Result<String> {
        // Ensure artifacts directory exists
        std::fs::create_dir_all("xtask/src/test/artifacts")?;

        // Generate artifact filename
        let test_name = self.test_name.as_deref().unwrap_or("unknown");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let artifact_file = format!("xtask/src/test/artifacts/{}_{}.txt", test_name, timestamp);

        // Read the output and save it
        let output = std::fs::read_to_string(&self.output_file)
            .context("Failed to read output log")?;

        std::fs::write(&artifact_file, &output)
            .context("Failed to write artifact file")?;

        Ok(artifact_file)
    }

    /// Connect to QMP socket
    pub fn qmp_client(&self) -> Result<crate::vm::qmp::QmpClient> {
        crate::vm::qmp::QmpClient::connect(&self.qmp_socket)
    }

    /// Stop VM gracefully or forcefully
    pub fn stop(&mut self) -> Result<()> {
        // Try QMP quit first
        if let Ok(mut client) = self.qmp_client() {
            if client.handshake().is_ok() {
                let _ = client.quit();
                std::thread::sleep(Duration::from_millis(500));
            }
        }

        // Check if still alive
        if let Some(mut process) = self.process.take() {
            match process.try_wait()? {
                Some(_) => {
                    // Already exited
                }
                None => {
                    // Force kill
                    let _ = process.kill();
                    let _ = process.wait();
                }
            }
        }

        // Clean up sockets
        for socket in [&self.qmp_socket, &self.serial_socket] {
            if Path::new(socket).exists() {
                let _ = std::fs::remove_file(socket);
            }
        }

        // Clear builder's session file to avoid interfering with subsequent VM starts
        // This is critical for test isolation - builder stores session state at build/.vm-session.json
        let session_file = "build/.vm-session.json";
        if Path::new(session_file).exists() {
            let _ = std::fs::remove_file(session_file);
        }

        Ok(())
    }
}

impl Drop for TestVm {
    fn drop(&mut self) {
        // Emergency cleanup on drop
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
        let _ = std::fs::remove_file(&self.qmp_socket);
        let _ = std::fs::remove_file(&self.serial_socket);
        // Always clear session file to prevent test isolation issues
        let _ = std::fs::remove_file("build/.vm-session.json");
    }
}
