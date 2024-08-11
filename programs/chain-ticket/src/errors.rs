use anchor_lang::error_code;

#[error_code]
pub enum ChainTicketError {
    #[msg("Error calculating platform fee")]
    FeeCalculationError,

    #[msg("Max tickets sold")]
    MaxTicketsSold,

    #[msg("Unauthorised access")]
    Unauthorised,

    #[msg("Not all tickets refunded")]
    SupplyNotZero,

    #[msg("Event has not ended")]
    EventNotEnded,

    #[msg("Could not parse pubkey")]
    PubkeyParseError,

    #[msg("Incorrect platform owner address")]
    IncorrectPlatformOwner,

    #[msg("Amount overflow")]
    Overflow,
}
