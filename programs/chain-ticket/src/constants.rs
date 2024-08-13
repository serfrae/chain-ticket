/// Seed used for constructing the event PDA.
pub const EVENT_SEED: &[u8; 5] = b"event";
/// Seed used for constructing the vault PDA.
pub const VAULT_SEED: &[u8; 5] = b"vault";
/// Seed used for constructing the mint PDA.
pub const MINT_SEED: &[u8; 4] = b"mint";
/// Seed required for constructing the metadata PDA.
pub const METADATA_SEED: &[u8; 8] = b"metadata";

/// Size of the account holding the event's details (its state).
pub const EVENT_STATE_SIZE: usize = 126;

/// The public key of the platform owner
pub const PLATFORM_OWNER: anchor_lang::solana_program::pubkey::Pubkey =
    anchor_lang::pubkey!("EcmsHx8pZQqpHViyecmTNyzKpRrm3PGw86WWaK6dXmcs");

/// Platform fee (as a percentage)
pub const FEE: u64 = 1;

/// SOL amount to deposit. Used as a guarantee should the event be cancelled
pub const DEPOSIT_AMOUNT: u64 = 2;
