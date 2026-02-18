use anchor_lang::prelude::*;
use anchor_spl::token_interface::{approve, revoke, Approve, Revoke, TokenAccount, TokenInterface};

use crate::errors::SolscribeError;
use crate::state::{PlanAccount, ServiceAccount, SubscriptionAccount, SubscriptionStatus};

#[derive(Accounts)]
pub struct ChangePlan<'info> {
    pub subscriber: Signer<'info>,

    #[account(
        seeds = [b"service", service.authority.as_ref()],
        bump = service.bump,
    )]
    pub service: Account<'info, ServiceAccount>,

    /// The old plan (validated via subscription.plan).
    #[account(
        seeds = [b"plan", service.key().as_ref(), old_plan.plan_index.to_le_bytes().as_ref()],
        bump = old_plan.bump,
        has_one = service,
    )]
    pub old_plan: Account<'info, PlanAccount>,

    /// The new plan to switch to.
    #[account(
        seeds = [b"plan", service.key().as_ref(), new_plan.plan_index.to_le_bytes().as_ref()],
        bump = new_plan.bump,
        has_one = service,
        constraint = new_plan.is_active @ SolscribeError::PlanNotActive,
    )]
    pub new_plan: Account<'info, PlanAccount>,

    #[account(
        mut,
        seeds = [b"subscription", subscriber.key().as_ref(), old_plan.key().as_ref()],
        bump = subscription.bump,
        has_one = subscriber,
        has_one = service,
        constraint = subscription.plan == old_plan.key(),
        constraint = subscription.status == SubscriptionStatus::Active @ SolscribeError::SubscriptionNotActive,
    )]
    pub subscription: Account<'info, SubscriptionAccount>,

    /// The subscriber's token account.
    #[account(
        mut,
        address = subscription.subscriber_token_account,
    )]
    pub subscriber_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: The subscription PDA used as delegate.
    #[account(
        seeds = [b"subscription", subscriber.key().as_ref(), old_plan.key().as_ref()],
        bump,
    )]
    pub delegate: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<ChangePlan>) -> Result<()> {
    let new_plan = &ctx.accounts.new_plan;
    let subscription = &mut ctx.accounts.subscription;

    // Update subscription to new plan terms (effective next cycle)
    subscription.plan = new_plan.key();
    subscription.amount = new_plan.amount;
    subscription.interval = new_plan.interval;

    // Revoke old approval and set new one for the new amount
    revoke(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Revoke {
            source: ctx.accounts.subscriber_token_account.to_account_info(),
            authority: ctx.accounts.subscriber.to_account_info(),
        },
    ))?;

    approve(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Approve {
                to: ctx.accounts.subscriber_token_account.to_account_info(),
                delegate: ctx.accounts.delegate.to_account_info(),
                authority: ctx.accounts.subscriber.to_account_info(),
            },
        ),
        new_plan.amount,
    )?;

    msg!(
        "Subscription plan changed: {} → plan {} ({} tokens/{}s)",
        subscription.subscriber,
        new_plan.plan_index,
        new_plan.amount,
        new_plan.interval,
    );
    Ok(())
}
