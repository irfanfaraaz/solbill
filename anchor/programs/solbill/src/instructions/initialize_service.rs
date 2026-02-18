use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::state::ServiceAccount;

#[derive(Accounts)]
pub struct InitializeService<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + ServiceAccount::INIT_SPACE,
        seeds = [b"service", authority.key().as_ref()],
        bump,
    )]
    pub service: Account<'info, ServiceAccount>,

    /// The SPL token mint accepted for payments (e.g. USDC).
    pub accepted_mint: InterfaceAccount<'info, Mint>,

    /// The merchant's token account where payments will be deposited.
    /// Must be owned by authority and use the accepted mint.
    #[account(
        token::mint = accepted_mint,
        token::authority = authority,
        token::token_program = token_program,
    )]
    pub treasury: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeService>) -> Result<()> {
    let service = &mut ctx.accounts.service;
    let clock = Clock::get()?;

    service.authority = ctx.accounts.authority.key();
    service.treasury = ctx.accounts.treasury.key();
    service.accepted_mint = ctx.accounts.accepted_mint.key();
    service.plan_count = 0;
    service.subscriber_count = 0;
    service.created_at = clock.unix_timestamp;
    service.bump = ctx.bumps.service;

    msg!("Service initialized by {}", service.authority);
    Ok(())
}
