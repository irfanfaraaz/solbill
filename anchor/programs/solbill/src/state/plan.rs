use anchor_lang::prelude::*;

/// Size: 8 (discriminator) + 32 + 32 + 8 + 8 + 1 + 8 + 2 + 1 + 8 = 108
pub const PLAN_ACCOUNT_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 1 + 8 + 2 + 1 + 8;

#[account]
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
    /// PDA bump seed.
    pub bump: u8,
}
