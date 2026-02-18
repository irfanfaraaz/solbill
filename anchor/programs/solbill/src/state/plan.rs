use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PlanAccount {
    /// Parent `ServiceAccount` pubkey.
    pub service: Pubkey,
    /// Plan name, fixed 32 bytes (UTF-8, zero-padded).
    pub name: [u8; 32],
    /// Payment amount per interval (smallest token unit, e.g. 1_000_000 = 1 USDC).
    pub amount: u64,
    /// Reward paid to the cranker (caller) for processing payment.
    pub crank_reward: u64,
    /// Billing interval in seconds (e.g. 2_592_000 = 30 days).
    pub interval: i64,
    /// Whether new subscriptions can be created for this plan.
    pub is_active: bool,
    /// Seconds after due date before auto-cancellation.
    pub grace_period: i64,
    /// Index of this plan within the service (used in PDA seeds).
    pub plan_index: u16,
    /// Limit on number of billing cycles (0 = infinite, 1 = one-time).
    pub max_billing_cycles: u64,
    /// PDA bump seed.
    pub bump: u8,
}
