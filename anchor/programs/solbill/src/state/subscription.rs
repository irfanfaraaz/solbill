use anchor_lang::prelude::*;

/// Subscription lifecycle states.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Cancelled,
    Expired,
    Completed,
}

#[account]
#[derive(InitSpace)]
pub struct SubscriptionAccount {
    /// The subscriber's wallet address.
    pub subscriber: Pubkey,
    /// Parent `ServiceAccount` pubkey.
    pub service: Pubkey,
    /// The `PlanAccount` this subscription is linked to.
    pub plan: Pubkey,
    /// The subscriber's token account (source of funds).
    pub subscriber_token_account: Pubkey,
    /// Locked-in payment amount (copied from Plan at creation).
    pub amount: u64,
    /// Reward paid to the cranker (copied from Plan at creation).
    pub crank_reward: u64,
    /// Locked-in billing interval in seconds (copied from Plan at creation).
    pub interval: i64,
    /// Unix timestamp when the next payment is due.
    pub next_billing_timestamp: i64,
    /// Unix timestamp of the last successful payment.
    pub last_payment_timestamp: i64,
    /// Unix timestamp of subscription creation.
    pub created_at: i64,
    /// Current status of the subscription.
    pub status: SubscriptionStatus,
    /// Total number of successful payments collected.
    pub payments_made: u32,
    /// Limit on number of billing cycles (0 = infinite).
    pub max_billing_cycles: u64,
    /// PDA bump seed.
    pub bump: u8,
}
