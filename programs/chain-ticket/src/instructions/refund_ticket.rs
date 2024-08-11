use {
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount, Burn, Mint, burn},
    crate::{state::Event, constants::EVENT_SEED},
};

#[derive(Accounts)]
pub struct RefundTicket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub event: Account<'info, Event>,
    pub mint: Account<'info, Mint>,
    /// CHECK: No check required
    #[account(mut)]
    pub buyer: AccountInfo<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Refunds a ticket purchaser. This instruction requires the event account to be set as a
/// delegate for the purchaser's associated token account. Refunds will fail to process if this is
/// not the case. This instruction is called by the authority not by the purchaser hence the need
/// for the event account to be an approved delegate.
pub fn process_refund(ctx: Context<RefundTicket>) -> Result<()> {
    // Burn ticket
    burn(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.buyer_ata.to_account_info(),
                authority: ctx.accounts.event.to_account_info(),
            },
            &[&[
                EVENT_SEED,
                ctx.accounts.event.authority.as_ref(),
                &[ctx.accounts.event.bump],
            ]],
        ),
        1,
    )?;

    // Return sol
    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.event.key(),
            &ctx.accounts.buyer.key(),
            ctx.accounts.event.ticket_price,
        ),
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.event.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[
            EVENT_SEED,
            ctx.accounts.event.authority.as_ref(),
            &[ctx.accounts.event.bump],
        ]],
    )?;

    Ok(())
}
