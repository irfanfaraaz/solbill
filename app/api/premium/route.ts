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
// In a real scenario, these would be the merchant's actual addresses
export const GET = withSolbill(handler, {
  serviceAuthority: "89Qz...merchant_wallet...", // Merchant authority or treasury
  planAddress: "Plan...subscription_plan...", // The specific plan ID
  priceUsdc: "0.01", // Pay-as-you-go price
});
