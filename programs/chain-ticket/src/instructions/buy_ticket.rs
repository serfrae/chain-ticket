use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token::{
            Token, TokenAccount, Mint, MintTo, mint_to, ApproveChecked, approve_checked, 
            FreezeAccount, freeze_account,
        }, 
        associated_token::AssociatedToken
    },
    crate::{errors::ChainTicketError, state::Event, constants::{EVENT_SEED, MINT_SEED, VAULT_SEED}},
};

#[derive(Accounts)]
pub struct BuyTicket<'info> {
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
    #[account(mut)]
	pub buyer: Signer<'info>,
    #[account(
        init_if_needed, 
        payer = buyer, 
        associated_token::mint = mint, 
        associated_token::authority = buyer,
    )]
    buyer_ata: Account<'info, TokenAccount>,
	system_program: Program<'info, System>,
	token_program: Program<'info, Token>,
	associated_token_program: Program<'info, AssociatedToken>,
}

/// Purchases a ticket by transferring SOL to the event account, and minting a ticket token
/// to the buyer. The ticket's associated token account is then frozen and the event is set
/// as delegate. Necessary for refunds and clean-ups.
pub fn process_buy(ctx: Context<BuyTicket>) -> Result<()> {
    require_eq!(ctx.accounts.event.allow_purchase, true, ChainTicketError::SaleNotStarted);
    require_gte!(
        ctx.accounts.event.num_tickets as u64, 
        ctx.accounts.mint.supply, 
        ChainTicketError::MaxTicketsExceeded
    );

    require_gte!(1, ctx.accounts.buyer_ata.amount, ChainTicketError::AlreadyPurchased);

    // Transfer sol to the Event account
    anchor_lang::solana_program::program::invoke(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.vault.key(),
            ctx.accounts.event.ticket_price,
        ),
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Mint the token (which is the ticket)
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.buyer_ata.to_account_info(),
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

    // Set the delegate to the event account - this is used for chekcing refunds with
    // `refund_all` on the client side as we would burn the ticket token once a refund
    // goes through. Since a user owns the token account however, they COULD choose to remove
    // the Event account as a delegate so we cannot burn the ticket token.
    // However, since the logic checks that the Event account is the delegate, if the user
    // removes the Event account as delegate, they essentially void the right to refund.
    approve_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            ApproveChecked {
                to: ctx.accounts.buyer_ata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                delegate: ctx.accounts.event.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        1,
        0,
    )?;

    // Freeze the ATA so that a user cannot transfer
    freeze_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        FreezeAccount {
            account: ctx.accounts.buyer_ata.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.event.to_account_info(),
        },
        &[&[
            EVENT_SEED,
            ctx.accounts.event.authority.as_ref(),
            &[ctx.accounts.event.bump],
        ]],
    ))?;

    Ok(())
}

