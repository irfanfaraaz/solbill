use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::errors::SolscribeError;

pub fn execute_token_transfer<'info>(
    token_program: &Interface<'info, TokenInterface>,
    from: &InterfaceAccount<'info, TokenAccount>,
    to_treasury: &InterfaceAccount<'info, TokenAccount>,
    to_cranker: Option<&InterfaceAccount<'info, TokenAccount>>,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &AccountInfo<'info>,
    amount: u64,
    crank_reward: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    // Calculate treasury amount first
    let treasury_amount = if to_cranker.is_some() && crank_reward > 0 {
        amount
            .checked_sub(crank_reward)
            .ok_or(SolscribeError::Overflow)?
    } else {
        amount
    };

    // 1. Pay the Cranker their reward (if applicable)
    if let Some(cranker_acc) = to_cranker {
        if crank_reward > 0 {
            let cpi_accounts = TransferChecked {
                from: from.to_account_info(),
                to: cranker_acc.to_account_info(),
                authority: authority.clone(),
                mint: mint.to_account_info(),
            };
            let cpi_p = token_program.to_account_info();

            if let Some(seeds) = signer_seeds {
                let cpi_ctx = CpiContext::new_with_signer(cpi_p, cpi_accounts, seeds);
                transfer_checked(cpi_ctx, crank_reward, mint.decimals)?;
            } else {
                let cpi_ctx = CpiContext::new(cpi_p, cpi_accounts);
                transfer_checked(cpi_ctx, crank_reward, mint.decimals)?;
            }
        }
    }

    // 2. Transfer remainder to Treasury
    let cpi_accounts = TransferChecked {
        from: from.to_account_info(),
        to: to_treasury.to_account_info(),
        authority: authority.clone(),
        mint: mint.to_account_info(),
    };
    let cpi_p = token_program.to_account_info();

    if let Some(seeds) = signer_seeds {
        let cpi_ctx = CpiContext::new_with_signer(cpi_p, cpi_accounts, seeds);
        transfer_checked(cpi_ctx, treasury_amount, mint.decimals)?;
    } else {
        let cpi_ctx = CpiContext::new(cpi_p, cpi_accounts);
        transfer_checked(cpi_ctx, treasury_amount, mint.decimals)?;
    }

    Ok(())
}
