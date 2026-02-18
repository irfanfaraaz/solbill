use anchor_lang::prelude::*;

use crate::errors::SolBillError;
use crate::state::{PlanAccount, SubscriptionAccount, SubscriptionStatus};

#[derive(Accounts)]
pub struct ExpireSubscription<'info> {
    /// Anyone can call this (permissionless crank).
    pub cranker: Signer<'info>,

    #[account(
        seeds = [b"plan", plan.service.as_ref(), plan.plan_index.to_le_bytes().as_ref()],
        bump = plan.bump,
    )]
    pub plan: Account<'info, PlanAccount>,

    #[account(
        mut,
        seeds = [b"subscription", subscription.subscriber.as_ref(), subscription.plan.as_ref()],
        bump = subscription.bump,
        constraint = subscription.plan == plan.key(),
        constraint = subscription.status == SubscriptionStatus::PastDue @ SolBillError::NotPastDue,
    )]
    pub subscription: Account<'info, SubscriptionAccount>,
}

pub fn handler(ctx: Context<ExpireSubscription>) -> Result<()> {
    let subscription = &mut ctx.accounts.subscription;
    let plan = &ctx.accounts.plan;
    let clock = Clock::get()?;

    // Grace period must have elapsed
    let expiry_time = subscription
        .next_billing_timestamp
        .checked_add(plan.grace_period)
        .ok_or(SolBillError::Overflow)?;

    require!(
        clock.unix_timestamp >= expiry_time,
        SolBillError::GracePeriodNotElapsed,
    );

    subscription.status = SubscriptionStatus::Expired;

    msg!(
        "Subscription expired: {} (was past due since {})",
        subscription.subscriber,
        subscription.next_billing_timestamp,
    );
    Ok(())
}
