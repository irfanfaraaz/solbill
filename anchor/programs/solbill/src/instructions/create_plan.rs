use anchor_lang::prelude::*;

use crate::errors::SolscribeError;
use crate::state::{PlanAccount, ServiceAccount, PLAN_ACCOUNT_SIZE};

#[derive(Accounts)]
pub struct CreatePlan<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"service", authority.key().as_ref()],
        bump = service.bump,
        has_one = authority @ SolscribeError::UnauthorizedAuthority,
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
    crank_reward: u64,
    interval: i64,
    grace_period: i64,
) -> Result<()> {
    require!(
        !name.is_empty() && name.len() <= 32,
        SolscribeError::InvalidPlanName
    );
    require!(amount > 0, SolscribeError::InvalidAmount);
    require!(interval > 0, SolscribeError::InvalidInterval);
    require!(crank_reward < amount, SolscribeError::InvalidCrankReward);

    let plan = &mut ctx.accounts.plan;
    let service = &mut ctx.accounts.service;

    plan.service = service.key();

    // Copy name into fixed-size array, zero-padded
    let mut name_bytes = [0u8; 32];
    let name_raw = name.as_bytes();
    name_bytes[..name_raw.len()].copy_from_slice(name_raw);
    plan.name = name_bytes;

    plan.amount = amount;
    plan.crank_reward = crank_reward;
    plan.interval = interval;
    plan.is_active = true;
    plan.grace_period = grace_period;
    plan.plan_index = service.plan_count;
    plan.bump = ctx.bumps.plan;

    // Increment the service's plan counter
    service.plan_count = service
        .plan_count
        .checked_add(1)
        .ok_or(SolscribeError::Overflow)?;

    msg!(
        "Plan '{}' created (index {}) — {} tokens every {}s (Crank Reward: {})",
        name,
        plan.plan_index,
        plan.amount,
        plan.interval,
        plan.crank_reward,
    );
    Ok(())
}
