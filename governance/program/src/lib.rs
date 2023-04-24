#![allow(clippy::integer_arithmetic)]
#![deny(missing_docs)]
//! A Governance program for the Solana blockchain.

pub mod addins;
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod tools;

/// solana-security-txt for SPL Governance program deployed by Marinade.finance
use solana_security_txt::security_txt;
#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "SPL Governance deployed by Marinade.finance",
    project_url: "https://github.com/solana-labs/solana-program-library/tree/master/governance",
    contacts: "link:https://docs.marinade.finance/marinade-dao,link:https://discord.com/invite/6EtUf4Euu6",
    policy: "https://docs.marinade.finance/marinade-protocol/security",
    preferred_languages: "en",
    source_code: "https://github.com/marinade-finance/solana-program-library/tree/governance-v3.1.0-marinade",
    auditors: "https://github.com/solana-labs/solana-program-library/tree/master/governance#audit"
}

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

/// Seed prefix for Governance  PDAs
/// Note: This prefix is used for the initial set of PDAs and shouldn't be used for any new accounts
/// All new PDAs should use a unique prefix to guarantee uniqueness for each account
pub const PROGRAM_AUTHORITY_SEED: &[u8] = b"governance";
