use anchor_lang::error_code;

#[error_code]
pub enum ChainTicketError {
    #[msg("Error calculating platform fee")]
    FeeCalculationError,

    #[msg("Max tickets sold")]
    MaxTicketsExceeded,

    #[msg("Unauthorised access")]
    Unauthorised,

    #[msg("Mint supply is not zero")]
    NonZeroSupply,

    #[msg("Event has not ended")]
    EventNotEnded,

    #[msg("Could not parse pubkey")]
    PubkeyParseError,

    #[msg("Incorrect platform owner address")]
    IncorrectPlatformOwner,

    #[msg("Amount overflow")]
    Overflow,

    #[msg("Invalid mint address")]
    InvalidMint,

    #[msg("Invalid vault address")]
    InvalidVault,

    #[msg("User has already purchased a ticket")]
    AlreadyPurchased,
}
