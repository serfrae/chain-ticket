use {
    crate::{
        constants::{EVENT_SEED, EVENT_STATE_SIZE, MINT_SEED, VAULT_SEED},
        errors::ChainTicketError,
        state::Event,
    },
    anchor_lang::{
        prelude::*,
        solana_program::{clock::Clock, rent::Rent},
    },
    anchor_spl::token::{Mint, Token},
};

#[derive(Accounts)]
pub struct EndEvent<'info> {
    #[account(
        mut,
        address = event.authority @ ChainTicketError::Unauthorised,
    )]
    authority: Signer<'info>,
    #[account(
        mut, 
        close = authority,
        seeds = [EVENT_SEED, authority.key().as_ref()],
        bump,
    )]
    event: Account<'info, Event>,
    /// CHECK: Address is derived and is a native vault,
    /// in order to facilitate transfers from the vault
    /// it must have no data and thus no discriminator.
    #[account(
        mut,
        seeds = [VAULT_SEED, event.key().as_ref()],
        bump,
        address = event.vault @ ChainTicketError::InvalidVault,
    )]
    vault: UncheckedAccount<'info>,
    #[account(
        mut, 
        seeds = [MINT_SEED, event.key().as_ref()],
        bump,
        address = event.mint @ ChainTicketError::InvalidMint,
    )]
    mint: Account<'info, Mint>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

/// Ends the event by closing the mint and the event accounts relcaiming rent in the process.
/// Can only be called if the associated mint's supply is 0. I.e. requires burning all tokens.
pub fn process_end(ctx: Context<EndEvent>) -> Result<()> {
    let clock = Clock::get()?;
    let rent = Rent::get()?;

    let event_lamports = ctx.accounts.event.get_lamports();

    // Check the event has ended
    require_gte!(
        clock.unix_timestamp,
        ctx.accounts.event.event_date + ctx.accounts.event.refund_period,
        ChainTicketError::EventNotEnded,
    );

    // Check that funds have been withdrawn by comparing against the minimum rent amount
    require_gte!(rent.minimum_balance(8 + EVENT_STATE_SIZE), event_lamports);

    // Check mint supply
    require_eq!(ctx.accounts.mint.supply, 0, ChainTicketError::NonZeroSupply);

    // Close vault 
    ctx.accounts
        .vault
        .to_account_info()
        .assign(&ctx.accounts.system_program.key());

    // Zero data
    ctx.accounts.vault.to_account_info().realloc(0, false)?;

    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.vault.key(),
        &ctx.accounts.authority.key(),
        ctx.accounts.vault.get_lamports(),
    );

    // Transfer lamports to authority
    anchor_lang::solana_program::program::invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        &[&[
            VAULT_SEED,
            ctx.accounts.event.key().as_ref(),
            &[ctx.bumps.vault],
        ]],
    )?;

    Ok(())
}
