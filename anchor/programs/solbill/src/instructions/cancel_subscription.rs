use anchor_lang::prelude::*;
use anchor_spl::token_interface::{revoke, Revoke, TokenAccount, TokenInterface};

use crate::errors::SolBillError;
use crate::state::{ServiceAccount, SubscriptionAccount, SubscriptionStatus};

#[derive(Accounts)]
pub struct CancelSubscription<'info> {
    pub subscriber: Signer<'info>,

    #[account(
        mut,
        seeds = [b"service", service.authority.as_ref()],
        bump = service.bump,
    )]
    pub service: Account<'info, ServiceAccount>,

    #[account(
        mut,
        seeds = [b"subscription", subscriber.key().as_ref(), subscription.plan.as_ref()],
        bump = subscription.bump,
        has_one = subscriber,
        has_one = service,
        close = subscriber,
        constraint = subscription.status != SubscriptionStatus::Cancelled @ SolBillError::AlreadyCancelled,
    )]
    pub subscription: Account<'info, SubscriptionAccount>,

    /// The subscriber's token account to revoke delegation from.
    #[account(
        mut,
        address = subscription.subscriber_token_account,
    )]
    pub subscriber_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<CancelSubscription>) -> Result<()> {
    let subscription = &mut ctx.accounts.subscription;

    // Set status to cancelled
    subscription.status = SubscriptionStatus::Cancelled;

    // Revoke the token delegation
    revoke(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Revoke {
            source: ctx.accounts.subscriber_token_account.to_account_info(),
            authority: ctx.accounts.subscriber.to_account_info(),
        },
    ))?;

    // Decrement service subscriber count
    let service = &mut ctx.accounts.service;
    service.subscriber_count = service.subscriber_count.saturating_sub(1);

    msg!("Subscription cancelled: {}", subscription.subscriber,);
    Ok(())
}
