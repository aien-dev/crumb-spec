pub mod ledger;

pub use ledger::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crumb_ledger_cryptographic_hash_chaining() {
        let mut ledger = CrumbLedger::new();
        assert!(ledger.is_empty());

        let base_time: u64 = 1_700_000_000;
        let actions = [
            (CrumbAction::Create, "crates/core/src/lib.rs", "Init core crate"),
            (CrumbAction::Modify, "crates/core/src/lib.rs", "Implement protocol handler"),
            (CrumbAction::Test, "crates/core/tests/test.rs", "Add unit tests"),
            (CrumbAction::Build, "target/release/core", "Compile release binary"),
            (CrumbAction::Audit, "crates/core/src/lib.rs", "Security audit"),
            (CrumbAction::Modify, "Cargo.toml", "Bump version to 1.0.0"),
            (CrumbAction::Test, "crates/core/tests/integration.rs", "Run integration tests"),
            (CrumbAction::Create, "README.md", "Document architecture"),
        ];

        for (i, (action, target, intent)) in actions.iter().enumerate() {
            let payload = format!("payload-content-step-{}", i);
            let timestamp = base_time + (i as u64 * 10);
            ledger
                .append(
                    "agent-atlas-prime".into(),
                    action.clone(),
                    target.to_string(),
                    intent.to_string(),
                    payload.as_bytes(),
                    timestamp,
                )
                .expect("append should succeed");
        }

        assert_eq!(ledger.len(), 8);
        assert!(ledger.verify_chain().is_ok(), "Untampered chain must verify successfully");

        // Scenario 1: Tamper with genesis event payload
        {
            let mut tampered = ledger.clone();
            tampered.events[0].intent = "Malicious backdoor inserted".to_string();
            let err = tampered.verify_chain();
            assert!(
                matches!(err, Err(LedgerError::HashMismatch { index: 0, .. })),
                "Tampering genesis event must trigger hash mismatch at index 0"
            );

            // If attacker updates event 0's hash to match tampered payload, event 1 fails parent check
            tampered.events[0].hash = LedgerEvent::compute_hash(
                tampered.events[0].index,
                tampered.events[0].timestamp,
                &tampered.events[0].agent,
                &tampered.events[0].action,
                &tampered.events[0].target,
                &tampered.events[0].intent,
                &tampered.events[0].payload_hash,
                &tampered.events[0].parent_hash,
            );
            let err2 = tampered.verify_chain();
            assert!(
                matches!(err2, Err(LedgerError::ParentHashMismatch { index: 1, .. })),
                "Downstream event 1 must reject modified genesis parent hash"
            );
        }

        // Scenario 2: Tamper with historical event in the middle of the chain (index 3)
        {
            let mut tampered = ledger.clone();
            tampered.events[3].target = "crates/tampered/path.rs".to_string();
            let err = tampered.verify_chain();
            assert!(
                matches!(err, Err(LedgerError::HashMismatch { index: 3, .. })),
                "Tampering historical event 3 must trigger hash mismatch at index 3"
            );

            // Recompute event 3 hash to match tampered data
            tampered.events[3].hash = LedgerEvent::compute_hash(
                tampered.events[3].index,
                tampered.events[3].timestamp,
                &tampered.events[3].agent,
                &tampered.events[3].action,
                &tampered.events[3].target,
                &tampered.events[3].intent,
                &tampered.events[3].payload_hash,
                &tampered.events[3].parent_hash,
            );
            // Event 4 parent_hash still expects original event 3 hash
            let err_downstream = tampered.verify_chain();
            assert!(
                matches!(err_downstream, Err(LedgerError::ParentHashMismatch { index: 4, .. })),
                "Downstream event 4 must fail verification due to broken parent hash link"
            );
        }

        // Scenario 3: Single bit flip in recorded hash
        {
            let mut tampered = ledger.clone();
            tampered.events[5].hash[0] ^= 0x01;
            let err = tampered.verify_chain();
            assert!(
                matches!(err, Err(LedgerError::HashMismatch { index: 5, .. })),
                "Single bit alteration in hash must trigger hash mismatch"
            );
        }
    }

    #[test]
    fn test_crumb_ledger_chronological_sequencing_validation() {
        let mut ledger = CrumbLedger::new();
        let base_time: u64 = 1_700_000_100;

        ledger
            .append(
                "agent-alpha".into(),
                CrumbAction::Create,
                "src/main.rs".into(),
                "Initial commit".into(),
                b"fn main() {}",
                base_time,
            )
            .expect("first append must succeed");

        // Attempt to append an event with a past timestamp (chronological violation)
        let past_time = base_time - 50;
        let append_res = ledger.append(
            "agent-beta".into(),
            CrumbAction::Modify,
            "src/main.rs".into(),
            "Retroactive edit attempt".into(),
            b"fn main() { println!(); }",
            past_time,
        );

        assert!(
            matches!(
                append_res,
                Err(LedgerError::ChronologicalViolation {
                    index: 1,
                    previous: 1_700_000_100,
                    current: 1_700_000_050
                })
            ),
            "Appending past timestamp must be rejected with ChronologicalViolation"
        );

        // Identical timestamp within same batch succeeds
        assert!(
            ledger
                .append(
                    "agent-gamma".into(),
                    CrumbAction::Audit,
                    "src/main.rs".into(),
                    "Concurrent audit in same second".into(),
                    b"audit-proof",
                    base_time,
                )
                .is_ok()
        );

        // Forward timestamp succeeds
        assert!(
            ledger
                .append(
                    "agent-delta".into(),
                    CrumbAction::Test,
                    "tests/unit.rs".into(),
                    "Pass test".into(),
                    b"test-proof",
                    base_time + 10,
                )
                .is_ok()
        );

        assert!(ledger.verify_chain().is_ok());
    }

    #[test]
    fn test_crumb_ledger_missing_parent_detection() {
        let mut ledger = CrumbLedger::new();
        let base_time: u64 = 1_700_000_000;

        for i in 0..5 {
            ledger
                .append(
                    "agent-worker".into(),
                    CrumbAction::Modify,
                    format!("file_{}.rs", i),
                    format!("Intent {}", i),
                    format!("data-{}", i).as_bytes(),
                    base_time + i,
                )
                .unwrap();
        }

        // Test missing parent when index 2 is deleted from the sequence
        let mut gapped_ledger = CrumbLedger::new();
        gapped_ledger.events.push(ledger.events[0].clone());
        gapped_ledger.events.push(ledger.events[1].clone());
        // Skip index 2, append index 3
        gapped_ledger.events.push(ledger.events[3].clone());

        let err = gapped_ledger.verify_chain();
        assert!(
            matches!(err, Err(LedgerError::InvalidIndex { index: 3, expected: 2 })),
            "Gapped chain must be rejected for index inconsistency"
        );

        // Test altered parent_hash directly
        let mut corrupted_parent = ledger.clone();
        corrupted_parent.events[2].parent_hash = [0xFF; 32];
        let err_parent = corrupted_parent.verify_chain();
        // Since hash depends on parent_hash, hash verification fails first
        assert!(matches!(err_parent, Err(LedgerError::HashMismatch { index: 2, .. })));

        // If attacker recomputes hash to match corrupted parent_hash:
        corrupted_parent.events[2].hash = LedgerEvent::compute_hash(
            corrupted_parent.events[2].index,
            corrupted_parent.events[2].timestamp,
            &corrupted_parent.events[2].agent,
            &corrupted_parent.events[2].action,
            &corrupted_parent.events[2].target,
            &corrupted_parent.events[2].intent,
            &corrupted_parent.events[2].payload_hash,
            &corrupted_parent.events[2].parent_hash,
        );
        let err_parent2 = corrupted_parent.verify_chain();
        assert!(
            matches!(err_parent2, Err(LedgerError::ParentHashMismatch { index: 2, .. })),
            "Parent hash mismatch must be detected when referencing non-existent parent"
        );

        // Test verify_parent method
        assert!(ledger.verify_parent(&ledger.events[1]).is_ok());
        let mut detached_event = ledger.events[3].clone();
        detached_event.parent_hash = [0xAA; 32];
        assert!(matches!(
            ledger.verify_parent(&detached_event),
            Err(LedgerError::ParentHashMismatch { .. })
        ));
    }

    #[test]
    fn test_crumb_ledger_serialization_roundtrip_json() {
        let mut ledger = CrumbLedger::new();
        let base_time: u64 = 1_700_000_500;

        ledger
            .append(
                "agent-builder".into(),
                CrumbAction::Create,
                "Cargo.toml".into(),
                "Define workspace manifest".into(),
                b"[workspace]\nresolver = \"2\"",
                base_time,
            )
            .unwrap();

        ledger
            .append(
                "agent-reviewer".into(),
                CrumbAction::Audit,
                "Cargo.toml".into(),
                "Verify workspace dependencies".into(),
                b"audit: clean",
                base_time + 5,
            )
            .unwrap();

        let json_str = ledger.to_json().expect("JSON serialization must succeed");
        assert!(json_str.contains("agent-builder"));
        assert!(json_str.contains("agent-reviewer"));
        assert!(json_str.contains("payload_hash"));

        let restored = CrumbLedger::from_json(&json_str).expect("JSON deserialization must succeed");
        assert_eq!(ledger, restored, "Deserialized ledger must exactly equal original");
        assert!(restored.verify_chain().is_ok(), "Restored ledger chain must verify");

        // Corrupted JSON must fail
        assert!(CrumbLedger::from_json("invalid json payload").is_err());
    }

    #[test]
    fn test_crumb_ledger_serialization_roundtrip_canonical_binary() {
        let mut ledger = CrumbLedger::new();
        let base_time: u64 = 1_700_000_900;

        for i in 0..10 {
            ledger
                .append(
                    format!("agent-binary-{}", i),
                    CrumbAction::Modify,
                    format!("kernel/module_{}.bin", i),
                    format!("Install binary patch {}", i),
                    &[i as u8; 64],
                    base_time + (i as u64 * 10),
                )
                .unwrap();
        }

        let binary_data = ledger.to_binary().expect("Binary serialization must succeed");
        assert!(!binary_data.is_empty());

        let restored = CrumbLedger::from_binary(&binary_data).expect("Binary deserialization must succeed");
        assert_eq!(ledger, restored, "Canonical binary roundtrip must yield identical ledger");
        assert!(restored.verify_chain().is_ok(), "Restored ledger from binary must verify");

        // Corrupted binary payload must fail
        let corrupted_bytes = vec![0xFF, 0x00, 0xAA];
        assert!(CrumbLedger::from_binary(&corrupted_bytes).is_err());
    }
}
