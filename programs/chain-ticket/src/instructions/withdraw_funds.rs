use {
    crate::{
        constants::{DEPOSIT_AMOUNT, EVENT_SEED, EVENT_STATE_SIZE, FEE, PLATFORM_OWNER},
        errors::ChainTicketError,
        state::Event,
        utils::sol_to_lamports,
    },
    anchor_lang::prelude::*,
    std::str::FromStr,
};

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    /// CHECK: Checked against constant
    #[account(mut)]
    pub platform_owner: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [EVENT_SEED, event.key().as_ref()],
        bump,
    )]
    pub event: Account<'info, Event>,
    pub system_program: Program<'info, System>,
}

pub fn process_withdraw(ctx: Context<WithdrawFunds>) -> Result<()> {
    // Check platform owner address
    require_keys_eq!(
        ctx.accounts.platform_owner.key(),
        Pubkey::from_str(PLATFORM_OWNER).map_err(|_| ChainTicketError::PubkeyParseError)?,
        ChainTicketError::IncorrectPlatformOwner
    );

    // Check authority
    require_keys_eq!(
        ctx.accounts.authority.key(),
        ctx.accounts.event.authority,
        ChainTicketError::Unauthorised
    );

    let rent = Rent::get()?;
    let deposit_amount = sol_to_lamports(DEPOSIT_AMOUNT as f64);
    let rent_lamports = rent.minimum_balance(EVENT_STATE_SIZE);

    let proceeds = ctx
        .accounts
        .event
        .get_lamports()
        .checked_sub(deposit_amount)
        .ok_or(ChainTicketError::Overflow)?
        .checked_sub(rent_lamports)
        .ok_or(ChainTicketError::Overflow)?;

    let platform_fee = proceeds
        .checked_div(FEE * 100)
        .ok_or(ChainTicketError::Overflow)?;

    // Transfer platform fee
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.event.key(),
            &ctx.accounts.platform_owner.key(),
            platform_fee,
        ),
        &[
            ctx.accounts.event.to_account_info(),
            ctx.accounts.platform_owner.to_account_info(),
        ],
        &[&[
            EVENT_SEED,
            ctx.accounts.event.authority.as_ref(),
            &[ctx.accounts.event.bump],
        ]],
    )?;

    // Transfer proceeds
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.event.key(),
            &ctx.accounts.authority.key(),
            proceeds + deposit_amount,
        ),
        &[
            ctx.accounts.event.to_account_info(),
            ctx.accounts.authority.to_account_info(),
        ],
        &[&[
            EVENT_SEED,
            ctx.accounts.event.authority.as_ref(),
            &[ctx.accounts.event.bump],
        ]],
    )?;

    Ok(())
}
