use {
    crate::{
        constants::{DEPOSIT_AMOUNT, EVENT_SEED, FEE, PLATFORM_OWNER, VAULT_SEED},
        errors::ChainTicketError,
        state::Event,
        utils::sol_to_lamports,
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    /// CHECK: Checked with constraint
    #[account(
        mut,
        address = PLATFORM_OWNER @ ChainTicketError::Unauthorised,
    )]
    pub platform_owner: UncheckedAccount<'info>,
    #[account(
        mut,
        address = event.authority @ ChainTicketError::Unauthorised,
    )]
    pub authority: Signer<'info>,
    #[account(
        seeds = [EVENT_SEED, authority.key().as_ref()],
        bump,
    )]
    pub event: Account<'info, Event>,
    /// CHECK: Address is derived and is a native vault,
    /// in order to facilitate transfers from the vault
    /// it must have no data and thus no discriminator.
    #[account(
        mut,
        seeds = [VAULT_SEED, event.key().as_ref()],
        bump,
        address = event.vault @ ChainTicketError::InvalidVault,
    )]
    pub vault: UncheckedAccount<'info>,
}

pub fn process_withdraw(ctx: Context<WithdrawFunds>) -> Result<()> {
    let clock = Clock::get()?;

    // Check refund period has elapsed
    require_gte!(
        clock.unix_timestamp,
        (ctx.accounts.event.event_date + ctx.accounts.event.refund_period)
    );

    let deposit_amount = sol_to_lamports(DEPOSIT_AMOUNT as f64);

    let proceeds = ctx
        .accounts
        .vault
        .get_lamports()
        .checked_sub(deposit_amount)
        .ok_or(ChainTicketError::Overflow)?;

    let platform_fee = proceeds
        .checked_div(FEE * 100)
        .ok_or(ChainTicketError::Overflow)?;

    // Deduct platform fee, proceeds and deposit amount
    **ctx.accounts.vault.try_borrow_mut_lamports()? -= proceeds + deposit_amount;
    // Transfer platform fee
    **ctx.accounts.platform_owner.try_borrow_mut_lamports()? += platform_fee;
    // Transfer proceeds + deposit amount
    **ctx.accounts.authority.try_borrow_mut_lamports()? +=
        (proceeds - platform_fee) + deposit_amount;

    Ok(())
}
