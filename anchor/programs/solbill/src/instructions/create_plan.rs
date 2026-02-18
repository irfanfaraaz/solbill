use anchor_lang::prelude::*;

use crate::errors::SolBillError;
use crate::state::{PlanAccount, ServiceAccount, PLAN_ACCOUNT_SIZE};

#[derive(Accounts)]
pub struct CreatePlan<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"service", authority.key().as_ref()],
        bump = service.bump,
        has_one = authority @ SolBillError::UnauthorizedAuthority,
    )]
    pub service: Account<'info, ServiceAccount>,

    #[account(
        init,
        payer = authority,
        space = PLAN_ACCOUNT_SIZE,
        seeds = [b"plan", service.key().as_ref(), service.plan_count.to_le_bytes().as_ref()],
        bump,
    )]
    pub plan: Account<'info, PlanAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreatePlan>,
    name: String,
    amount: u64,
    interval: i64,
    grace_period: i64,
) -> Result<()> {
    require!(
        !name.is_empty() && name.len() <= 32,
        SolBillError::InvalidPlanName
    );
    require!(amount > 0, SolBillError::InvalidAmount);
    require!(interval > 0, SolBillError::InvalidInterval);

    let plan = &mut ctx.accounts.plan;
    let service = &mut ctx.accounts.service;

    plan.service = service.key();

    // Copy name into fixed-size array, zero-padded
    let mut name_bytes = [0u8; 32];
    let name_raw = name.as_bytes();
    name_bytes[..name_raw.len()].copy_from_slice(name_raw);
    plan.name = name_bytes;

    plan.amount = amount;
    plan.interval = interval;
    plan.is_active = true;
    plan.grace_period = grace_period;
    plan.plan_index = service.plan_count;
    plan.bump = ctx.bumps.plan;

    // Increment the service's plan counter
    service.plan_count = service
        .plan_count
        .checked_add(1)
        .ok_or(SolBillError::Overflow)?;

    msg!(
        "Plan '{}' created (index {}) — {} tokens every {}s",
        name,
        plan.plan_index,
        plan.amount,
        plan.interval,
    );
    Ok(())
}
