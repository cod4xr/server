// SPDX-License-Identifier: AGPL-3.0-or-later
//! Challenge datagrams from the complete original formatting/send path.
#![allow(
    clippy::expect_used,
    reason = "Tests compare checked-in original C bytes."
)]

use cod4x_protocol::connection::ChallengeResponse;

#[test]
fn signed_boundaries_nonce_echo_and_exact_datagram_bytes_match_original_server() {
    let mut count = 0;
    for row in include_str!("fixtures/wire_vectors.txt")
        .lines()
        .filter_map(|row| row.strip_prefix("challenge "))
    {
        let fields: Vec<&str> = row.split_whitespace().collect();
        assert_eq!(fields.len(), 3);
        let response = ChallengeResponse {
            server_challenge: fields[0].parse().expect("Cookie"),
            client_challenge: fields[1].parse().expect("Echo"),
        };
        let expected: Vec<u8> = fields[2]
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("Hex ASCII"), 16).expect("Byte")
            })
            .collect();
        for capacity in 0..expected.len() {
            let mut output = vec![0xa5; capacity];
            assert!(response.encode(&mut output).is_err());
            assert_eq!(output, vec![0xa5; capacity]);
        }
        let mut output = [0xa5; 128];
        assert_eq!(
            response.encode(&mut output).expect("Response"),
            expected.len()
        );
        assert_eq!(&output[..expected.len()], expected);
        assert!(output[expected.len()..].iter().all(|byte| *byte == 0xa5));
        assert_eq!(format!("{response:?}"), "ChallengeResponse { .. }");
        count += 1;
    }
    assert_eq!(count, 40);
}
