//! This crate provides helpers for Generating provable transactions on World Chain.

use alloy_primitives::{Address, address};

pub mod bindings;
pub mod transaction;
pub mod world_id;

/// The chain ID for WC Sepolia
pub const CHAIN_ID: u64 = 4801;

/// The PBH CTF contract address.
pub const PBH_CTF_CONTRACT: Address = address!("0xbA9FfCB4cc50eE2EC2F89740f5d6841cC8A74030");

/// The entrypoint contract for all PBH transactions.
pub const PBH_ENTRY_POINT: Address = address!("0x6e37bAB9d23bd8Bdb42b773C58ae43C6De43A590");

/// The Signature Aggregator for 4337 priority bundles.
pub const PBH_SIGNATURE_AGGREGATOR: Address = address!("0xf07d3efadD82A1F0b4C5Cc3476806d9a170147Ba");

/// The URL for the inclusion proof endpoint.
pub const INCLUSION_PROOF_URL: &str = "https://signup-orb-ethereum.stage-crypto.worldcoin.dev";

// Re-exports
pub use transaction::*;

pub use world_chain_builder_pbh::{
    date_marker::DateMarker,
    external_nullifier::{EncodedExternalNullifier, ExternalNullifier},
    payload::Proof as PBHProof,
};

pub use world_chain_builder_test_utils::bindings::*;

pub use semaphore_rs::{
    Field, hash_to_field,
    identity::Identity,
    poseidon_tree::Proof,
    protocol::{generate_nullifier_hash, generate_proof},
};
