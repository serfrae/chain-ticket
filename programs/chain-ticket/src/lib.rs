use anchor_lang::prelude::*;

mod constants;
mod errors;
mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("AGZeqX1TNnjrH4yFeRHEf2iHnz3ZLX5W94rC1iJuBY8b");

#[program]
pub mod chain_ticket {
    use super::*;

    pub fn init_event(ctx: Context<InitEvent>, data: InitEventFields) -> Result<()> {
        instructions::init::process_init(ctx, data)?;
        Ok(())
    }

    pub fn amend_event(ctx: Context<AmendEvent>, data: AmendEventFields) -> Result<()> {
        instructions::amend_details::process_amend(ctx, data)?;
        Ok(())
    }

    pub fn start_sale(ctx: Context<StartSale>) -> Result<()> {
        instructions::start_sale::process_start(ctx)?;
        Ok(())
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        instructions::buy_ticket::process_buy(ctx)?;
        Ok(())
    }

    pub fn refund_ticket(ctx: Context<RefundTicket>) -> Result<()> {
        instructions::refund_ticket::process_refund(ctx)?;
        Ok(())
    }

    pub fn burn_ticket(ctx: Context<BurnTicket>) -> Result<()> {
        instructions::burn_ticket::process_burn(ctx)?;
        Ok(())
    }

    pub fn delegate_burn(ctx:Context<DelegateBurn>) -> Result<()> {
        instructions::delegate_burn::process_delegate_burn(ctx)?;
        Ok(())
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        instructions::withdraw_funds::process_withdraw(ctx)?;
        Ok(())
    }

    pub fn cancel_event(ctx: Context<CancelEvent>) -> Result<()> {
        instructions::cancel_event::process_cancel(ctx)?;
        Ok(())
    }

    pub fn end_event(ctx: Context<EndEvent>) -> Result<()> {
        instructions::end_event::process_end(ctx)?;
        Ok(())
    }
}
