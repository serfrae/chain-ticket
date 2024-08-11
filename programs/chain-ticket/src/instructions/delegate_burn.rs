use {
    crate::{
        constants::{EVENT_SEED, MINT_SEED},
        errors::ChainTicketError,
        state::Event,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct DelegateBurn<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [
            EVENT_SEED,
            authority.key().as_ref(),
        ],
        bump
    )]
    pub event: Account<'info, Event>,
    #[account(
        mut,
        seeds = [MINT_SEED, event.key().as_ref()],
        bump,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: Only used to derive the target_ata
    pub target_wallet: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = target_wallet,
    )]
    pub target_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Enables the delegate (i.e. the program) to burn tickets. Required to close the mint after
/// the event has ended, which is required to end the event so that addresses can be recycled
/// for by the user for future events
pub fn process_delegate_burn(ctx: Context<DelegateBurn>) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.event.authority,
        ctx.accounts.authority.key(),
        ChainTicketError::Unauthorised
    );

    // Burn ticket token
    burn(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                from: ctx.accounts.target_ata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.event.to_account_info(),
            },
            &[&[
                EVENT_SEED,
                ctx.accounts.authority.key().as_ref(),
                &[ctx.accounts.event.bump],
            ]],
        ),
        1,
    )?;

    Ok(())
}
