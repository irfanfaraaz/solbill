use anchor_lang::prelude::*;

#[error_code]
pub enum SolBillError {
    #[msg("Billing is not yet due")]
    BillingNotDue,
    #[msg("Subscription is not active")]
    SubscriptionNotActive,
    #[msg("Unauthorized authority")]
    UnauthorizedAuthority,
    #[msg("Plan is not active")]
    PlanNotActive,
    #[msg("Grace period has not elapsed")]
    GracePeriodNotElapsed,
    #[msg("Invalid plan name — must be non-empty and at most 32 bytes")]
    InvalidPlanName,
    #[msg("Subscription is already cancelled")]
    AlreadyCancelled,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Invalid amount — must be greater than zero")]
    InvalidAmount,
    #[msg("Invalid interval — must be greater than zero")]
    InvalidInterval,
    #[msg("Subscription is not past due")]
    NotPastDue,
    #[msg("Invalid crank reward — must be less than plan amount")]
    InvalidCrankReward,
    #[msg("Subscription has completed all billing cycles")]
    SubscriptionCompleted,
}
