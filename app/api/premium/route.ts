import { NextResponse } from "next/server";
import { withSolbill } from "../../lib/solbill-middleware";

/**
 * Demo Premium Endpoint
 *
 * This endpoint is protected by SolBill.
 * - If you have a subscription: access is free (200 OK).
 * - If you don't: you must pay 0.01 USDC (402 Payment Required).
 */
async function handler() {
  return NextResponse.json({
    message: "Welcome to the Premium SolBill Service!",
    content:
      "This content is monetized with Solana x402 + SolBill Subscriptions.",
    timestamp: new Date().toISOString(),
    tip: "You accessed this because you are either a monthly subscriber or just paid 0.01 USDC per-request!",
  });
}

// Wrap the handler with SolBill logic
// Set env vars for production; fallback to demo placeholders for local dev
export const GET = withSolbill(handler, {
  serviceAuthority:
    process.env.NEXT_PUBLIC_SOLBILL_SERVICE_AUTHORITY ?? "89Qz...merchant_wallet...",
  planAddress:
    process.env.NEXT_PUBLIC_SOLBILL_PLAN_ADDRESS ?? "Plan...subscription_plan...",
  priceUsdc: process.env.NEXT_PUBLIC_SOLBILL_PRICE_USDC ?? "0.01",
});
