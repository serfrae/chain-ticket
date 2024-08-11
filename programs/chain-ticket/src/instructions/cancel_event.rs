use {
    crate::{
        constants::{DEPOSIT_AMOUNT, EVENT_SEED, PLATFORM_OWNER, MINT_SEED},
        errors::ChainTicketError,
        state::Event,
        utils::sol_to_lamports,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::Mint,
    std::str::FromStr,
};

#[derive(Accounts)]
pub struct CancelEvent<'info> {
    /// CHECK: Checked against constant
    #[account(mut)]
    platform_owner: UncheckedAccount<'info>,
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        mut,
        seeds = [EVENT_SEED, authority.key().as_ref()],
        bump,
    )]
    event: Account<'info, Event>,
    #[account(
        seeds = [MINT_SEED, event.key().as_ref()],
        bump,
    )]
    mint: Account<'info, Mint>,
    system_program: Program<'info, System>,
}

pub fn process_cancel(ctx: Context<CancelEvent>) -> Result<()> {
    // Ensure token supply == 0 i.e. all tickets have been refunded
    require_eq!(ctx.accounts.mint.supply, 0, ChainTicketError::SupplyNotZero);

    // Check platform owner address is correct
    require_keys_eq!(
        ctx.accounts.platform_owner.key(),
        Pubkey::from_str(PLATFORM_OWNER).map_err(|_| ChainTicketError::PubkeyParseError)?,
        ChainTicketError::IncorrectPlatformOwner,
    );

    // Check authority is correct
    require_keys_eq!(
        ctx.accounts.authority.key(),
        ctx.accounts.event.authority,
        ChainTicketError::Unauthorised
    );

    // Forfeit SOL deposit
    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.event.key(),
        &ctx.accounts.platform_owner.key(),
        sol_to_lamports(DEPOSIT_AMOUNT as f64),
    );

    anchor_lang::solana_program::program::invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.event.to_account_info(),
            ctx.accounts.platform_owner.to_account_info(),
        ],
        &[&[
            ctx.accounts.authority.key().as_ref(),
            EVENT_SEED,
            &[ctx.accounts.event.bump],
        ]],
    )?;

    // Close mint
    let mint_lamports = ctx.accounts.mint.get_lamports();
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
    let event_lamports = ctx.accounts.event.get_lamports();
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
    // Close event
    Ok(())
}
