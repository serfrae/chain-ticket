use {
    anchor_lang::prelude::*,
    anchor_spl::{
        metadata::{
            create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3, Metadata
        },
        token::{Token, Mint},
    },
    crate::{
        constants::{
            EVENT_SEED, VAULT_SEED, MINT_SEED, METADATA_SEED, EVENT_STATE_SIZE, SECONDS_PER_DAY,
        },
        errors::ChainTicketError,
        state::Event,
    },
};

#[derive(Accounts)]
//#[instruction(data: InitEventFields)]
pub struct InitEvent<'info> {
	#[account(mut)]
	authority: Signer<'info>,

	#[account(
        init, 
        payer = authority, 
        seeds = [EVENT_SEED, authority.key.as_ref()],
        bump, 
        space = 8 + EVENT_STATE_SIZE
    )]
	pub event: Account<'info, Event>,

    /// CHECK: Address is derived and is a native vault,
    /// in order to facilitate transfers from the vault
    /// it must have no data and thus no discriminator.
    #[account(
        init,
        payer = authority,
        seeds = [VAULT_SEED, event.key().as_ref()],
        bump,
        space = 0,
    )]
    pub vault: UncheckedAccount<'info>,

    #[account(
        init_if_needed, 
        payer = authority, 
        seeds = [MINT_SEED, event.key().as_ref()],
        bump,
        mint::decimals = 0, 
        mint::authority = event, 
        mint::freeze_authority = event
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: Safe - PDA
    #[account(
        mut,
        seeds = [
            METADATA_SEED,
            token_metadata_program.key().as_ref(), 
            mint.key().as_ref()
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: UncheckedAccount<'info>, 

	pub system_program: Program<'info, System>,
	pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>, 
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitEventFields {
    event_name: String,
    event_symbol: String,
    image_uri: String,
    metadata_uri: String,
    event_date: i64,
    ticket_price: u64,
    num_tickets: u32,
    refund_period: i64,
}

pub fn process_init(ctx: Context<InitEvent>, data: InitEventFields) -> Result<()> {
    require_eq!(ctx.accounts.mint.supply, 0, ChainTicketError::NonZeroSupply);
    if data.refund_period < SECONDS_PER_DAY * 2 {
        ctx.accounts.event.refund_period = SECONDS_PER_DAY * 2;
    }
    ctx.accounts.event.refund_period = data.refund_period;
    ctx.accounts.event.bump = ctx.bumps.event;
    ctx.accounts.event.authority = ctx.accounts.authority.key();
    ctx.accounts.event.vault = ctx.accounts.vault.key();
    ctx.accounts.event.mint = ctx.accounts.mint.key();
    ctx.accounts.event.allow_purchase = false;
    ctx.accounts.event.event_date = data.event_date;
    ctx.accounts.event.ticket_price = data.ticket_price;
    ctx.accounts.event.num_tickets = data.num_tickets;

    // Create token metadata (used for wallets to read name, symbol, and token image)
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.event.to_account_info(),
                update_authority: ctx.accounts.event.to_account_info(),
                payer: ctx.accounts.authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &[&[
                EVENT_SEED,
                ctx.accounts.authority.key().as_ref(),
                &[ctx.bumps.event],
            ]],
        ),
        DataV2 {
            name: data.event_name,
            symbol: data.event_symbol,
            uri: data.image_uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false,
        true,
        None,
    )?;

    Ok(())
}
