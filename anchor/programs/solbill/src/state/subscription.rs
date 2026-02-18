use anchor_lang::prelude::*;

/// Subscription lifecycle states.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Cancelled,
    Expired,
}

/// Size: 8 (discriminator) + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 (enum) + 4 + 1 = 190
pub const SUBSCRIPTION_ACCOUNT_SIZE: usize =
    8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 4 + 1;

#[account]
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
    /// PDA bump seed.
    pub bump: u8,
}
