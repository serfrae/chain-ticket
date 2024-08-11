use {
    crate::{
        constants::{EVENT_SEED, EVENT_STATE_SIZE, MINT_SEED},
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
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        mut, 
        close = authority,
        seeds = [EVENT_SEED, authority.key().as_ref()],
        bump,
    )]
    event: Account<'info, Event>,
    #[account(
        mut, 
        close = authority,
        seeds = [MINT_SEED, event.key().as_ref()],
        bump,
    )]
    mint: Account<'info, Mint>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

/// Ends the event by closing the mint and the event accounts relcaiming rent in the process.
/// Can only be called if the associated mint's supply is 0. I.e. requires burning all tokens.
pub fn process_end(ctx: Context<EndEvent>) -> Result<()> {
    // Check authority is the caller
    require_keys_eq!(
        ctx.accounts.authority.key(),
        ctx.accounts.event.authority,
        ChainTicketError::Unauthorised
    );

    let clock = Clock::get()?;
    let rent = Rent::get()?;

    let event_lamports = ctx.accounts.event.get_lamports();
    let mint_lamports = ctx.accounts.mint.get_lamports();

    // Check the event has ended
    require_gte!(
        clock.unix_timestamp,
        ctx.accounts.event.event_date + ctx.accounts.event.refund_period,
        ChainTicketError::EventNotEnded,
    );

    // Check that funds have been withdrawn by comparing against the minimum rent amount
    require_gte!(rent.minimum_balance(EVENT_STATE_SIZE), event_lamports);

    // Check mint supply
    require_eq!(ctx.accounts.mint.supply, 0, ChainTicketError::SupplyNotZero);

    // Close mint
    ctx.accounts
        .mint
        .to_account_info()
        .assign(&ctx.accounts.system_program.key());

    // Zero data
    ctx.accounts.mint.to_account_info().realloc(0, false)?;

    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.mint.key(),
        &ctx.accounts.authority.key(),
        mint_lamports,
    );

    // Transfer lamports to authority
    anchor_lang::solana_program::program::invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        &[&[
            EVENT_SEED,
            ctx.accounts.authority.key().as_ref(),
            &[ctx.accounts.event.bump],
        ]],
    )?;

    // Close event
    ctx.accounts
        .event
        .to_account_info()
        .assign(&ctx.accounts.system_program.key());

    ctx.accounts.event.to_account_info().realloc(0, false)?;

    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.event.key(),
        &ctx.accounts.authority.key(),
        event_lamports,
    );

    anchor_lang::solana_program::program::invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.event.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        &[&[
            EVENT_SEED,
            ctx.accounts.authority.key().as_ref(),
            &[ctx.accounts.event.bump],
        ]],
    )?;

    Ok(())
}
