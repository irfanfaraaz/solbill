use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::errors::SolBillError;
use crate::state::{ServiceAccount, SubscriptionAccount, SubscriptionStatus};

#[derive(Accounts)]
pub struct CollectPayment<'info> {
    /// The merchant (or their worker) who triggers billing.
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"service", authority.key().as_ref()],
        bump = service.bump,
        has_one = authority @ SolBillError::UnauthorizedAuthority,
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

    /// The merchant's treasury token account (destination).
    #[account(
        mut,
        address = service.treasury,
    )]
    pub treasury: InterfaceAccount<'info, TokenAccount>,

    /// The accepted SPL token mint.
    #[account(
        address = service.accepted_mint,
    )]
    pub accepted_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: The subscription PDA used as delegate to sign the transfer.
    #[account(
        seeds = [b"subscription", subscription.subscriber.as_ref(), subscription.plan.as_ref()],
        bump = subscription.bump,
    )]
    pub delegate: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<CollectPayment>) -> Result<()> {
    let subscription = &mut ctx.accounts.subscription;
    let clock = Clock::get()?;

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

    // --- Transfer via delegate PDA ---
    let subscriber_key = subscription.subscriber;
    let plan_key = subscription.plan;
    let bump = subscription.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"subscription",
        subscriber_key.as_ref(),
        plan_key.as_ref(),
        &[bump],
    ]];

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.subscriber_token_account.to_account_info(),
                to: ctx.accounts.treasury.to_account_info(),
                authority: ctx.accounts.delegate.to_account_info(),
                mint: ctx.accounts.accepted_mint.to_account_info(),
            },
            signer_seeds,
        ),
        subscription.amount,
        ctx.accounts.accepted_mint.decimals,
    )?;

    // --- Update subscription state ---
    subscription.last_payment_timestamp = clock.unix_timestamp;
    subscription.next_billing_timestamp = clock
        .unix_timestamp
        .checked_add(subscription.interval)
        .ok_or(SolBillError::Overflow)?;
    subscription.payments_made = subscription
        .payments_made
        .checked_add(1)
        .ok_or(SolBillError::Overflow)?;
    subscription.status = SubscriptionStatus::Active;

    msg!(
        "Payment collected: {} tokens from {} (payment #{}, next billing: {})",
        subscription.amount,
        subscription.subscriber,
        subscription.payments_made,
        subscription.next_billing_timestamp,
    );
    Ok(())
}
