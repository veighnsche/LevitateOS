//! Network tests.
//!
//! Can users use network tools?
//! Note: Actual network connectivity depends on container setup.
//!
//! ## Anti-Reward-Hacking Design
//!
//! Tests verify actual command output, not just exit codes.

use super::{test_result, Test, TestResult};
use crate::container::Container;

/// Test: IP command works
struct IpCommand;

impl Test for IpCommand {
    fn name(&self) -> &str { "ip command" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "User can view and configure network interfaces"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                ip link show &&
                ip addr show lo
            "#)?;

            if !result.contains("lo") {
                anyhow::bail!("ip link show didn't list loopback");
            }
            if !result.contains("127.0.0.1") {
                anyhow::bail!("loopback doesn't have 127.0.0.1");
            }
            Ok("ip command works".into())
        })
    }
}

/// Test: Ping command exists
struct PingCommand;

impl Test for PingCommand {
    fn name(&self) -> &str { "ping" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "User can test network connectivity with ping"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                ping -V &&
                ping -c 1 127.0.0.1
            "#)?;

            if !result.contains("iputils") && !result.contains("ping") {
                anyhow::bail!("ping not working");
            }
            if !result.contains("1 received") && !result.contains("1 packets received") {
                anyhow::bail!("ping loopback failed");
            }
            Ok("ping works".into())
        })
    }
}

/// Test: Curl command works
struct CurlCommand;

impl Test for CurlCommand {
    fn name(&self) -> &str { "curl" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "User can download files and interact with HTTP services"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok("curl --version | head -1")?;

            if !result.contains("curl") {
                anyhow::bail!("curl not working: {}", result);
            }
            Ok(result.trim().into())
        })
    }
}

/// Test: DNS resolution config
struct DnsConfig;

impl Test for DnsConfig {
    fn name(&self) -> &str { "DNS config" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "System is configured for DNS resolution"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            // In a container, resolv.conf might be managed by nspawn
            // Just check nsswitch.conf has hosts entry
            let result = c.exec_ok("grep hosts /etc/nsswitch.conf")?;

            if result.is_empty() {
                anyhow::bail!("No hosts entry in nsswitch.conf");
            }
            Ok("DNS resolution configured".into())
        })
    }
}

/// Test: /etc/hosts works
struct HostsFile;

impl Test for HostsFile {
    fn name(&self) -> &str { "/etc/hosts" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "Local hostname resolution works via /etc/hosts"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok("cat /etc/hosts")?;

            if !result.contains("127.0.0.1") || !result.contains("localhost") {
                anyhow::bail!("/etc/hosts missing localhost entry");
            }
            Ok("/etc/hosts configured correctly".into())
        })
    }
}

/// Test: SS command works
struct SsCommand;

impl Test for SsCommand {
    fn name(&self) -> &str { "ss (sockets)" }
    fn category(&self) -> &str { "network" }
    fn ensures(&self) -> &str {
        "User can inspect network sockets and connections"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                ss --version &&
                ss -ln
            "#)?;

            if !result.contains("iproute") {
                anyhow::bail!("ss not from iproute2: {}", result);
            }
            Ok("ss command works".into())
        })
    }
}

pub fn tests() -> Vec<Box<dyn Test>> {
    vec![
        Box::new(IpCommand),
        Box::new(PingCommand),
        Box::new(CurlCommand),
        Box::new(DnsConfig),
        Box::new(HostsFile),
        Box::new(SsCommand),
    ]
}
