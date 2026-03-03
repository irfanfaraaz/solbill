/**
 * Run key SolBill flows on Devnet and output transaction links for README.
 * Uses default Solana CLI wallet (~/.config/solana/id.json).
 *
 * Run: npx ts-node scripts/run-devnet-txs.ts
 */
import { readFile } from "fs/promises";
import { join } from "path";
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
  getBytesEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import {
  getInitializeServiceInstructionAsync,
  getCreatePlanInstructionAsync,
  getCreateSubscriptionInstructionAsync,
  getCollectPaymentInstruction,
  fetchMaybeServiceAccount,
  fetchMaybeSubscriptionAccount,
} from "../app/generated/solbill";

const RPC = process.env.RPC_ENDPOINT || "https://api.devnet.solana.com";
const PROGRAM_ID = address("AK2xA7SHMKPqvQEirLUNf4gRQjzpQZT3q6v3d62kLyzx");
const USDC_MINT = address("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
const ASSOCIATED_TOKEN_PROGRAM = address(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);
const TOKEN_PROGRAM = address(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

async function getAta(mint: string, owner: string) {
  const [ata] = await getProgramDerivedAddress({
    programAddress: ASSOCIATED_TOKEN_PROGRAM,
    seeds: [
      getAddressEncoder().encode(address(owner)),
      getAddressEncoder().encode(TOKEN_PROGRAM),
      getAddressEncoder().encode(address(mint)),
    ],
  });
  return ata;
}

async function getServicePda(authority: string) {
  const [pda] = await getProgramDerivedAddress({
    programAddress: PROGRAM_ID,
    seeds: [
      getBytesEncoder().encode(new Uint8Array([115, 101, 114, 118, 105, 99, 101])),
      getAddressEncoder().encode(address(authority)),
    ],
  });
  return pda;
}

async function getPlanPda(service: string, index: number) {
  const [pda] = await getProgramDerivedAddress({
    programAddress: PROGRAM_ID,
    seeds: [
      getBytesEncoder().encode(new Uint8Array([112, 108, 97, 110])),
      getAddressEncoder().encode(address(service)),
      new Uint8Array([index & 0xff, (index >> 8) & 0xff]),
    ],
  });
  return pda;
}

async function getSubscriptionPda(subscriber: string, plan: string) {
  const [pda] = await getProgramDerivedAddress({
    programAddress: PROGRAM_ID,
    seeds: [
      getBytesEncoder().encode(
        new Uint8Array([115, 117, 98, 115, 99, 114, 105, 112, 116, 105, 111, 110])
      ),
      getAddressEncoder().encode(address(subscriber)),
      getAddressEncoder().encode(address(plan)),
    ],
  });
  return pda;
}

type InstructionLike = { accounts: unknown[]; data: Uint8Array; programAddress: string };

async function sendTx(ix: InstructionLike, signer: Awaited<ReturnType<typeof createSignerFromKeyPair>>) {
  const rpc = createSolanaRpc(RPC);
  const { value: blockhash } = await rpc.getLatestBlockhash().send();
  const msg = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(signer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (m) => appendTransactionMessageInstruction(ix as any, m)
  );
  const signed = await signTransactionMessageWithSigners(msg);
  const sig = getSignatureFromTransaction(signed);
  await rpc
    .sendTransaction(getBase64EncodedWireTransaction(signed), {
      encoding: "base64",
    })
    .send();
  return sig;
}

async function main() {
  const keypairPath = process.env.SOLANA_KEYPAIR || join(process.env.HOME!, ".config/solana/id.json");
  const keyData = JSON.parse(await readFile(keypairPath, "utf-8"));
  const keypair = await createKeyPairFromBytes(new Uint8Array(keyData));
  const signer = await createSignerFromKeyPair(keypair);
  const walletAddr = String(signer.address);

  const rpc = createSolanaRpc(RPC);
  const servicePda = await getServicePda(walletAddr);
  const maybeService = await fetchMaybeServiceAccount(rpc, servicePda);

  let initSig: string | null = null;
  let createPlanSig: string | null = null;
  let subscribeSig: string | null = null;
  let collectSig: string | null = null;

  if (!maybeService.exists) {
    console.log("1. Initializing service...");
    const treasury = await getAta(USDC_MINT, walletAddr);
    const ix = await getInitializeServiceInstructionAsync({
      authority: signer,
      acceptedMint: USDC_MINT,
      treasury,
    });
    initSig = await sendTx(ix, signer);
    console.log("   init_service:", initSig);
    await new Promise((r) => setTimeout(r, 3000));
  } else {
    console.log("1. Service already exists, skipping init");
  }

  const maybeSvc = await fetchMaybeServiceAccount(rpc, servicePda);
  if (!maybeSvc.exists) throw new Error("Service not found after init");
  const service = maybeSvc.data;

  const planIndex = Number(service.planCount);
  const planAddr = await getPlanPda(servicePda, planIndex);
  const planExists = (await rpc.getAccountInfo(planAddr).send()).value !== null;

  if (!planExists) {
    console.log("2. Creating plan...");
    const ix = await getCreatePlanInstructionAsync({
      authority: signer,
      plan: planAddr,
      name: "Pro Monthly",
      amount: BigInt(1_000_000),
      crankReward: BigInt(50_000),
      interval: BigInt(10),
      gracePeriod: BigInt(60),
      maxBillingCycles: BigInt(0),
    });
    createPlanSig = await sendTx(ix, signer);
    console.log("   create_plan:", createPlanSig);
    await new Promise((r) => setTimeout(r, 2000));
  } else {
    console.log("2. Plan already exists, skipping");
  }

  const planToSubscribe = planExists ? await getPlanPda(servicePda, planIndex - 1) : planAddr;
  const subPda = await getSubscriptionPda(walletAddr, planToSubscribe);
  const maybeSub = await fetchMaybeSubscriptionAccount(rpc, subPda);

  if (!maybeSub.exists) {
    console.log("3. Creating subscription (requires USDC - get from https://spl-token-faucet.com/?token-name=USDC-Dev)...");
    try {
      const subscriberAta = await getAta(USDC_MINT, walletAddr);
      const ix = await getCreateSubscriptionInstructionAsync({
        subscriber: signer,
        service: servicePda,
        plan: planToSubscribe,
        subscription: subPda,
        subscriberTokenAccount: subscriberAta,
        acceptedMint: USDC_MINT,
        treasury: service.treasury,
      });
      subscribeSig = await sendTx(ix, signer);
      console.log("   create_subscription:", subscribeSig);

      console.log("\n4. Waiting 12s for billing to become due...");
      await new Promise((r) => setTimeout(r, 12000));

      const subNow = await fetchMaybeSubscriptionAccount(rpc, subPda);
      if (subNow.exists && Number(subNow.data.nextBillingTimestamp) <= Date.now() / 1000) {
        console.log("5. Collecting payment...");
        const crankerAta = await getAta(USDC_MINT, walletAddr);
        const ix = getCollectPaymentInstruction({
          cranker: signer,
          service: servicePda,
          subscription: subPda,
          subscriberTokenAccount: subNow.data.subscriberTokenAccount,
          treasury: service.treasury,
          crankerTokenAccount: crankerAta,
          acceptedMint: USDC_MINT,
        });
        collectSig = await sendTx(ix, signer);
        console.log("   collect_payment:", collectSig);
      } else {
        console.log("5. Subscription not yet due, skipping collect");
      }
    } catch (e) {
      console.error("   Subscribe/collect error:", e);
    }
  } else {
    console.log("3. Subscription already exists, skipping");
  }

  console.log("\n--- Devnet Transaction Links ---\n");
  const base = "https://explorer.solana.com/tx";
  if (initSig) console.log(`initialize_service: ${base}/${initSig}?cluster=devnet`);
  if (createPlanSig) console.log(`create_plan: ${base}/${createPlanSig}?cluster=devnet`);
  if (subscribeSig) console.log(`create_subscription: ${base}/${subscribeSig}?cluster=devnet`);
  if (collectSig) console.log(`collect_payment: ${base}/${collectSig}?cluster=devnet`);
  console.log("\nProgram upgrade:", "https://explorer.solana.com/tx/4RLriJ6n84ytxf6b1XwKraVyUKsVf6JH46M1dyavPndKriAJ51rbRd4ptY4r8Wd6jSmmkTHLhfC29QMS4tTFQjPo?cluster=devnet");
}

main().catch(console.error);
