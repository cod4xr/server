// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real UDP cookie exchange, explicit refusal, aggregate metrics, and log redaction.
#![allow(
    clippy::expect_used,
    reason = "Tests own bounded child processes and synthetic requests."
)]

use std::io::{BufRead, BufReader, Read};
use std::net::{SocketAddr, UdpSocket};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use cod4x_protocol::{CommandLine, Direction, Packet};
use serde_json::Value;

struct ProbeProcess(Child);
impl Drop for ProbeProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn send(socket: &UdpSocket, address: SocketAddr, body: &[u8]) {
    let mut bytes = vec![0; body.len() + 4];
    Packet::Connectionless(body)
        .encode(&mut bytes)
        .expect("Datagram");
    socket.send_to(&bytes, address).expect("Send");
}

#[test]
fn real_handshake_probe_verifies_a_return_path_and_refuses_gameplay_without_disclosure()
-> Result<(), String> {
    let child = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
        .args([
            "--handshake-probe",
            "--json-logs",
            "--bind",
            "127.0.0.1:0",
            "--run-for-ms",
            "3000",
            "--max-datagrams",
            "3",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Probe process");
    let mut child = ProbeProcess(child);
    let mut logs = BufReader::new(child.0.stderr.take().expect("Logs"));
    let mut startup = String::new();
    logs.read_line(&mut startup).expect("Startup record");
    let event: Value = serde_json::from_str(&startup).expect("JSON");
    assert_eq!(event["fields"]["mode"], "handshake_probe");
    let address: SocketAddr = event["fields"]["bind"]
        .as_str()
        .expect("Bind")
        .parse()
        .expect("Address");
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Client");
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("Timeout");
    send(
        &socket,
        address,
        b"getchallenge 195487653 \"synthetic-guid-secret\"",
    );
    let mut response = [0; 1024];
    let (length, _) = socket.recv_from(&mut response).expect("Challenge response");
    let Packet::Connectionless(body) =
        Packet::parse(&response[..length], Direction::ServerToClient).expect("Packet")
    else {
        return Err("Expected connectionless response".into());
    };
    let line = CommandLine::parse(body).expect("Response line");
    let fields: Vec<&[u8]> = line.tokens().collect();
    assert_eq!(fields[0], b"challengeResponse");
    assert_eq!(fields[2], b"195487653");
    assert_eq!(&fields[3..], [b"0".as_slice(), b"xproto", b"18"]);
    let challenge: i32 = std::str::from_utf8(fields[1])
        .expect("Cookie ASCII")
        .parse()
        .expect("Cookie");
    let invalid = challenge ^ 1;
    let connect = |cookie: i32| {
        format!(
            "connect \"\\protocol\\21\\challenge\\{cookie}\\qport\\-1\\password\\synthetic-password-secret\""
        )
    };
    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("Short timeout");
    send(&socket, address, connect(invalid).as_bytes());
    assert!(
        socket.recv_from(&mut response).is_err(),
        "Bad cookies get no reply"
    );
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("Reply timeout");
    send(&socket, address, connect(challenge).as_bytes());
    let (length, _) = socket.recv_from(&mut response).expect("Explicit refusal");
    assert!(
        matches!(Packet::parse(&response[..length], Direction::ServerToClient),
        Ok(Packet::Connectionless(body)) if body.starts_with(b"error\n") &&
            body.windows(b"not ready".len()).any(|part| part == b"not ready"))
    );
    assert!(child.0.wait().expect("Reap probe").success());
    let mut rest = String::new();
    logs.read_to_string(&mut rest).expect("Flushed records");
    assert_private_logs_and_metrics(&startup, &rest);
    Ok(())
}

fn assert_private_logs_and_metrics(startup: &str, rest: &str) {
    let combined = format!("{startup}{rest}");
    for private in [
        "synthetic-guid-secret",
        "synthetic-password-secret",
        "195487653",
    ] {
        assert!(!combined.contains(private));
    }
    let events: Vec<Value> = rest
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON"))
        .collect();
    let metrics = events
        .iter()
        .find(|event| event["fields"]["event"] == "server.metrics")
        .expect("Final counters");
    assert_eq!(metrics["fields"]["received"], 3);
    assert_eq!(metrics["fields"]["replied"], 2);
    assert_eq!(metrics["fields"]["invalid_challenge"], 1);
    assert_eq!(metrics["fields"]["probe_challenges"], 1);
    assert_eq!(metrics["fields"]["probe_verified_rejections"], 1);
}

#[test]
fn probe_mode_cannot_be_combined_with_other_runtime_modes() {
    for arguments in [
        vec!["--handshake-probe", "--query-only"],
        vec!["--handshake-probe", "--verify-iwd", "missing"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4x-server"))
            .args(arguments)
            .output()
            .expect("Invalid mode selection");
        assert!(!output.status.success());
    }
}
