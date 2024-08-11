/// Seed used for constructing the event PDA. Used a a signer seed in CPIs
pub const EVENT_SEED: &[u8; 5] = b"event";
pub const MINT_SEED: &[u8; 4] = b"mint";
pub const METADATA_SEED: &[u8; 8] = b"metadata";

/// Size of the account holding the event's details (its state).
pub const EVENT_STATE_SIZE: usize = 94;

/// The public key of the platform owner 
pub const PLATFORM_OWNER: &str = "YOUR_PUBKEY_HERE";

/// Platform fee
pub const FEE: u64 = 1;

/// SOL amount to deposit used as a guarantee should the event be cancelled
pub const DEPOSIT_AMOUNT: u64 = 2;
