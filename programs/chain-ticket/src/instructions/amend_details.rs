use {
    crate::{constants::EVENT_SEED, state::Event},
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct AmendEvent<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        mut,
        seeds = [
            EVENT_SEED,
            authority.key().as_ref(),
        ],
            bump,
    )]
    event: Account<'info, Event>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AmendEventFields {
    pub event_date: Option<i64>,
    pub ticket_price: Option<u64>,
    pub num_tickets: Option<u32>,
}

/// Amend fields that are not passed in as `None`, passing a `None` for any of the fields in
/// `AmendEventFields` means that field will not be amended.
pub fn process_amend(ctx: Context<AmendEvent>, data: AmendEventFields) -> Result<()> {
    require_keys_eq!(ctx.accounts.authority.key(), ctx.accounts.event.authority);

    if let Some(event_date) = data.event_date {
        ctx.accounts.event.event_date = event_date;
    }

    if let Some(num_tickets) = data.num_tickets {
        ctx.accounts.event.num_tickets = num_tickets;
    }

    if let Some(ticket_price) = data.ticket_price {
        ctx.accounts.event.ticket_price = ticket_price;
    }

    Ok(())
}
