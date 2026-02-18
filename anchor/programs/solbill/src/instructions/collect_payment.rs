use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::errors::SolBillError;
use crate::state::{ServiceAccount, SubscriptionAccount, SubscriptionStatus};

#[derive(Accounts)]
pub struct CollectPayment<'info> {
    /// The public crank turner who triggers the payment and receives the reward.
    #[account(mut)]
    pub cranker: Signer<'info>,

    #[account(
        seeds = [b"service", service.authority.as_ref()],
        bump = service.bump,
    )]
    pub service: Account<'info, ServiceAccount>,

    #[account(
        mut,
        seeds = [b"subscription", subscription.subscriber.as_ref(), subscription.plan.as_ref()],
        bump = subscription.bump,
        has_one = service,
    )]
    pub subscription: Account<'info, SubscriptionAccount>,

    /// The subscriber's token account (source of funds).
    #[account(
        mut,
        address = subscription.subscriber_token_account,
    )]
    pub subscriber_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The merchant's treasury token account (destination for main payment).
    #[account(
        mut,
        address = service.treasury,
    )]
    pub treasury: InterfaceAccount<'info, TokenAccount>,

    /// The cranker's token account (destination for bounty/reward).
    #[account(
        mut,
        token::mint = accepted_mint,
    )]
    pub cranker_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The accepted SPL token mint.
    #[account(
        address = service.accepted_mint,
    )]
    pub accepted_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<CollectPayment>) -> Result<()> {
    let clock = Clock::get()?;

    // We access data immutably first for guards and transfer
    {
        let subscription = &ctx.accounts.subscription;

        // --- Guards ---

        // Must be active or past due
        require!(
            subscription.status == SubscriptionStatus::Active
                || subscription.status == SubscriptionStatus::PastDue,
            SolBillError::SubscriptionNotActive,
        );

        // Timing enforcement: cannot bill before due date
        require!(
            clock.unix_timestamp >= subscription.next_billing_timestamp,
            SolBillError::BillingNotDue,
        );

        // Check for max billing cycles limit BEFORE collecting
        if subscription.max_billing_cycles > 0 {
            if subscription.payments_made >= subscription.max_billing_cycles as u32 {
                // This should not happen if status is correctly managed,
                // but as a safety guard against race conditions or manual errors.
                return err!(SolBillError::SubscriptionCompleted);
            }
        }

        // --- Transfer Logic ---
        let subscriber_key = subscription.subscriber;
        let plan_key = subscription.plan;
        let bump = subscription.bump;
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"subscription",
            subscriber_key.as_ref(),
            plan_key.as_ref(),
            &[bump],
        ]];

        crate::instructions::utils::execute_token_transfer(
            &ctx.accounts.token_program,
            &ctx.accounts.subscriber_token_account,
            &ctx.accounts.treasury,
            Some(&ctx.accounts.cranker_token_account),
            &ctx.accounts.accepted_mint,
            &ctx.accounts.subscription.to_account_info(),
            subscription.amount,
            subscription.crank_reward,
            Some(signer_seeds),
        )?;
    }

    // Now borrow mutably to update state
    let subscription = &mut ctx.accounts.subscription;
    let clock = Clock::get()?;

    // --- Update subscription state ---
    subscription.last_payment_timestamp = clock.unix_timestamp;

    // Increment payments made
    subscription.payments_made = subscription
        .payments_made
        .checked_add(1)
        .ok_or(SolBillError::Overflow)?;

    msg!(
        "Payment collected. Total payments made: {}",
        subscription.payments_made
    );

    // Check if we hit the limit
    if subscription.max_billing_cycles > 0 {
        msg!(
            "Checking max cycles: {}/{}",
            subscription.payments_made,
            subscription.max_billing_cycles
        );
        if subscription.payments_made >= subscription.max_billing_cycles as u32 {
            subscription.status = SubscriptionStatus::Completed;
            // Prevent further billing
            subscription.next_billing_timestamp = i64::MAX;
            msg!("Max cycles reached. Status set to Completed.");
        } else {
            // Not yet completed, schedule next
            subscription.next_billing_timestamp = clock
                .unix_timestamp
                .checked_add(subscription.interval)
                .ok_or(SolBillError::Overflow)?;
            subscription.status = SubscriptionStatus::Active;
            msg!(
                "Plan continues. Next billing: {}",
                subscription.next_billing_timestamp
            );
        }
    } else {
        // Infinite
        subscription.next_billing_timestamp = clock
            .unix_timestamp
            .checked_add(subscription.interval)
            .ok_or(SolBillError::Overflow)?;
        subscription.status = SubscriptionStatus::Active;
        msg!(
            "Infinite plan continues. Next billing: {}",
            subscription.next_billing_timestamp
        );
    }

    let treasury_amount = subscription
        .amount
        .checked_sub(subscription.crank_reward)
        .unwrap_or(0);

    msg!(
        "Collection success: Cranker Reward: {}, Treasury: {}, Next billing: {}",
        subscription.crank_reward,
        treasury_amount,
        subscription.next_billing_timestamp,
    );
    Ok(())
}
