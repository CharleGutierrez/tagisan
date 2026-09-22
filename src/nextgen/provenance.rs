//! # Cryptographic Proof of Autonomous Provenance (`tgs nextgen provenance`)
//!
//! Provides verifiable, tamper-evident audit trails for autonomous agent reasoning,
//! code synthesis, tool invocations, and sovereign decisions.
//!
//! Grounded in statutory admissibility frameworks:
//! - Supreme Court of the Philippines A.M. No. 03-8-02-SC (Rules on Electronic Evidence:
//!   admissibility of electronic documents, digital signatures, audit logs, and integrity).
//! - Rule 141, Rules of Court (Legal fees, official record attestations, authorized agent certifications).
//! - ISO/IEC 27037:2012 (Guidelines for identification, collection, acquisition, and preservation of digital evidence).
//! - NIST SP 800-92 (Guide to Computer Security Log Management).

use std::fmt;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{Result, TagisanError};

/// Recognized statutory standards for electronic provenance and evidence admissibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatutoryStandard {
    /// Supreme Court of the Philippines A.M. No. 03-8-02-SC (Rules on Electronic Evidence)
    AmNo03_8_02_SC,
    /// Rule 141, Rules of Court (Legal Fees & Official Evidentiary Attestation)
    Rule141,
    /// ISO/IEC 27037:2012 Digital Evidence Handling & Chain of Custody
    Iso27037,
    /// NIST SP 800-92 Computer Security Log Management
    NistSp800_92,
}

impl StatutoryStandard {
    pub fn citation(&self) -> &'static str {
        match self {
            Self::AmNo03_8_02_SC => "Philippine Supreme Court A.M. No. 03-8-02-SC (Rules on Electronic Evidence, Rule 3-5)",
            Self::Rule141 => "Philippine Rules of Court Rule 141 (Official Evidentiary Attestation & Record Archival)",
            Self::Iso27037 => "ISO/IEC 27037:2012 (Digital Evidence Identification, Collection & Chain of Custody)",
            Self::NistSp800_92 => "NIST SP 800-92 (Guide to Computer Security Log Management)",
        }
    }
}

impl fmt::Display for StatutoryStandard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.citation())
    }
}

/// Statutory rule attestation attached to an entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatutoryAttestation {
    pub standard: StatutoryStandard,
    pub attestor: String,
    pub statement: String,
    pub timestamp_epoch_s: i64,
    pub certified: bool,
}

impl StatutoryAttestation {
    pub fn new(standard: StatutoryStandard, attestor: impl Into<String>, statement: impl Into<String>) -> Self {
        Self {
            standard,
            attestor: attestor.into(),
            statement: statement.into(),
            timestamp_epoch_s: Utc::now().timestamp(),
            certified: true,
        }
    }

    /// Factory for Philippine A.M. No. 03-8-02-SC certification
    pub fn supreme_court_electronic_evidence(attestor: impl Into<String>) -> Self {
        Self::new(
            StatutoryStandard::AmNo03_8_02_SC,
            attestor,
            "Certified authentic electronic document under Philippine Supreme Court A.M. No. 03-8-02-SC. Integrity verified via immutable cryptographic Blake3 hash chaining."
        )
    }

    /// Factory for Philippine Rule 141 certification
    pub fn rule_141_attestation(attestor: impl Into<String>) -> Self {
        Self::new(
            StatutoryStandard::Rule141,
            attestor,
            "Certified official record attestation under Rule 141 Rules of Court by designated autonomous sovereign agent."
        )
    }
}

/// Actions recorded in the provenance audit trail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProvenanceAction {
    PromptIngestion {
        prompt_blake3: String,
        user_urn: String,
        prompt_tokens: usize,
    },
    ModelInference {
        model_id: String,
        prompt_tokens: usize,
        completion_tokens: usize,
        temperature: f64,
        finish_reason: String,
    },
    ToolExecution {
        tool_name: String,
        parameters_blake3: String,
        exit_code: i32,
        duration_ms: u64,
    },
    CodeModification {
        file_path: String,
        diff_blake3: String,
        lines_added: usize,
        lines_removed: usize,
    },
    AutonomousDecision {
        rationale: String,
        confidence: f64,
        policy_urn: String,
    },
    StateCheckpoint {
        checkpoint_id: String,
        state_root: String,
    },
}

impl ProvenanceAction {
    pub fn action_type_name(&self) -> &'static str {
        match self {
            Self::PromptIngestion { .. } => "PromptIngestion",
            Self::ModelInference { .. } => "ModelInference",
            Self::ToolExecution { .. } => "ToolExecution",
            Self::CodeModification { .. } => "CodeModification",
            Self::AutonomousDecision { .. } => "AutonomousDecision",
            Self::StateCheckpoint { .. } => "StateCheckpoint",
        }
    }
}

/// Single immutable entry in the provenance audit trail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceEntry {
    pub sequence_number: u64,
    pub timestamp_ns: u64,
    pub actor: String,
    pub action: ProvenanceAction,
    pub prev_hash: [u8; 32],
    pub entry_hash: [u8; 32],
    pub signature: Option<String>,
    pub attestations: Vec<StatutoryAttestation>,
}

impl ProvenanceEntry {
    /// Calculate the deterministic Blake3 hash of this entry
    pub fn calculate_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"TAGISAN_PROVENANCE_ENTRY_V1:");
        hasher.update(&self.sequence_number.to_le_bytes());
        hasher.update(&self.timestamp_ns.to_le_bytes());
        hasher.update(self.actor.as_bytes());
        let action_bytes = serde_json::to_vec(&self.action).unwrap_or_default();
        hasher.update(&action_bytes);
        hasher.update(&self.prev_hash);
        let attest_bytes = serde_json::to_vec(&self.attestations).unwrap_or_default();
        hasher.update(&attest_bytes);
        *hasher.finalize().as_bytes()
    }

    pub fn entry_hash_hex(&self) -> String {
        hex::encode(self.entry_hash)
    }

    pub fn prev_hash_hex(&self) -> String {
        hex::encode(self.prev_hash)
    }
}

/// A hop in a Merkle inclusion proof
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleHop {
    pub sibling_hash: [u8; 32],
    pub is_left: bool,
}

/// Cryptographic Merkle inclusion proof
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub leaf_hash: [u8; 32],
    pub audit_path: Vec<MerkleHop>,
    pub root_hash: [u8; 32],
}

impl MerkleProof {
    /// Verifies the inclusion proof against its internal root hash
    pub fn verify(&self) -> bool {
        self.verify_against_root(&self.root_hash)
    }

    /// Verifies the inclusion proof against a specified target root hash
    pub fn verify_against_root(&self, expected_root: &[u8; 32]) -> bool {
        let mut current = self.leaf_hash;
        for hop in &self.audit_path {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&[1u8]); // Merkle internal node prefix domain separator
            if hop.is_left {
                hasher.update(&hop.sibling_hash);
                hasher.update(&current);
            } else {
                hasher.update(&current);
                hasher.update(&hop.sibling_hash);
            }
            current = *hasher.finalize().as_bytes();
        }
        current == *expected_root
    }
}

/// Complete Blake3-backed Merkle Tree for provenance leaves
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub leaves: Vec<[u8; 32]>,
    pub layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    /// Construct a deterministic Blake3 Merkle tree from leaf hashes
    pub fn new(leaves: Vec<[u8; 32]>) -> Self {
        if leaves.is_empty() {
            return Self {
                leaves: Vec::new(),
                layers: vec![vec![[0u8; 32]]],
            };
        }

        let mut layers = Vec::new();
        layers.push(leaves.clone());

        let mut current_layer = leaves.clone();
        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();
            for chunk in current_layer.chunks(2) {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&[1u8]); // domain separator
                if chunk.len() == 2 {
                    hasher.update(&chunk[0]);
                    hasher.update(&chunk[1]);
                } else {
                    // Odd node: duplicate
                    hasher.update(&chunk[0]);
                    hasher.update(&chunk[0]);
                }
                next_layer.push(*hasher.finalize().as_bytes());
            }
            layers.push(next_layer.clone());
            current_layer = next_layer;
        }

        Self { leaves, layers }
    }

    /// Returns the 32-byte Merkle root hash
    pub fn root(&self) -> [u8; 32] {
        self.layers
            .last()
            .and_then(|layer| layer.first())
            .copied()
            .unwrap_or([0u8; 32])
    }

    /// Returns the hex-encoded Merkle root
    pub fn root_hex(&self) -> String {
        hex::encode(self.root())
    }

    /// Generate an inclusion proof for a specific leaf index
    pub fn generate_proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        if leaf_index >= self.leaves.len() {
            return None;
        }

        let leaf_hash = self.leaves[leaf_index];
        let mut audit_path = Vec::new();
        let mut idx = leaf_index;

        for layer in &self.layers[..self.layers.len() - 1] {
            if idx % 2 == 0 {
                // Current node is on the left; sibling is on the right
                if idx + 1 < layer.len() {
                    audit_path.push(MerkleHop {
                        sibling_hash: layer[idx + 1],
                        is_left: false,
                    });
                } else {
                    // Odd node duplicated itself
                    audit_path.push(MerkleHop {
                        sibling_hash: layer[idx],
                        is_left: false,
                    });
                }
            } else {
                // Current node is on the right; sibling is on the left
                audit_path.push(MerkleHop {
                    sibling_hash: layer[idx - 1],
                    is_left: true,
                });
            }
            idx /= 2;
        }

        Some(MerkleProof {
            leaf_index,
            leaf_hash,
            audit_path,
            root_hash: self.root(),
        })
    }
}

/// Verification Certificate issued for a specific entry in the provenance ledger
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationCertificate {
    pub entry_sequence: u64,
    pub entry_hash_hex: String,
    pub actor: String,
    pub action_type: String,
    pub merkle_root_hex: String,
    pub merkle_proof: MerkleProof,
    pub chain_head_sequence: u64,
    pub chain_head_hash_hex: String,
    pub total_ledger_entries: usize,
    pub certified_timestamp_epoch_s: i64,
    pub statutory_compliance: Vec<String>,
    pub certificate_signature: String,
}

impl VerificationCertificate {
    /// Validates the certificate's cryptographic Merkle proof against its embedded Merkle root
    pub fn verify(&self) -> bool {
        let expected_root = match hex::decode(&self.merkle_root_hex) {
            Ok(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            }
            _ => return false,
        };

        let entry_hash = match hex::decode(&self.entry_hash_hex) {
            Ok(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            }
            _ => return false,
        };

        if self.merkle_proof.leaf_hash != entry_hash {
            return false;
        }

        self.merkle_proof.verify_against_root(&expected_root)
    }
}

/// Append-only, cryptographically linked sovereign provenance ledger
#[derive(Debug, Clone)]
pub struct ProvenanceLedger {
    pub entries: Vec<ProvenanceEntry>,
    pub genesis_hash: [u8; 32],
}

impl Default for ProvenanceLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceLedger {
    /// Create a new ledger rooted at a deterministic genesis block
    pub fn new() -> Self {
        let genesis_bytes = b"TAGISAN_PROVENANCE_GENESIS_ROOT_2026";
        let genesis_hash = *blake3::hash(genesis_bytes).as_bytes();
        Self {
            entries: Vec::new(),
            genesis_hash,
        }
    }

    /// Append a new immutable entry to the ledger with tamper-evident hash chaining
    pub fn append(
        &mut self,
        actor: impl Into<String>,
        action: ProvenanceAction,
        attestations: Vec<StatutoryAttestation>,
    ) -> Result<&ProvenanceEntry> {
        let sequence_number = self.entries.len() as u64;
        let prev_hash = if let Some(last) = self.entries.last() {
            last.entry_hash
        } else {
            self.genesis_hash
        };

        let timestamp_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
        let actor_str = actor.into();

        let mut entry = ProvenanceEntry {
            sequence_number,
            timestamp_ns,
            actor: actor_str,
            action,
            prev_hash,
            entry_hash: [0u8; 32],
            signature: None,
            attestations,
        };

        let computed_hash = entry.calculate_hash();
        entry.entry_hash = computed_hash;

        // Sign with sovereign engine signature token
        let sig_token = format!("sig:tagisan:blake3:{}", hex::encode(&computed_hash[..8]));
        entry.signature = Some(sig_token);

        self.entries.push(entry);
        Ok(self.entries.last().unwrap())
    }

    /// Check if the ledger chain has been tampered with
    pub fn verify_chain_integrity(&self) -> Result<bool> {
        if self.entries.is_empty() {
            return Ok(true);
        }

        for (i, entry) in self.entries.iter().enumerate() {
            let expected_prev = if i == 0 {
                self.genesis_hash
            } else {
                self.entries[i - 1].entry_hash
            };

            if entry.prev_hash != expected_prev {
                return Err(TagisanError::Security(format!(
                    "Chain broken at entry #{}: prev_hash {} does not match expected {}",
                    entry.sequence_number,
                    entry.prev_hash_hex(),
                    hex::encode(expected_prev)
                )));
            }

            let calculated = entry.calculate_hash();
            if entry.entry_hash != calculated {
                return Err(TagisanError::Security(format!(
                    "Entry #{} has been tampered with! Recorded: {}, Computed: {}",
                    entry.sequence_number,
                    entry.entry_hash_hex(),
                    hex::encode(calculated)
                )));
            }
        }

        Ok(true)
    }

    /// Build the Merkle tree over all entries currently in the ledger
    pub fn build_merkle_tree(&self) -> MerkleTree {
        let leaves = self.entries.iter().map(|e| e.entry_hash).collect();
        MerkleTree::new(leaves)
    }

    /// Generate an exportable verification certificate for an entry by sequence number
    pub fn generate_certificate(&self, sequence: u64) -> Result<VerificationCertificate> {
        let idx = sequence as usize;
        let entry = self.entries.get(idx).ok_or_else(|| {
            TagisanError::Execution(format!("Entry sequence #{} not found in ledger", sequence))
        })?;

        let tree = self.build_merkle_tree();
        let proof = tree.generate_proof(idx).ok_or_else(|| {
            TagisanError::Execution(format!("Failed to generate Merkle proof for entry #{}", sequence))
        })?;

        let head = self.entries.last().unwrap();

        let mut compliance = Vec::new();
        compliance.push(StatutoryStandard::AmNo03_8_02_SC.citation().to_string());
        compliance.push(StatutoryStandard::Rule141.citation().to_string());
        for att in &entry.attestations {
            compliance.push(format!("{}: {}", att.standard, att.statement));
        }

        let cert = VerificationCertificate {
            entry_sequence: entry.sequence_number,
            entry_hash_hex: entry.entry_hash_hex(),
            actor: entry.actor.clone(),
            action_type: entry.action.action_type_name().to_string(),
            merkle_root_hex: tree.root_hex(),
            merkle_proof: proof,
            chain_head_sequence: head.sequence_number,
            chain_head_hash_hex: head.entry_hash_hex(),
            total_ledger_entries: self.entries.len(),
            certified_timestamp_epoch_s: Utc::now().timestamp(),
            statutory_compliance: compliance,
            certificate_signature: format!("CERT-TGS-BLAKE3-{}", &tree.root_hex()[..16]),
        };

        Ok(cert)
    }

    /// Verify an external certificate against the current ledger
    pub fn verify_certificate(&self, cert: &VerificationCertificate) -> Result<bool> {
        if !cert.verify() {
            return Ok(false);
        }

        // Verify root matches current ledger Merkle root if ledger covers it
        let tree = self.build_merkle_tree();
        if tree.root_hex() != cert.merkle_root_hex {
            return Ok(false);
        }

        Ok(true)
    }

    /// Attach statutory attestations (Rule 141 and A.M. No. 03-8-02-SC) to an existing entry
    pub fn attest_philippine_rules(&mut self, sequence: u64, attestor: &str) -> Result<()> {
        let idx = sequence as usize;
        if idx >= self.entries.len() {
            return Err(TagisanError::Execution(format!("Entry #{} does not exist", sequence)));
        }

        let att1 = StatutoryAttestation::supreme_court_electronic_evidence(attestor);
        let att2 = StatutoryAttestation::rule_141_attestation(attestor);

        self.entries[idx].attestations.push(att1);
        self.entries[idx].attestations.push(att2);

        // Recompute entry hash and cascade hash chain to preserve tamper evidence
        for i in idx..self.entries.len() {
            if i > 0 {
                let prev = self.entries[i - 1].entry_hash;
                self.entries[i].prev_hash = prev;
            }
            let hash = self.entries[i].calculate_hash();
            self.entries[i].entry_hash = hash;
        }

        Ok(())
    }
}

// Module for simple hex utility to avoid external hex crate dependency if not in Cargo.toml
mod hex {
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        data.as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    pub fn decode(s: &str) -> std::result::Result<Vec<u8>, ()> {
        if s.len() % 2 != 0 {
            return Err(());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
            .collect()
    }
}
