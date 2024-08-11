use anchor_lang::prelude::*;

#[account]
pub struct Event {
    // Stored seed to avoid computation on every call that requires a CPI
    pub bump: u8, // 1
    // The address of the event organiser
    pub authority: Pubkey, // 32
    // Whether purchases are allowed - setting this field to true will
    // incur the deposit fee
    pub allow_purchase: bool, // 1
    // Mint address for the ticket token - generated by the program.
    pub mint: Pubkey, // 32
    // Event date in unix time
    pub event_date: i64, // 8
    // Ticket price in lamports
    pub ticket_price: u64, // 8
    // Time period for which refunds can be requested, is added to `event_date` to determine
    // when this period has elapsed. Funds cannot be withdrawn until this value is exceeded
    pub refund_period: i64, // 8
    // Number of tickets that are available for the event, mint supply will be capped to this
    // amount
    pub num_tickets: u32, // 4
}
