//! The Cryptographic Crumb Ledger.
//!
//! Provides hash-chained, chronological action vector verification
//! for multi-agent stigmergic coordination and immutable repository history.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrumbAction {
    Create,
    Modify,
    Delete,
    Audit,
    Test,
    Build,
    Custom(String),
}

impl CrumbAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Create => "create",
            Self::Modify => "modify",
            Self::Delete => "delete",
            Self::Audit => "audit",
            Self::Test => "test",
            Self::Build => "build",
            Self::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum LedgerError {
    #[error("Hash mismatch at index {index}: computed {computed:?}, recorded {recorded:?}")]
    HashMismatch {
        index: u64,
        computed: [u8; 32],
        recorded: [u8; 32],
    },
    #[error("Parent hash mismatch at index {index}: expected {expected:?}, found {found:?}")]
    ParentHashMismatch {
        index: u64,
        expected: [u8; 32],
        found: [u8; 32],
    },
    #[error("Chronological sequencing violation at index {index}: previous timestamp {previous} > current timestamp {current}")]
    ChronologicalViolation {
        index: u64,
        previous: u64,
        current: u64,
    },
    #[error("Invalid index sequence at {index}: expected {expected}")]
    InvalidIndex {
        index: u64,
        expected: u64,
    },
    #[error("Missing parent event for event at index {0}")]
    MissingParent(u64),
    #[error("Genesis parent hash must be all zeros, got {0:?}")]
    InvalidGenesisParent([u8; 32]),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEvent {
    pub index: u64,
    pub timestamp: u64,
    pub agent: String,
    pub action: CrumbAction,
    pub target: String,
    pub intent: String,
    pub payload_hash: [u8; 32],
    pub parent_hash: [u8; 32],
    pub hash: [u8; 32],
}

impl LedgerEvent {
    pub fn compute_hash(
        index: u64,
        timestamp: u64,
        agent: &str,
        action: &CrumbAction,
        target: &str,
        intent: &str,
        payload_hash: &[u8; 32],
        parent_hash: &[u8; 32],
    ) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"CRUMB_LEDGER_EVENT_V1");
        hasher.update(&index.to_be_bytes());
        hasher.update(&timestamp.to_be_bytes());
        hasher.update(&(agent.len() as u32).to_be_bytes());
        hasher.update(agent.as_bytes());
        let action_str = action.as_str();
        hasher.update(&(action_str.len() as u32).to_be_bytes());
        hasher.update(action_str.as_bytes());
        hasher.update(&(target.len() as u32).to_be_bytes());
        hasher.update(target.as_bytes());
        hasher.update(&(intent.len() as u32).to_be_bytes());
        hasher.update(intent.as_bytes());
        hasher.update(payload_hash);
        hasher.update(parent_hash);
        *hasher.finalize().as_bytes()
    }

    pub fn new(
        index: u64,
        timestamp: u64,
        agent: String,
        action: CrumbAction,
        target: String,
        intent: String,
        payload: &[u8],
        parent_hash: [u8; 32],
    ) -> Self {
        let payload_hash = *blake3::hash(payload).as_bytes();
        let hash = Self::compute_hash(
            index,
            timestamp,
            &agent,
            &action,
            &target,
            &intent,
            &payload_hash,
            &parent_hash,
        );

        Self {
            index,
            timestamp,
            agent,
            action,
            target,
            intent,
            payload_hash,
            parent_hash,
            hash,
        }
    }

    pub fn verify_hash(&self) -> Result<(), LedgerError> {
        let computed = Self::compute_hash(
            self.index,
            self.timestamp,
            &self.agent,
            &self.action,
            &self.target,
            &self.intent,
            &self.payload_hash,
            &self.parent_hash,
        );

        if computed != self.hash {
            return Err(LedgerError::HashMismatch {
                index: self.index,
                computed,
                recorded: self.hash,
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrumbLedger {
    pub events: Vec<LedgerEvent>,
}

impl CrumbLedger {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn last_event(&self) -> Option<&LedgerEvent> {
        self.events.last()
    }

    pub fn head_hash(&self) -> [u8; 32] {
        self.events
            .last()
            .map(|e| e.hash)
            .unwrap_or([0u8; 32])
    }

    pub fn append(
        &mut self,
        agent: String,
        action: CrumbAction,
        target: String,
        intent: String,
        payload: &[u8],
        timestamp: u64,
    ) -> Result<&LedgerEvent, LedgerError> {
        let index = self.events.len() as u64;
        let parent_hash = if let Some(last) = self.events.last() {
            if timestamp < last.timestamp {
                return Err(LedgerError::ChronologicalViolation {
                    index,
                    previous: last.timestamp,
                    current: timestamp,
                });
            }
            last.hash
        } else {
            [0u8; 32]
        };

        let event = LedgerEvent::new(
            index,
            timestamp,
            agent,
            action,
            target,
            intent,
            payload,
            parent_hash,
        );

        self.events.push(event);
        Ok(self.events.last().expect("event just pushed"))
    }

    pub fn verify_chain(&self) -> Result<(), LedgerError> {
        for (i, event) in self.events.iter().enumerate() {
            if event.index != i as u64 {
                return Err(LedgerError::InvalidIndex {
                    index: event.index,
                    expected: i as u64,
                });
            }

            event.verify_hash()?;

            if i == 0 {
                if event.parent_hash != [0u8; 32] {
                    return Err(LedgerError::InvalidGenesisParent(event.parent_hash));
                }
            } else {
                let prev = &self.events[i - 1];
                if event.parent_hash != prev.hash {
                    return Err(LedgerError::ParentHashMismatch {
                        index: event.index,
                        expected: prev.hash,
                        found: event.parent_hash,
                    });
                }
                if event.timestamp < prev.timestamp {
                    return Err(LedgerError::ChronologicalViolation {
                        index: event.index,
                        previous: prev.timestamp,
                        current: event.timestamp,
                    });
                }
            }
        }

        Ok(())
    }

    pub fn verify_parent(&self, event: &LedgerEvent) -> Result<(), LedgerError> {
        if event.index == 0 {
            if event.parent_hash != [0u8; 32] {
                return Err(LedgerError::InvalidGenesisParent(event.parent_hash));
            }
            return Ok(());
        }

        let parent_idx = (event.index - 1) as usize;
        if parent_idx >= self.events.len() {
            return Err(LedgerError::MissingParent(event.index));
        }

        let expected_parent = &self.events[parent_idx];
        if event.parent_hash != expected_parent.hash {
            return Err(LedgerError::ParentHashMismatch {
                index: event.index,
                expected: expected_parent.hash,
                found: event.parent_hash,
            });
        }

        Ok(())
    }

    pub fn to_json(&self) -> Result<String, LedgerError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| LedgerError::Serialization(e.to_string()))
    }

    pub fn from_json(json_str: &str) -> Result<Self, LedgerError> {
        serde_json::from_str(json_str)
            .map_err(|e| LedgerError::Deserialization(e.to_string()))
    }

    pub fn to_binary(&self) -> Result<Vec<u8>, LedgerError> {
        bincode::serialize(self)
            .map_err(|e| LedgerError::Serialization(e.to_string()))
    }

    pub fn from_binary(bytes: &[u8]) -> Result<Self, LedgerError> {
        bincode::deserialize(bytes)
            .map_err(|e| LedgerError::Deserialization(e.to_string()))
    }
}
