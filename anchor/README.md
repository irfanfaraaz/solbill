# SolBill: Smart Contract Suite

This directory contains the Solana program logic for SolBill, an autonomous recurring billing engine built with Anchor.

## 🏗 Program Structure

- **`programs/solbill/src/lib.rs`**: Main entry point and program ID declaration.
- **`programs/solbill/src/instructions/`**: Modular instruction handlers (Modularized to prevent stack overflows).
  - `initialize_service`: Setup merchant profile.
  - `create_plan`: Define billing rules (price, interval, max cycles).
  - `create_subscription`: Handle signup and upfront payments.
  - `collect_payment`: The "Crank" endpoint for recurring billing.
- **`programs/solbill/src/state.rs`**: Definitions for `ServiceAccount`, `PlanAccount`, and `SubscriptionAccount`.
- **`programs/solbill/src/tests.rs`**: Comprehensive `LiteSVM` tests.

## ⚙️ Development

### Prerequisites

- [Anchor CLI v0.31.0+](https://www.anchor-lang.com/docs/installation)
- [Solana CLI v2.1.0+](https://docs.solana.com/cli/install-solana-cli-tools)

### Build

```bash
anchor build
```

### Test (Local Simulation)

We use `LiteSVM` for blazing fast test execution without a local validator.

```bash
cargo test -- --nocapture
```

## 🔐 Security Constants

- **Access Guard**: Only the merchant can update their plans.
- **Account Sizing**: All accounts use `InitSpace` for precise allocation.
- **PDA Seeds**:
  - Service: `[b"service", merchant.key()]`
  - Plan: `[b"plan", service.key(), index.to_le_bytes()]`
  - Subscription: `[b"subscription", subscriber.key(), plan.key()]`

## 🚀 Deployment

1. Update `declare_id!` in `lib.rs`.
2. Update `[programs.localnet]` in `Anchor.toml`.
3. Run `anchor deploy`.
