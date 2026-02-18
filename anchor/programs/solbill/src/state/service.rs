use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ServiceAccount {
    /// The merchant's wallet address (owner/authority).
    pub authority: Pubkey,
    /// The merchant's token account where payments are deposited.
    pub treasury: Pubkey,
    /// The SPL token mint accepted for payments (e.g. USDC).
    pub accepted_mint: Pubkey,
    /// Number of plans created under this service.
    pub plan_count: u16,
    /// Total active subscribers across all plans.
    pub subscriber_count: u32,
    /// Unix timestamp of service creation.
    pub created_at: i64,
    /// PDA bump seed.
    pub bump: u8,
}
