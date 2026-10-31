// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real process/socket lifecycle and log disclosure regression tests.
#![allow(
    clippy::expect_used,
    reason = "Process tests fail with explicit setup context."
)]

use std::io::{BufRead, BufReader, Read};
use std::net::{SocketAddr, UdpSocket};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use cod4x_protocol::{Direction, Packet};
use serde_json::Value;

struct RunningServer(Child);

impl Drop for RunningServer {
    fn drop(&mut self) {
        // The test owns this exact child. Reap it even when an assertion fails.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cod4x-server"));
    command.arg("--json-logs");
    command
}

#[test]
fn real_udp_queries_have_structured_lifecycle_metrics_without_payload_disclosure() {
    let child = command()
        .args([
            "--query-only",
            "--bind",
            "127.0.0.1:0",
            "--max-datagrams",
            "2",
            "--run-for-ms",
            "3000",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Start development server");
    let mut server = RunningServer(child);
    let stderr = server.0.stderr.take().expect("Piped stderr");
    let mut logs = BufReader::new(stderr);
    let mut startup = String::new();
    logs.read_line(&mut startup).expect("Read startup record");
    let event: Value = serde_json::from_str(&startup).expect("Structured startup JSON");
    assert_eq!(event["fields"]["event"], "server.started");
    assert_eq!(event["fields"]["mode"], "query_only");
    let address: SocketAddr = event["fields"]["bind"]
        .as_str()
        .expect("Bound socket address")
        .parse()
        .expect("Valid socket address");
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Bind test client");
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("Client timeout");
    let secret = "do-not-log-payload-90210";
    for body in [
        format!("rcon {secret} command"),
        format!("getinfo {secret}"),
    ] {
        let mut bytes = vec![0; body.len() + 4];
        Packet::Connectionless(body.as_bytes())
            .encode(&mut bytes)
            .expect("Query envelope");
        socket.send_to(&bytes, address).expect("Send query");
    }
    let mut response = [0; 1400];
    let (length, peer) = socket
        .recv_from(&mut response)
        .expect("Receive real UDP response");
    assert_eq!(peer, address);
    let reply = Packet::parse(&response[..length], Direction::ServerToClient)
        .expect("Valid response envelope");
    assert!(matches!(reply, Packet::Connectionless(body) if body.starts_with(b"infoResponse\n")));
    assert!(
        response[..length]
            .windows(secret.len())
            .any(|part| part == secret.as_bytes())
    );
    assert!(
        server
            .0
            .wait()
            .expect("Wait for datagram-limit shutdown")
            .success()
    );
    let mut rest = String::new();
    logs.read_to_string(&mut rest)
        .expect("Read flushed final records");
    let combined = startup + &rest;
    assert!(
        !combined.contains(secret),
        "Wire secrets must not enter log fields"
    );
    assert!(
        !combined.contains("rcon"),
        "Remote command contents must not be logged"
    );
    let events: Vec<Value> = combined
        .lines()
        .map(|line| serde_json::from_str(line).expect("Every output record is valid JSON"))
        .collect();
    let metrics = events
        .iter()
        .find(|event| event["fields"]["event"] == "server.metrics")
        .expect("Final metrics record");
    assert_eq!(metrics["fields"]["received"], 2);
    assert_eq!(metrics["fields"]["replied"], 1);
    assert_eq!(metrics["fields"]["unsupported"], 1);
    let shutdown = events.last().expect("Shutdown event");
    assert_eq!(shutdown["fields"]["event"], "server.stopped");
    assert_eq!(shutdown["fields"]["reason"], "datagram_limit");
}

#[test]
fn cli_help_version_and_bad_configuration_have_explicit_exit_statuses() {
    assert!(
        command()
            .arg("--help")
            .output()
            .expect("Help output")
            .status
            .success()
    );
    let version = command().arg("--version").output().expect("Version output");
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains("query-only development"));
    for arguments in [
        vec![],
        vec!["--query-only", "--bind", "bad"],
        vec!["--query-only", "--run-for-ms", "0"],
        vec!["--query-only", "--log-level", "module=not_a_level"],
        vec!["--query-only", "--max-datagrams", "0"],
    ] {
        let result = command()
            .args(arguments)
            .output()
            .expect("Invalid configuration run");
        assert!(!result.status.success());
        assert!(!result.stderr.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn sigterm_produces_a_clean_structured_shutdown() {
    let child = command()
        .args([
            "--query-only",
            "--bind",
            "127.0.0.1:0",
            "--run-for-ms",
            "3000",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Start signal test server");
    let mut server = RunningServer(child);
    let mut logs = BufReader::new(server.0.stderr.take().expect("Piped stderr"));
    let mut startup = String::new();
    logs.read_line(&mut startup)
        .expect("Startup event before signal");
    assert!(
        Command::new("kill")
            .args(["-TERM", &server.0.id().to_string()])
            .status()
            .expect("Send SIGTERM to this test-owned child")
            .success()
    );
    assert!(server.0.wait().expect("Wait for signal shutdown").success());
    let mut rest = String::new();
    logs.read_to_string(&mut rest)
        .expect("Final flushed events");
    let events: Vec<Value> = rest
        .lines()
        .map(|line| serde_json::from_str(line).expect("Structured shutdown JSON"))
        .collect();
    let shutdown = events
        .iter()
        .find(|event| event["fields"]["event"] == "server.stopped")
        .expect("Stopped event");
    assert_eq!(shutdown["fields"]["reason"], "signal");
}
