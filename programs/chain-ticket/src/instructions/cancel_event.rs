use {
    crate::{
        constants::{DEPOSIT_AMOUNT, EVENT_SEED, MINT_SEED, PLATFORM_OWNER, VAULT_SEED},
        errors::ChainTicketError,
        state::Event,
        utils::sol_to_lamports,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::Mint,
};

#[derive(Accounts)]
pub struct CancelEvent<'info> {
    /// CHECK: Checked with constraint
    #[account(
        mut, 
        address = PLATFORM_OWNER @ ChainTicketError::Unauthorised
    )]
    platform_owner: UncheckedAccount<'info>,
    #[account(mut, address = event.authority)]
    authority: Signer<'info>,
    #[account(
        mut,
        seeds = [EVENT_SEED, authority.key().as_ref()],
        bump,
        owner = crate::id(),
        close = authority,
    )]
    event: Account<'info, Event>,
    #[account(
        seeds = [MINT_SEED, event.key().as_ref()],
        bump,
        address = event.mint @ ChainTicketError::InvalidMint,
    )]
    mint: Account<'info, Mint>,
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

pub fn process_cancel(ctx: Context<CancelEvent>) -> Result<()> {
    // Ensure token supply == 0 i.e. all tickets have been refunded
    require_eq!(ctx.accounts.mint.supply, 0, ChainTicketError::NonZeroSupply);

    // Forfeit SOL deposit
    let deposit_amount = sol_to_lamports(DEPOSIT_AMOUNT as f64);
    **ctx.accounts.vault.try_borrow_mut_lamports()? -= deposit_amount;
    **ctx.accounts.platform_owner.try_borrow_mut_lamports()? += deposit_amount; 

    // Close event
    //let event_lamports = ctx.accounts.event.get_lamports();
    //ctx.accounts
    //    .event
    //    .to_account_info()
    //    .assign(&ctx.accounts.system_program.key());

    //ctx.accounts.event.to_account_info().realloc(0, false)?;

    //let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
    //    &ctx.accounts.event.key(),
    //    &ctx.accounts.authority.key(),
    //    event_lamports,
    //);

    //anchor_lang::solana_program::program::invoke_signed(
    //    &transfer_ix,
    //    &[
    //        ctx.accounts.event.to_account_info(),
    //        ctx.accounts.authority.to_account_info(),
    //    ],
    //    &[&[
    //        EVENT_SEED,
    //        ctx.accounts.authority.key().as_ref(),
    //        &[ctx.accounts.event.bump],
    //    ]],
    //)?;
    // Close event
    Ok(())
}
