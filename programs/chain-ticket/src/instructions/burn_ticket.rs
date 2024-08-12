use {
    crate::{constants::{EVENT_SEED, MINT_SEED}, state::Event},
    anchor_lang::prelude::*,
    anchor_spl::token::{
        burn, close_account, thaw_account, Burn, CloseAccount, Mint, ThawAccount, Token,
        TokenAccount,
    },
};

#[derive(Accounts)]
pub struct BurnTicket<'info> {
    event: Account<'info, Event>,
    #[account(
        mut,
        seeds = [MINT_SEED, event.key().as_ref()],
        bump,
    )]
    mint: Account<'info, Mint>,
    #[account(mut)]
    ticket_holder: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = ticket_holder,
    )]
    ticket_holder_ata: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}

/// Burns a token ticket and closes it's associated token account so that the user
/// can reclaim rent used for the token account. This function is required as the token
/// account is frozen upon creation to prevent users from transferring tickets. Thus, this function
/// first thaws the token account, then performs the ticket burn and finally closes the token
/// account
pub fn process_burn(ctx: Context<BurnTicket>) -> Result<()> {
    // Thaw token acount
    thaw_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        ThawAccount {
            account: ctx.accounts.ticket_holder_ata.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.event.to_account_info(),
        },
        &[&[
            EVENT_SEED,
            ctx.accounts.event.authority.as_ref(),
            &[ctx.accounts.event.bump],
        ]],
    ))?;

    // Burn ticket
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.ticket_holder_ata.to_account_info(),
                authority: ctx.accounts.ticket_holder.to_account_info(),
            },
        ),
        1,
    )?;

    // Close token account
    close_account(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.ticket_holder_ata.to_account_info(),
            destination: ctx.accounts.ticket_holder.to_account_info(),
            authority: ctx.accounts.ticket_holder.to_account_info(),
        },
    ))?;

    Ok(())
}
