#[cfg(test)]
mod tests {
    use crate::ID as PROGRAM_ID;
    use litesvm::LiteSVM;
    use solana_sdk::program_pack::Pack;
    use solana_sdk::{
        clock::Clock,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::Transaction,
    };
    #[allow(deprecated)]
    use solana_sdk::system_program;
    use spl_token::state::{Account as TokenAccount, Mint};

    const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

    // Helper to generate discriminators
    fn get_discriminator(name: &str) -> [u8; 8] {
        let mut discriminator = [0u8; 8];
        let hash = solana_sdk::hash::hash(format!("global:{}", name).as_bytes());
        discriminator.copy_from_slice(&hash.to_bytes()[..8]);
        discriminator
    }

    fn get_service_pda(authority: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"service", authority.as_ref()], &PROGRAM_ID)
    }

    fn get_plan_pda(service: &Pubkey, index: u16) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"plan", service.as_ref(), index.to_le_bytes().as_ref()],
            &PROGRAM_ID,
        )
    }

    fn get_subscription_pda(subscriber: &Pubkey, plan: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"subscription", subscriber.as_ref(), plan.as_ref()],
            &PROGRAM_ID,
        )
    }

    #[test]
    fn test_initialization_and_plan_creation() {
        let mut svm = LiteSVM::new();

        // Load the program
        let program_bytes = include_bytes!("../../../target/deploy/solbill.so");
        let _ = svm.add_program(PROGRAM_ID, program_bytes);

        // Identities
        let merchant = Keypair::new();
        let mint = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();

        svm.airdrop(&merchant.pubkey(), LAMPORTS_PER_SOL).unwrap();

        // Initialize Mint
        let mut mint_data = vec![0u8; Mint::LEN];
        let mint_state = Mint {
            mint_authority: solana_sdk::program_option::COption::Some(merchant.pubkey()),
            supply: 0,
            decimals: 6,
            is_initialized: true,
            freeze_authority: solana_sdk::program_option::COption::None,
        };
        Mint::pack(mint_state, &mut mint_data).unwrap();

        let mint_account = solana_sdk::account::Account {
            lamports: 1_000_000_000,
            data: mint_data,
            owner: anchor_spl::token::ID,
            executable: false,
            rent_epoch: 0,
        };
        svm.set_account(mint, mint_account).unwrap();

        // Initialize Treasury (as a token account)
        let mut treasury_data = vec![0u8; TokenAccount::LEN];
        let treasury_state = TokenAccount {
            mint,
            owner: merchant.pubkey(),
            amount: 0,
            delegate: solana_sdk::program_option::COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: solana_sdk::program_option::COption::None,
            delegated_amount: 0,
            close_authority: solana_sdk::program_option::COption::None,
        };
        TokenAccount::pack(treasury_state, &mut treasury_data).unwrap();

        let treasury_account = solana_sdk::account::Account {
            lamports: 1_000_000_000,
            data: treasury_data,
            owner: anchor_spl::token::ID,
            executable: false,
            rent_epoch: 0,
        };
        svm.set_account(treasury, treasury_account).unwrap();

        // 1. Initialize Service
        let (service_pda, _bump) = get_service_pda(&merchant.pubkey());
        let init_ix_data = get_discriminator("initialize_service");

        let init_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(treasury, false),
                AccountMeta::new_readonly(anchor_spl::token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: init_ix_data.to_vec(),
        };

        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            blockhash,
        );

        svm.send_transaction(tx).expect("Initialize service failed");

        // 2. Create Plan
        let (plan_pda, _plan_bump) = get_plan_pda(&service_pda, 0);
        let plan_name = "Pro Plan";
        let amount: u64 = 10_000_000; // 10 USDC
        let crank_reward: u64 = 100_000; // 0.1 USDC bounty
        let interval: i64 = 3600; // 1 hr
        let grace_period: i64 = 86400; // 1 day

        let mut plan_ix_data = get_discriminator("create_plan").to_vec();
        // Manual serialization: name: String, amount: u64, crank_reward: u64, interval: i64, grace_period: i64
        let name_bytes = plan_name.as_bytes();
        plan_ix_data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        plan_ix_data.extend_from_slice(name_bytes);
        plan_ix_data.extend_from_slice(&amount.to_le_bytes());
        plan_ix_data.extend_from_slice(&crank_reward.to_le_bytes());
        plan_ix_data.extend_from_slice(&interval.to_le_bytes());
        plan_ix_data.extend_from_slice(&grace_period.to_le_bytes());
        plan_ix_data.extend_from_slice(&0u64.to_le_bytes()); // max_billing_cycles = 0 (infinite)

        let plan_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new(plan_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: plan_ix_data,
        };

        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            &[plan_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            blockhash,
        );

        svm.send_transaction(tx).expect("Create plan failed");

        // Verify PlanAccount state
        let plan_account_data = svm.get_account(&plan_pda).unwrap().data;
        assert_eq!(&plan_account_data[8..40], service_pda.as_ref());

        let amount_in_state = u64::from_le_bytes(plan_account_data[72..80].try_into().unwrap());
        assert_eq!(amount_in_state, amount);

        let reward_in_state = u64::from_le_bytes(plan_account_data[80..88].try_into().unwrap());
        assert_eq!(reward_in_state, crank_reward);
    }

    #[test]
    fn test_subscription_and_billing_cycle() {
        let mut svm = LiteSVM::new();

        // Load the program
        let program_bytes = include_bytes!("../../../target/deploy/solbill.so");
        let _ = svm.add_program(PROGRAM_ID, program_bytes);

        // Identities
        let merchant = Keypair::new();
        let subscriber = Keypair::new();
        let cranker = Keypair::new();
        let mint = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let subscriber_token = Pubkey::new_unique();
        let cranker_token = Pubkey::new_unique();

        svm.airdrop(&merchant.pubkey(), LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&subscriber.pubkey(), LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&cranker.pubkey(), LAMPORTS_PER_SOL).unwrap();

        // 0. Setup Token Accounts
        let mut mint_data = vec![0u8; Mint::LEN];
        Mint::pack(
            Mint {
                mint_authority: solana_sdk::program_option::COption::Some(merchant.pubkey()),
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: solana_sdk::program_option::COption::None,
            },
            &mut mint_data,
        )
        .unwrap();
        svm.set_account(
            mint,
            solana_sdk::account::Account {
                lamports: 100_000_000,
                data: mint_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        // Treasury
        let mut treasury_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint,
                owner: merchant.pubkey(),
                amount: 0,
                state: spl_token::state::AccountState::Initialized,
                ..TokenAccount::default()
            },
            &mut treasury_data,
        )
        .unwrap();
        svm.set_account(
            treasury,
            solana_sdk::account::Account {
                lamports: 100_000_000,
                data: treasury_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        // Subscriber Token
        let initial_balance = 50_000_000;
        let mut sub_token_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint,
                owner: subscriber.pubkey(),
                amount: initial_balance,
                state: spl_token::state::AccountState::Initialized,
                ..TokenAccount::default()
            },
            &mut sub_token_data,
        )
        .unwrap();
        svm.set_account(
            subscriber_token,
            solana_sdk::account::Account {
                lamports: 100_000_000,
                data: sub_token_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        // Cranker Token
        let mut cranker_token_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint,
                owner: cranker.pubkey(),
                amount: 0,
                state: spl_token::state::AccountState::Initialized,
                ..TokenAccount::default()
            },
            &mut cranker_token_data,
        )
        .unwrap();
        svm.set_account(
            cranker_token,
            solana_sdk::account::Account {
                lamports: 100_000_000,
                data: cranker_token_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        // 1. Initialize Service
        let (service_pda, _) = get_service_pda(&merchant.pubkey());
        let init_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(treasury, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: get_discriminator("initialize_service").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // 2. Create Plan
        let (plan_pda, _) = get_plan_pda(&service_pda, 0);
        let amount: u64 = 10_000_000;
        let crank_reward: u64 = 500_000; // 0.5 USDC bounty
        let interval: i64 = 3600;
        let mut plan_data = get_discriminator("create_plan").to_vec();
        plan_data.extend_from_slice(&8u32.to_le_bytes()); // name length
        plan_data.extend_from_slice(b"Pro Plan");
        plan_data.extend_from_slice(&amount.to_le_bytes());
        plan_data.extend_from_slice(&crank_reward.to_le_bytes());
        plan_data.extend_from_slice(&interval.to_le_bytes());
        plan_data.extend_from_slice(&3600i64.to_le_bytes()); // grace period
        plan_data.extend_from_slice(&0u64.to_le_bytes()); // max_billing_cycles = 0 (infinite)

        let plan_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new(plan_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: plan_data,
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[plan_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // 3. Create Subscription (and pay upfront)
        let (sub_pda, _) = get_subscription_pda(&subscriber.pubkey(), &plan_pda);
        let sub_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(subscriber.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new_readonly(plan_pda, false),
                AccountMeta::new(sub_pda, false),
                AccountMeta::new(subscriber_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(treasury, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: get_discriminator("create_subscription").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[sub_ix],
            Some(&subscriber.pubkey()),
            &[&subscriber],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // CHECK: Verify upfront payment
        let sub_token_acc =
            TokenAccount::unpack(&svm.get_account(&subscriber_token).unwrap().data).unwrap();
        assert_eq!(sub_token_acc.amount, initial_balance - amount); // Paid 1st month

        let treasury_acc = TokenAccount::unpack(&svm.get_account(&treasury).unwrap().data).unwrap();
        assert_eq!(treasury_acc.amount, amount); // Received 1st month (no crank reward)

        // 4. Collect Payment (Incentivized Crank - Month 2)
        let mut clock = svm.get_sysvar::<Clock>();
        clock.unix_timestamp += interval + 1; // Fast-forward 1 month
        svm.set_sysvar::<Clock>(&clock);

        let collect_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(cranker.pubkey(), true),
                AccountMeta::new_readonly(service_pda, false),
                AccountMeta::new(sub_pda, false),
                AccountMeta::new(subscriber_token, false),
                AccountMeta::new(treasury, false),
                AccountMeta::new(cranker_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(spl_token::ID, false),
            ],
            data: get_discriminator("collect_payment").to_vec(),
        };

        svm.send_transaction(Transaction::new_signed_with_payer(
            &[collect_ix],
            Some(&cranker.pubkey()),
            &[&cranker],
            svm.latest_blockhash(),
        ))
        .expect("Crank collection failed");

        // 5. Verify Balances after Month 2 Collection
        let sub_token_acc =
            TokenAccount::unpack(&svm.get_account(&subscriber_token).unwrap().data).unwrap();
        assert_eq!(sub_token_acc.amount, initial_balance - (amount * 2)); // Paid 2 months total

        let treasury_acc = TokenAccount::unpack(&svm.get_account(&treasury).unwrap().data).unwrap();
        assert_eq!(treasury_acc.amount, amount + (amount - crank_reward)); // 1st full + 2nd partial

        let cranker_token_acc =
            TokenAccount::unpack(&svm.get_account(&cranker_token).unwrap().data).unwrap();
        assert_eq!(cranker_token_acc.amount, crank_reward); // Earned reward for 2nd month

        println!(
            "🚀 Cranker received bounty: {} tokens",
            cranker_token_acc.amount
        );
    }

    #[test]
    fn test_one_time_payment_plan() {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../../target/deploy/solbill.so");
        let _ = svm.add_program(PROGRAM_ID, program_bytes);

        let merchant = Keypair::new();
        let subscriber = Keypair::new();
        let mint = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let subscriber_token = Pubkey::new_unique();

        svm.airdrop(&merchant.pubkey(), LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&subscriber.pubkey(), LAMPORTS_PER_SOL).unwrap();

        // Mint Setup
        let mut mint_data = vec![0u8; Mint::LEN];
        Mint::pack(
            Mint {
                mint_authority: solana_sdk::program_option::COption::Some(merchant.pubkey()),
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: solana_sdk::program_option::COption::None,
            },
            &mut mint_data,
        )
        .unwrap();
        svm.set_account(
            mint,
            solana_sdk::account::Account {
                lamports: 100000000,
                data: mint_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        // Account Setup (Treasury & Subscriber)
        let mut treasury_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint,
                owner: merchant.pubkey(),
                amount: 0,
                state: spl_token::state::AccountState::Initialized,
                ..TokenAccount::default()
            },
            &mut treasury_data,
        )
        .unwrap();
        svm.set_account(
            treasury,
            solana_sdk::account::Account {
                lamports: 100000000,
                data: treasury_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        let mut sub_token_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint,
                owner: subscriber.pubkey(),
                amount: 10_000_000,
                state: spl_token::state::AccountState::Initialized,
                ..Default::default()
            },
            &mut sub_token_data,
        )
        .unwrap();
        svm.set_account(
            subscriber_token,
            solana_sdk::account::Account {
                lamports: 100000000,
                data: sub_token_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        // 1. Initialize Service
        let (service_pda, _) = get_service_pda(&merchant.pubkey());
        let init_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(treasury, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: get_discriminator("initialize_service").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // 2. Create One-Time Plan (max_cycles = 1)
        let (plan_pda, _) = get_plan_pda(&service_pda, 0);
        let amount: u64 = 5_000_000;
        let mut plan_data = get_discriminator("create_plan").to_vec();
        plan_data.extend_from_slice(&8u32.to_le_bytes()); // name len
        plan_data.extend_from_slice(b"One Time");
        plan_data.extend_from_slice(&amount.to_le_bytes());
        plan_data.extend_from_slice(&0u64.to_le_bytes()); // reward
        plan_data.extend_from_slice(&3600i64.to_le_bytes());
        plan_data.extend_from_slice(&3600i64.to_le_bytes());
        plan_data.extend_from_slice(&1u64.to_le_bytes()); // max_billing_cycles = 1 (One-time)

        let plan_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new(plan_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: plan_data,
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[plan_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // 3. Create Subscription
        let (sub_pda, _) = get_subscription_pda(&subscriber.pubkey(), &plan_pda);
        let sub_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(subscriber.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new_readonly(plan_pda, false),
                AccountMeta::new(sub_pda, false),
                AccountMeta::new(subscriber_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(treasury, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: get_discriminator("create_subscription").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[sub_ix],
            Some(&subscriber.pubkey()),
            &[&subscriber],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // 4. Verify Completed Status
        let sub_account_data = svm.get_account(&sub_pda).unwrap().data;
        // Layout: Disc(8) + subscriber(32) + service(32) + original_plan(32) + plan(32) + token_acct(32) + amount(8) + reward(8) + interval(8) + next(8) + last(8) + created(8) + status(1) + payments(4)
        // Status at 8+32*5+8*6+1 = 216
        let status_byte = sub_account_data[216];
        // 0=Active, 1=PastDue, 2=Cancelled, 3=Expired, 4=Completed
        assert_eq!(
            status_byte, 4,
            "Subscription should be Completed (enum variant 4)"
        );

        let payments_made = u32::from_le_bytes(sub_account_data[217..221].try_into().unwrap());
        assert_eq!(payments_made, 1, "Should have made 1 payment");
    }

    #[test]
    fn test_fixed_term_plan() {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../../target/deploy/solbill.so");
        let _ = svm.add_program(PROGRAM_ID, program_bytes);
        let merchant = Keypair::new();
        let subscriber = Keypair::new();
        let cranker = Keypair::new();
        let mint = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let subscriber_token = Pubkey::new_unique();
        let cranker_token = Pubkey::new_unique();

        svm.airdrop(&merchant.pubkey(), LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&subscriber.pubkey(), LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&cranker.pubkey(), LAMPORTS_PER_SOL).unwrap();

        // Mint & Accounts
        let mut mint_data = vec![0u8; Mint::LEN];
        Mint::pack(
            Mint {
                mint_authority: solana_sdk::program_option::COption::Some(merchant.pubkey()),
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: solana_sdk::program_option::COption::None,
            },
            &mut mint_data,
        )
        .unwrap();
        svm.set_account(
            mint,
            solana_sdk::account::Account {
                lamports: 100000000,
                data: mint_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        let mut treasury_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint,
                owner: merchant.pubkey(),
                state: spl_token::state::AccountState::Initialized,
                ..TokenAccount::default()
            },
            &mut treasury_data,
        )
        .unwrap();
        svm.set_account(
            treasury,
            solana_sdk::account::Account {
                lamports: 100000000,
                data: treasury_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        let mut cranker_token_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint,
                owner: cranker.pubkey(),
                state: spl_token::state::AccountState::Initialized,
                ..TokenAccount::default()
            },
            &mut cranker_token_data,
        )
        .unwrap();
        svm.set_account(
            cranker_token,
            solana_sdk::account::Account {
                lamports: 100000000,
                data: cranker_token_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        let mut sub_token_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint,
                owner: subscriber.pubkey(),
                amount: 20_000_000,
                state: spl_token::state::AccountState::Initialized,
                ..Default::default()
            },
            &mut sub_token_data,
        )
        .unwrap();
        svm.set_account(
            subscriber_token,
            solana_sdk::account::Account {
                lamports: 100000000,
                data: sub_token_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        // Init Service
        let (service_pda, _) = get_service_pda(&merchant.pubkey());
        let init_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(treasury, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: get_discriminator("initialize_service").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // Create 2-Cycle Plan
        let (plan_pda, _) = get_plan_pda(&service_pda, 0);
        let amount: u64 = 5_000_000;
        let mut plan_data = get_discriminator("create_plan").to_vec();
        plan_data.extend_from_slice(&7u32.to_le_bytes());
        plan_data.extend_from_slice(b"2Cycles");
        plan_data.extend_from_slice(&amount.to_le_bytes());
        plan_data.extend_from_slice(&100_000u64.to_le_bytes());
        plan_data.extend_from_slice(&3600i64.to_le_bytes());
        plan_data.extend_from_slice(&3600i64.to_le_bytes());
        plan_data.extend_from_slice(&2u64.to_le_bytes()); // max_billing_cycles = 2

        let plan_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new(plan_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: plan_data,
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[plan_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // Subscribe (Payment 1/2)
        let (sub_pda, _) = get_subscription_pda(&subscriber.pubkey(), &plan_pda);
        let sub_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(subscriber.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new_readonly(plan_pda, false),
                AccountMeta::new(sub_pda, false),
                AccountMeta::new(subscriber_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(treasury, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: get_discriminator("create_subscription").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[sub_ix],
            Some(&subscriber.pubkey()),
            &[&subscriber],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // Verify Active (1/2 paid)
        let sub_data = svm.get_account(&sub_pda).unwrap().data;
        assert_eq!(sub_data[216], 0); // Active (Status Offset 216)
        let payments = u32::from_le_bytes(sub_data[217..221].try_into().unwrap());
        assert_eq!(payments, 1);

        // Advance Clock
        let mut clock = svm.get_sysvar::<Clock>();
        clock.unix_timestamp += 3601;
        svm.set_sysvar::<Clock>(&clock);

        // Collect (Payment 2/2)
        let collect_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(cranker.pubkey(), true),
                AccountMeta::new_readonly(service_pda, false),
                AccountMeta::new(sub_pda, false),
                AccountMeta::new(subscriber_token, false),
                AccountMeta::new(treasury, false),
                AccountMeta::new(cranker_token, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(spl_token::ID, false),
            ],
            data: get_discriminator("collect_payment").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[collect_ix],
            Some(&cranker.pubkey()),
            &[&cranker],
            svm.latest_blockhash(),
        ))
        .unwrap();

        // Verify Completed (2/2 paid)
        let sub_data = svm.get_account(&sub_pda).unwrap().data;
        assert_eq!(sub_data[216], 4); // Completed
        let payments = u32::from_le_bytes(sub_data[217..221].try_into().unwrap());
        assert_eq!(payments, 2);
    }

    #[test]
    fn test_cancel_subscription() {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../../target/deploy/solbill.so");
        let _ = svm.add_program(PROGRAM_ID, program_bytes);

        let merchant = Keypair::new();
        let subscriber = Keypair::new();
        let mint = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let subscriber_token = Pubkey::new_unique();

        svm.airdrop(&merchant.pubkey(), LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&subscriber.pubkey(), LAMPORTS_PER_SOL).unwrap();

        setup_mint_and_accounts(
            &mut svm,
            &merchant,
            &subscriber,
            &mint,
            &treasury,
            &subscriber_token,
            20_000_000,
        );

        let (service_pda, _) = get_service_pda(&merchant.pubkey());
        let (plan_pda, _) = get_plan_pda(&service_pda, 0);
        let (sub_pda, _) = get_subscription_pda(&subscriber.pubkey(), &plan_pda);

        init_service_and_plan(&mut svm, &merchant, &service_pda, &plan_pda, &mint, &treasury);
        create_subscription_ix(&mut svm, &subscriber, &service_pda, &plan_pda, &sub_pda, &subscriber_token, &mint, &treasury);

        assert!(svm.get_account(&sub_pda).is_some(), "Subscription should exist before cancel");

        let cancel_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(subscriber.pubkey(), true),
                AccountMeta::new(service_pda, false),
                AccountMeta::new(sub_pda, false),
                AccountMeta::new(subscriber_token, false),
                AccountMeta::new_readonly(spl_token::ID, false),
            ],
            data: get_discriminator("cancel_subscription").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[cancel_ix],
            Some(&subscriber.pubkey()),
            &[&subscriber],
            svm.latest_blockhash(),
        ))
        .expect("Cancel subscription failed");

        assert!(svm.get_account(&sub_pda).is_none(), "Subscription account should be closed");
    }

    #[test]
    fn test_update_plan() {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../../target/deploy/solbill.so");
        let _ = svm.add_program(PROGRAM_ID, program_bytes);

        let merchant = Keypair::new();
        let mint = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();

        svm.airdrop(&merchant.pubkey(), LAMPORTS_PER_SOL).unwrap();
        setup_mint_and_treasury(&mut svm, &merchant, &mint, &treasury);

        let (service_pda, _) = get_service_pda(&merchant.pubkey());
        let (plan_pda, _) = get_plan_pda(&service_pda, 0);

        init_service_and_plan(&mut svm, &merchant, &service_pda, &plan_pda, &mint, &treasury);

        let new_amount: u64 = 15_000_000;
        let mut update_data = get_discriminator("update_plan").to_vec();
        update_data.push(1); // Some
        update_data.extend_from_slice(&new_amount.to_le_bytes());
        update_data.push(0); // None cranker_reward
        update_data.push(0); // None interval
        update_data.push(0); // None is_active
        update_data.push(0); // None grace_period

        let update_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new_readonly(service_pda, false),
                AccountMeta::new(plan_pda, false),
            ],
            data: update_data,
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[update_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .expect("Update plan failed");

        let plan_data = svm.get_account(&plan_pda).unwrap().data;
        let amount_offset = 8 + 32 + 32; // discriminator + service + name[32]
        let amount = u64::from_le_bytes(plan_data[amount_offset..amount_offset + 8].try_into().unwrap());
        assert_eq!(amount, new_amount, "Plan amount should be updated");
    }

    #[test]
    fn test_expire_subscription() {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../../target/deploy/solbill.so");
        let _ = svm.add_program(PROGRAM_ID, program_bytes);

        let merchant = Keypair::new();
        let subscriber = Keypair::new();
        let cranker = Keypair::new();
        let mint = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let subscriber_token = Pubkey::new_unique();

        svm.airdrop(&merchant.pubkey(), LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&subscriber.pubkey(), LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&cranker.pubkey(), LAMPORTS_PER_SOL).unwrap();

        setup_mint_and_accounts(
            &mut svm,
            &merchant,
            &subscriber,
            &mint,
            &treasury,
            &subscriber_token,
            20_000_000,
        );

        let (service_pda, _) = get_service_pda(&merchant.pubkey());
        let (plan_pda, _) = get_plan_pda(&service_pda, 0);
        let (sub_pda, _) = get_subscription_pda(&subscriber.pubkey(), &plan_pda);

        init_service_and_plan_with_grace(&mut svm, &merchant, &service_pda, &plan_pda, &mint, &treasury, 3600);
        create_subscription_ix(&mut svm, &subscriber, &service_pda, &plan_pda, &sub_pda, &subscriber_token, &mint, &treasury);

        let mut sub_account = svm.get_account(&sub_pda).unwrap();
        sub_account.data[216] = 1; // Set status to PastDue
        svm.set_account(sub_pda, sub_account).unwrap();

        let mut clock = svm.get_sysvar::<Clock>();
        clock.unix_timestamp += 7200; // 2 hours past next_billing + grace
        svm.set_sysvar::<Clock>(&clock);

        let expire_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(cranker.pubkey(), true),
                AccountMeta::new_readonly(plan_pda, false),
                AccountMeta::new(sub_pda, false),
            ],
            data: get_discriminator("expire_subscription").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[expire_ix],
            Some(&cranker.pubkey()),
            &[&cranker],
            svm.latest_blockhash(),
        ))
        .expect("Expire subscription failed");

        assert!(svm.get_account(&sub_pda).is_none(), "Subscription should be closed");
    }

    fn setup_mint_and_treasury(
        svm: &mut LiteSVM,
        merchant: &Keypair,
        mint: &Pubkey,
        treasury: &Pubkey,
    ) {
        let mut mint_data = vec![0u8; Mint::LEN];
        Mint::pack(
            Mint {
                mint_authority: solana_sdk::program_option::COption::Some(merchant.pubkey()),
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: solana_sdk::program_option::COption::None,
            },
            &mut mint_data,
        )
        .unwrap();
        svm.set_account(
            *mint,
            solana_sdk::account::Account {
                lamports: 100_000_000,
                data: mint_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();

        let mut treasury_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint: *mint,
                owner: merchant.pubkey(),
                state: spl_token::state::AccountState::Initialized,
                ..TokenAccount::default()
            },
            &mut treasury_data,
        )
        .unwrap();
        svm.set_account(
            *treasury,
            solana_sdk::account::Account {
                lamports: 100_000_000,
                data: treasury_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();
    }

    fn setup_mint_and_accounts(
        svm: &mut LiteSVM,
        merchant: &Keypair,
        subscriber: &Keypair,
        mint: &Pubkey,
        treasury: &Pubkey,
        subscriber_token: &Pubkey,
        sub_balance: u64,
    ) {
        setup_mint_and_treasury(svm, merchant, mint, treasury);
        let mut sub_token_data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint: *mint,
                owner: subscriber.pubkey(),
                amount: sub_balance,
                state: spl_token::state::AccountState::Initialized,
                ..TokenAccount::default()
            },
            &mut sub_token_data,
        )
        .unwrap();
        svm.set_account(
            *subscriber_token,
            solana_sdk::account::Account {
                lamports: 100_000_000,
                data: sub_token_data,
                owner: spl_token::ID,
                ..Default::default()
            },
        )
        .unwrap();
    }

    fn init_service_and_plan(
        svm: &mut LiteSVM,
        merchant: &Keypair,
        service_pda: &Pubkey,
        plan_pda: &Pubkey,
        mint: &Pubkey,
        treasury: &Pubkey,
    ) {
        init_service_and_plan_with_grace(svm, merchant, service_pda, plan_pda, mint, treasury, 3600);
    }

    fn init_service_and_plan_with_grace(
        svm: &mut LiteSVM,
        merchant: &Keypair,
        service_pda: &Pubkey,
        plan_pda: &Pubkey,
        mint: &Pubkey,
        treasury: &Pubkey,
        grace_period: i64,
    ) {
        let init_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(*service_pda, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(*treasury, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: get_discriminator("initialize_service").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .unwrap();

        let mut plan_data = get_discriminator("create_plan").to_vec();
        plan_data.extend_from_slice(&8u32.to_le_bytes());
        plan_data.extend_from_slice(b"Pro Plan");
        plan_data.extend_from_slice(&10_000_000u64.to_le_bytes());
        plan_data.extend_from_slice(&100_000u64.to_le_bytes());
        plan_data.extend_from_slice(&3600i64.to_le_bytes());
        plan_data.extend_from_slice(&grace_period.to_le_bytes());
        plan_data.extend_from_slice(&0u64.to_le_bytes());

        let plan_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(merchant.pubkey(), true),
                AccountMeta::new(*service_pda, false),
                AccountMeta::new(*plan_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: plan_data,
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[plan_ix],
            Some(&merchant.pubkey()),
            &[&merchant],
            svm.latest_blockhash(),
        ))
        .unwrap();
    }

    fn create_subscription_ix(
        svm: &mut LiteSVM,
        subscriber: &Keypair,
        service_pda: &Pubkey,
        plan_pda: &Pubkey,
        sub_pda: &Pubkey,
        subscriber_token: &Pubkey,
        mint: &Pubkey,
        treasury: &Pubkey,
    ) {
        let sub_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(subscriber.pubkey(), true),
                AccountMeta::new(*service_pda, false),
                AccountMeta::new_readonly(*plan_pda, false),
                AccountMeta::new(*sub_pda, false),
                AccountMeta::new(*subscriber_token, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new(*treasury, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: get_discriminator("create_subscription").to_vec(),
        };
        svm.send_transaction(Transaction::new_signed_with_payer(
            &[sub_ix],
            Some(&subscriber.pubkey()),
            &[&subscriber],
            svm.latest_blockhash(),
        ))
        .unwrap();
    }

    #[test]
    fn test_id() {
        assert_eq!(
            PROGRAM_ID,
            address_to_pubkey("AK2xA7SHMKPqvQEirLUNf4gRQjzpQZT3q6v3d62kLyzx")
        );
    }

    fn address_to_pubkey(addr: &str) -> Pubkey {
        addr.parse().unwrap()
    }
}
