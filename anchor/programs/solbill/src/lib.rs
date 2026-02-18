use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

#[cfg(test)]
mod tests;

use instructions::*;

declare_id!("AK2xA7SHMKPqvQEirLUNf4gRQjzpQZT3q6v3d62kLyzx");

#[program]
pub mod solbill {
    use super::*;

    /// Merchant: Create a new billing service.
    pub fn initialize_service(ctx: Context<InitializeService>) -> Result<()> {
        instructions::initialize_service::handler(ctx)
    }

    /// Merchant: Create a subscription plan under the service.
    pub fn create_plan(
        ctx: Context<CreatePlan>,
        name: String,
        amount: u64,
        crank_reward: u64,
        interval: i64,
        grace_period: i64,
    ) -> Result<()> {
        instructions::create_plan::handler(ctx, name, amount, crank_reward, interval, grace_period)
    }

    /// Merchant: Update a plan's fields (does not affect existing subscriptions).
    pub fn update_plan(
        ctx: Context<UpdatePlan>,
        new_amount: Option<u64>,
        new_cranker_reward: Option<u64>,
        new_interval: Option<i64>,
        new_is_active: Option<bool>,
        new_grace_period: Option<i64>,
    ) -> Result<()> {
        instructions::update_plan::handler(
            ctx,
            new_amount,
            new_cranker_reward,
            new_interval,
            new_is_active,
            new_grace_period,
        )
    }

    /// Subscriber: Subscribe to a plan.
    pub fn create_subscription(ctx: Context<CreateSubscription>) -> Result<()> {
        instructions::create_subscription::handler(ctx)
    }

    /// Subscriber: Cancel an active subscription (instant, revokes token delegation).
    pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
        instructions::cancel_subscription::handler(ctx)
    }

    /// Subscriber: Switch to a different plan.
    pub fn change_plan(ctx: Context<ChangePlan>) -> Result<()> {
        instructions::change_plan::handler(ctx)
    }

    /// Merchant/Worker: Collect a due payment from a subscriber.
    pub fn collect_payment(ctx: Context<CollectPayment>) -> Result<()> {
        instructions::collect_payment::handler(ctx)
    }

    /// Anyone: Expire a past-due subscription after grace period.
    pub fn expire_subscription(ctx: Context<ExpireSubscription>) -> Result<()> {
        instructions::expire_subscription::handler(ctx)
    }
}
