use {
    crate::{
        constants::{DEPOSIT_AMOUNT, EVENT_SEED, VAULT_SEED},
        errors::ChainTicketError,
        state::Event,
        utils::sol_to_lamports,
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct StartSale<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    #[account(
        mut,
        seeds = [
            EVENT_SEED,
            authority.key().as_ref(),
        ],
        bump
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
    system_program: Program<'info, System>,
}

/// Changes the field in the event state `allow_purchase` to true, event organiser is charged
/// the deposit amount, will fail if the event organiser has insufficient balance in their wallet
pub fn process_start(ctx: Context<StartSale>) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.authority.key(),
        ctx.accounts.event.authority,
        ChainTicketError::Unauthorised
    );

    anchor_lang::solana_program::program::invoke(
        &anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.vault.key(),
            sol_to_lamports(DEPOSIT_AMOUNT as f64),
        ),
        &[
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    ctx.accounts.event.allow_purchase = true;
    Ok(())
}
