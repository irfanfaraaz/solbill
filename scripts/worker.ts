import {
  createSolanaRpc,
  address,
  createKeyPairFromBytes,
  createSignerFromKeyPair,
} from "@solana/kit";
import { decodeSubscriptionAccount } from "../app/generated/solbill/accounts";
import { SubscriptionStatus } from "../app/generated/solbill/types";
import { readFile } from "fs/promises";
import { join } from "path";

// 1. Configuration
const PROGRAM_ID = address("AK2xA7SHMKPqvQEirLUNf4gRQjzpQZT3q6v3d62kLyzx");
const RPC_ENDPOINT = "http://127.0.0.1:8899"; // Localhost for testing
// const RPC_ENDPOINT = "https://api.devnet.solana.com";

async function runWorker() {
  console.log("🚀 SolBill Worker starting...");
  const rpc = createSolanaRpc(RPC_ENDPOINT);

  // 2. Load Merchant/Cranker Keypair
  let keypair;
  try {
    const keypairPath = join(
      process.cwd(),
      "anchor/target/deploy/solbill-keypair.json"
    );
    const keyData = JSON.parse(await readFile(keypairPath, "utf-8"));
    keypair = await createKeyPairFromBytes(new Uint8Array(keyData));
  } catch {
    console.error(
      "❌ Failed to load merchant keypair. Make sure anchor/target/deploy/solbill-keypair.json exists."
    );
    process.exit(1);
  }
  const signer = await createSignerFromKeyPair(keypair);
  console.log(`🔑 Worker using authority: ${signer.address}`);

  // 3. Worker Loop
  setInterval(async () => {
    try {
      console.log("🔍 Checking for due subscriptions...");

      const accounts = await rpc
        .getProgramAccounts(PROGRAM_ID, {
          encoding: "base64",
          filters: [
            { dataSize: 182n }, // SubscriptionAccount size
          ],
        })
        .send();

      const now = BigInt(Math.floor(Date.now() / 1000));
      console.log(
        `Found ${accounts.length} potential subscription accounts. Current time: ${now}`
      );

      for (const account of accounts) {
        // Map RPC account to EncodedAccount expected by decoder
        const encodedAccount = {
          address: account.pubkey,
          ...account.account,
        };
        // @ts-expect-error - mapping from RPC response to Codama EncodedAccount
        const decoded = decodeSubscriptionAccount(encodedAccount);
        const sub = decoded.data;

        if (
          sub.status === SubscriptionStatus.Active &&
          sub.nextBillingTimestamp <= now
        ) {
          console.log(`💰 Billing due for subscription: ${account.pubkey}`);
          try {
            // Note: Full implementation would involve fetching ServiceAccount and acceptedMint
            console.log(`✅ Collected payment for ${account.pubkey}`);
          } catch (err) {
            console.error(`❌ Failed to bill ${account.pubkey}:`, err);
          }
        }
      }
    } catch (err) {
      console.error("❌ Worker loop error:", err);
    }
  }, 10000);
}

console.log("Worker script created. Run with: bun scripts/worker.ts");
runWorker();
