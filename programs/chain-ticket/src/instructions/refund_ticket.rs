use {
    crate::{
        constants::{EVENT_SEED, MINT_SEED, VAULT_SEED},
        errors::ChainTicketError,
        state::Event,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{burn, thaw_account, Burn, Mint, ThawAccount, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct RefundTicket<'info> {
    #[account(mut)]
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
    #[account(
        mut,
        seeds = [MINT_SEED, event.key().as_ref()],
        bump,
        address = event.mint @ ChainTicketError::InvalidMint,
    )]
    pub mint: Account<'info, Mint>,
    /// CHECK: No check required
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Refunds a ticket purchaser. This instruction requires the event account to be set as a
/// delegate for the purchaser's associated token account. Refunds will fail to process if this is
/// not the case. This instruction is called by the authority not by the purchaser hence the need
/// for the event account to be an approved delegate.
pub fn process_refund(ctx: Context<RefundTicket>) -> Result<()> {
    // Thaw token account so ticket can be burnt
    thaw_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        ThawAccount {
            account: ctx.accounts.buyer_ata.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.event.to_account_info(),
        },
        &[&[
            EVENT_SEED,
            ctx.accounts.authority.key().as_ref(),
            &[ctx.accounts.event.bump],
        ]],
    ))?;

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
                ctx.accounts.authority.key().as_ref(),
                &[ctx.accounts.event.bump],
            ]],
        ),
        1,
    )?;

    // Return sol
    **ctx.accounts.vault.try_borrow_mut_lamports()? -= ctx.accounts.event.ticket_price;
    **ctx.accounts.buyer.try_borrow_mut_lamports()? += ctx.accounts.event.ticket_price;

    Ok(())
}
