use anchor_lang::prelude::*;
use anchor_spl::token_interface::{approve, Approve, Mint, TokenAccount, TokenInterface};

use crate::errors::SolBillError;
use crate::state::{PlanAccount, ServiceAccount, SubscriptionAccount, SubscriptionStatus};

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
        space = 8 + SubscriptionAccount::INIT_SPACE,
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

    /// The merchant's treasury token account (destination for first payment).
    #[account(
        mut,
        address = service.treasury,
    )]
    pub treasury: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateSubscription>) -> Result<()> {
    let plan = &ctx.accounts.plan;
    let clock = Clock::get()?;

    // Initialize the subscription in a scoped block to drop the mutable borrow
    {
        let subscription = &mut ctx.accounts.subscription;
        subscription.subscriber = ctx.accounts.subscriber.key();
        subscription.service = ctx.accounts.service.key();
        subscription.plan = plan.key();
        subscription.subscriber_token_account = ctx.accounts.subscriber_token_account.key();
        subscription.amount = plan.amount;
        subscription.crank_reward = plan.crank_reward;
        subscription.interval = plan.interval;
        subscription.max_billing_cycles = plan.max_billing_cycles;
        subscription.payments_made = 1;
        subscription.bump = ctx.bumps.subscription;

        msg!(
            "Creating subscription. Plan max_cycles: {}",
            plan.max_billing_cycles
        );

        // Logic for One-Time Payments vs Recurring
        if plan.max_billing_cycles > 0 {
            if plan.max_billing_cycles == 1 {
                // If it's a one-time payment, we set the status to Completed immediately
                // because the user pays upfront in this transaction (see execute_token_transfer below).
                subscription.status = SubscriptionStatus::Completed;
                // Prevent future billing
                subscription.next_billing_timestamp = i64::MAX;
                msg!("One-time payment plan. Status set to Completed.");
            } else {
                // It's a finite recurring plan (e.g. 3 months)
                subscription.status = SubscriptionStatus::Active;
                subscription.next_billing_timestamp = clock
                    .unix_timestamp
                    .checked_add(plan.interval)
                    .ok_or(SolBillError::Overflow)?;
                msg!(
                    "Finite plan ({} cycles). Status Active. Next bill: {}",
                    plan.max_billing_cycles,
                    subscription.next_billing_timestamp
                );
            }
        } else {
            // Infinite recurring
            subscription.status = SubscriptionStatus::Active;
            subscription.next_billing_timestamp = clock
                .unix_timestamp
                .checked_add(plan.interval)
                .ok_or(SolBillError::Overflow)?;
            msg!(
                "Infinite plan. Status Active. Next bill: {}",
                subscription.next_billing_timestamp
            );
        }
    }

    // Approve the subscription PDA as delegate on subscriber's token account
    approve(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Approve {
                to: ctx.accounts.subscriber_token_account.to_account_info(),
                delegate: ctx.accounts.subscription.to_account_info(),
                authority: ctx.accounts.subscriber.to_account_info(),
            },
        ),
        plan.amount,
    )?;

    // Execute first payment upfront (No crank reward for self-execution)
    crate::instructions::utils::execute_token_transfer(
        &ctx.accounts.token_program,
        &ctx.accounts.subscriber_token_account,
        &ctx.accounts.treasury,
        None, // No cranker for first payment
        &ctx.accounts.accepted_mint,
        &ctx.accounts.subscriber.to_account_info(), // Authority is the user
        plan.amount, // Use plan.amount instead of subscription.amount to avoid borrow
        0,           // No reward split
        None,        // No seeds needed (direct user signature)
    )?;

    // Increment service subscriber count
    let service = &mut ctx.accounts.service;
    service.subscriber_count = service
        .subscriber_count
        .checked_add(1)
        .ok_or(SolBillError::Overflow)?;

    msg!(
        "Subscription created & paid: {} -> plan {} (next billing: {})",
        ctx.accounts.subscriber.key(),
        plan.plan_index,
        i64::MAX // Placeholder for log since we dropped subscription borrow
    );
    Ok(())
}
