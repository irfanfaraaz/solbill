use anchor_lang::prelude::*;
use anchor_spl::token_interface::{approve, Approve, Mint, TokenAccount, TokenInterface};

use crate::errors::SolBillError;
use crate::state::{
    PlanAccount, ServiceAccount, SubscriptionAccount, SubscriptionStatus, SUBSCRIPTION_ACCOUNT_SIZE,
};

#[derive(Accounts)]
pub struct CreateSubscription<'info> {
    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(
        mut,
        seeds = [b"service", service.authority.as_ref()],
        bump = service.bump,
    )]
    pub service: Account<'info, ServiceAccount>,

    #[account(
        seeds = [b"plan", service.key().as_ref(), plan.plan_index.to_le_bytes().as_ref()],
        bump = plan.bump,
        has_one = service,
        constraint = plan.is_active @ SolBillError::PlanNotActive,
    )]
    pub plan: Account<'info, PlanAccount>,

    #[account(
        init,
        payer = subscriber,
        space = SUBSCRIPTION_ACCOUNT_SIZE,
        seeds = [b"subscription", subscriber.key().as_ref(), plan.key().as_ref()],
        bump,
    )]
    pub subscription: Account<'info, SubscriptionAccount>,

    /// The subscriber's token account (source of funds).
    #[account(
        mut,
        token::mint = accepted_mint,
        token::authority = subscriber,
        token::token_program = token_program,
    )]
    pub subscriber_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The SPL mint accepted by the service.
    #[account(
        address = service.accepted_mint,
    )]
    pub accepted_mint: InterfaceAccount<'info, Mint>,

    /// The subscription PDA will be the delegate.
    /// CHECK: This is the subscription PDA used as delegate for token approval.
    #[account(
        seeds = [b"subscription", subscriber.key().as_ref(), plan.key().as_ref()],
        bump,
    )]
    pub delegate: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateSubscription>) -> Result<()> {
    let plan = &ctx.accounts.plan;
    let clock = Clock::get()?;

    // Initialize the subscription
    let subscription = &mut ctx.accounts.subscription;
    subscription.subscriber = ctx.accounts.subscriber.key();
    subscription.service = ctx.accounts.service.key();
    subscription.plan = plan.key();
    subscription.subscriber_token_account = ctx.accounts.subscriber_token_account.key();
    subscription.amount = plan.amount;
    subscription.interval = plan.interval;
    subscription.next_billing_timestamp = clock
        .unix_timestamp
        .checked_add(plan.interval)
        .ok_or(SolBillError::Overflow)?;
    subscription.last_payment_timestamp = 0;
    subscription.created_at = clock.unix_timestamp;
    subscription.status = SubscriptionStatus::Active;
    subscription.payments_made = 0;
    subscription.bump = ctx.bumps.subscription;

    // Approve the subscription PDA as delegate on subscriber's token account
    approve(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Approve {
                to: ctx.accounts.subscriber_token_account.to_account_info(),
                delegate: ctx.accounts.delegate.to_account_info(),
                authority: ctx.accounts.subscriber.to_account_info(),
            },
        ),
        plan.amount,
    )?;

    // Increment service subscriber count
    let service = &mut ctx.accounts.service;
    service.subscriber_count = service
        .subscriber_count
        .checked_add(1)
        .ok_or(SolBillError::Overflow)?;

    msg!(
        "Subscription created: {} → plan {} (next billing: {})",
        subscription.subscriber,
        plan.plan_index,
        subscription.next_billing_timestamp,
    );
    Ok(())
}
