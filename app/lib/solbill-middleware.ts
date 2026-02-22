import { withX402 } from "x402-next";
import {
  createSolanaRpc,
  address,
  getProgramDerivedAddress,
  getAddressEncoder,
} from "@solana/kit";
import {
  fetchMaybeSubscriptionAccount,
  SubscriptionStatus,
  SOLBILL_PROGRAM_ADDRESS,
} from "../generated/solbill";
import { NextResponse, type NextRequest } from "next/server";

import { SOLANA_RPC_URL } from "./solbill-config";

const RPC_ENDPOINT = SOLANA_RPC_URL;

/**
 * Unified Billing Middleware for SolBill
 */
export function withSolbill(
  handler: (req: NextRequest) => Promise<NextResponse>,
  config: {
    serviceAuthority: string;
    planAddress: string;
    priceUsdc: string;
  }
) {
  return async (req: NextRequest) => {
    const walletHeader = req.headers.get("x-wallet-address");
    const rpc = createSolanaRpc(RPC_ENDPOINT);

    // 1. Check for Subscription Bypass
    if (walletHeader) {
      try {
        const subscriber = address(walletHeader);
        const plan = address(config.planAddress);

        const [subPda] = await getProgramDerivedAddress({
          programAddress: address(SOLBILL_PROGRAM_ADDRESS),
          seeds: [
            new TextEncoder().encode("subscription"),
            getAddressEncoder().encode(subscriber),
            getAddressEncoder().encode(plan),
          ],
        });

        const maybeSubscription = await fetchMaybeSubscriptionAccount(
          rpc,
          subPda
        );

        if (
          maybeSubscription.exists &&
          (maybeSubscription.data.status === SubscriptionStatus.Active ||
            maybeSubscription.data.status === SubscriptionStatus.PastDue)
        ) {
          return handler(req);
        }
      } catch (e) {
        console.error("SolBill Middleware: Subscription check failed", e);
      }
    }

    const pathname = req.nextUrl.pathname;

    // 2. Fallback to x402 Pay-per-use
    try {
      const x402Handler = withX402(
        async (request: NextRequest) => handler(request),
        address(config.serviceAuthority),
        {
          price: `$${config.priceUsdc}`,
          network: "solana",
          config: {
            description: `SolBill Unified Gateway: Access to ${pathname}`,
          },
        },
        {
          url: "https://facilitator.cdp.coinbase.com",
        }
      );

      return await x402Handler(req);
    } catch {
      console.warn(
        "SolBill Middleware: x402 middleware failed (likely reachability). Returning manual 402 challenge."
      );

      // MANUAL FALLBACK 402 RESPONSE
      // This ensures the demo still shows the "Payment Required" flow even if DNS is flaky
      return new NextResponse(
        JSON.stringify({
          error: "Payment Required",
          message: "No active subscription found. Please pay via x402.",
          challenge: {
            amount: config.priceUsdc,
            currency: "USDC",
            network: "solana",
            payTo: config.serviceAuthority,
          },
        }),
        {
          status: 402,
          headers: {
            "Content-Type": "application/json",
            "X-X402-Required": "true",
            "X-X402-Pay-To": config.serviceAuthority,
            "X-X402-Amount": config.priceUsdc,
            "X-X402-Currency": "USDC",
            "X-X402-Network": "solana",
          },
        }
      );
    }
  };
}
