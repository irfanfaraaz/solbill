use anchor_lang::prelude::*;

use crate::errors::SolBillError;
use crate::state::{PlanAccount, ServiceAccount};

#[derive(Accounts)]
pub struct UpdatePlan<'info> {
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"service", authority.key().as_ref()],
        bump = service.bump,
        has_one = authority @ SolBillError::UnauthorizedAuthority,
    )]
    pub service: Account<'info, ServiceAccount>,

    #[account(
        mut,
        seeds = [b"plan", service.key().as_ref(), plan.plan_index.to_le_bytes().as_ref()],
        bump = plan.bump,
        has_one = service,
    )]
    pub plan: Account<'info, PlanAccount>,
}

pub fn handler(
    ctx: Context<UpdatePlan>,
    new_amount: Option<u64>,
    new_interval: Option<i64>,
    new_is_active: Option<bool>,
    new_grace_period: Option<i64>,
) -> Result<()> {
    let plan = &mut ctx.accounts.plan;

    if let Some(amount) = new_amount {
        require!(amount > 0, SolBillError::InvalidAmount);
        plan.amount = amount;
    }
    if let Some(interval) = new_interval {
        require!(interval > 0, SolBillError::InvalidInterval);
        plan.interval = interval;
    }
    if let Some(is_active) = new_is_active {
        plan.is_active = is_active;
    }
    if let Some(grace_period) = new_grace_period {
        plan.grace_period = grace_period;
    }

    msg!(
        "Plan {} updated — amount: {}, interval: {}s, active: {}",
        plan.plan_index,
        plan.amount,
        plan.interval,
        plan.is_active,
    );
    Ok(())
}
