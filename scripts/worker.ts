import {
  createSolanaRpc,
  address,
  createKeyPairFromBytes,
  createSignerFromKeyPair,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  getSignatureFromTransaction,
  getBase64EncodedWireTransaction,
  getAddressEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import {
  decodeSubscriptionAccount,
  getSubscriptionAccountSize,
  fetchServiceAccount,
  getCollectPaymentInstruction,
  SubscriptionStatus,
} from "../app/generated/solbill";
import { readFile } from "fs/promises";
import { join } from "path";

// 1. Configuration
const PROGRAM_ID = address("AK2xA7SHMKPqvQEirLUNf4gRQjzpQZT3q6v3d62kLyzx");
const RPC_ENDPOINT = "http://127.0.0.1:8899"; // Change to Devnet if needed
const TOKEN_PROGRAM_ID = address("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ASSOCIATED_TOKEN_PROGRAM_ID = address(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

/**
 * Derives the Associated Token Account (ATA) address for a given wallet and mint.
 */
async function findAssociatedTokenAddress(
  walletAddr: string,
  mintAddr: string
) {
  const [ataAddress] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM_ID,
    seeds: [
      getAddressEncoder().encode(address(walletAddr)),
      getAddressEncoder().encode(TOKEN_PROGRAM_ID),
      getAddressEncoder().encode(address(mintAddr)),
    ],
  });
  return ataAddress;
}

async function runWorker() {
  console.log("🚀 SolBill Cranker Bot starting...");
  const rpc = createSolanaRpc(RPC_ENDPOINT);

  // 2. Load Cranker Keypair (This bot will sign as the cranker)
  let keypair;
  try {
    const keypairPath = join(
      process.cwd(),
      "anchor/target/deploy/solbill-keypair.json"
    );
    const keyData = JSON.parse(await readFile(keypairPath, "utf-8"));
    keypair = await createKeyPairFromBytes(new Uint8Array(keyData));
  } catch {
    console.warn(
      "⚠️  Failed to load anchor keypair. Using a random one for demonstration if needed, but collection will fail without SOL."
    );
    // In a real scenario, you'd load from a secure environment variable or local file.
    process.exit(1);
  }
  const signer = await createSignerFromKeyPair(keypair);
  console.log(`🔑 Cranker identity: ${signer.address}`);

  // 3. Bot Loop
  setInterval(async () => {
    try {
      console.log("🔍 Scanning for due payments...");

      // Get all Subscription Accounts
      const accounts = await rpc
        .getProgramAccounts(PROGRAM_ID, {
          encoding: "base64",
          filters: [{ dataSize: BigInt(getSubscriptionAccountSize()) }],
        })
        .send();

      const now = BigInt(Math.floor(Date.now() / 1000));
      let dueCount = 0;

      for (const account of accounts) {
        try {
          const encodedAccount = {
            address: account.pubkey,
            ...account.account,
          };

          // @ts-expect-error - Codama decodeAccount type mapping for RPC responses
          const decoded = decodeSubscriptionAccount(encodedAccount);
          const sub = decoded.data;

          // Check if subscription is ACTIVE and PAST DUE
          if (
            sub.status === SubscriptionStatus.Active &&
            sub.nextBillingTimestamp <= now
          ) {
            dueCount++;
            console.log(
              `💰 [DUE] ${account.pubkey} | Reward: ${sub.crankReward} tokens`
            );

            // Fetch Service details to find treasury and mint
            const serviceResult = await fetchServiceAccount(rpc, sub.service);
            const service = serviceResult.data;

            // Find or Derive Cranker ATA
            const crankerTokenAccount = await findAssociatedTokenAddress(
              signer.address,
              service.acceptedMint
            );

            // Construct instruction
            const collectIx = getCollectPaymentInstruction({
              cranker: signer,
              service: sub.service,
              subscription: account.pubkey,
              subscriberTokenAccount: sub.subscriberTokenAccount,
              treasury: service.treasury,
              crankerTokenAccount: crankerTokenAccount,
              acceptedMint: service.acceptedMint,
            });

            // Send Transaction
            const { value: latestBlockhash } = await rpc
              .getLatestBlockhash()
              .send();

            const transactionMessage = pipe(
              createTransactionMessage({ version: 0 }),
              (m) => setTransactionMessageFeePayerSigner(signer, m),
              (m) =>
                setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
              (m) => appendTransactionMessageInstruction(collectIx, m)
            );

            const signedTx =
              await signTransactionMessageWithSigners(transactionMessage);
            const signature = getSignatureFromTransaction(signedTx);
            const base64Tx = getBase64EncodedWireTransaction(signedTx);

            console.log(`⏳ Submitting transaction for ${account.pubkey}...`);
            await rpc.sendTransaction(base64Tx, { encoding: "base64" }).send();

            console.log(`✅ [COLLECTED] Signature: ${signature}`);
          }
        } catch (err) {
          console.error(`❌ Error processing account ${account.pubkey}:`, err);
        }
      }

      if (dueCount === 0) {
        console.log("😴 No payments due. Sleeping...");
      }
    } catch (err) {
      console.error("❌ Bot encountered an error:", err);
    }
  }, 15000); // Check every 15 seconds
}

console.log("\n--- SolBill Cranker Bot ---\n");
runWorker();
