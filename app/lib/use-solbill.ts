"use client";

import { useState, useEffect, useCallback } from "react";
import {
  useWalletConnection,
  useSendTransaction,
  useSolanaClient,
} from "@solana/react-hooks";
import {
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
  type Address,
  type TransactionSigner,
} from "@solana/kit";
import {
  SOLBILL_PROGRAM_ADDRESS,
  fetchMaybeServiceAccount,
  fetchAllPlanAccount,
  fetchAllMaybeSubscriptionAccount,
  getCreatePlanInstructionAsync,
  getCreateSubscriptionInstructionAsync,
  getInitializeServiceInstructionAsync,
  getCancelSubscriptionInstruction,
  type PlanAccount,
  type ServiceAccount,
  type SubscriptionAccount,
} from "../generated/solbill";

export type PlanWithAddress = PlanAccount & { address: Address };
export type ServiceWithAddress = ServiceAccount & { address: Address };

const ASSOCIATED_TOKEN_PROGRAM_ID =
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" as Address;
const TOKEN_PROGRAM_ID =
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" as Address;

export function useSolbill() {
  const { wallet, status } = useWalletConnection();
  const client = useSolanaClient();
  const { send, isSending } = useSendTransaction();

  const [loading, setLoading] = useState(false);
  const [service, setService] = useState<ServiceWithAddress | null>(null);
  const [plans, setPlans] = useState<PlanWithAddress[]>([]);
  const [userSubscriptions, setUserSubscriptions] = useState<
    Array<{ plan: Address; account: SubscriptionAccount; address: Address }>
  >([]);
  const [txStatus, setTxStatus] = useState<string | null>(null);

  const walletAddress = wallet?.account.address as Address | undefined;

  const getServiceAddress = useCallback(async (authority: Address) => {
    const [pda] = await getProgramDerivedAddress({
      programAddress: SOLBILL_PROGRAM_ADDRESS,
      seeds: [
        getBytesEncoder().encode(
          new Uint8Array([115, 101, 114, 118, 105, 99, 101])
        ), // "service"
        getAddressEncoder().encode(authority),
      ],
    });
    return pda;
  }, []);

  const getPlanAddress = useCallback(
    async (serviceAddr: Address, index: number) => {
      const [pda] = await getProgramDerivedAddress({
        programAddress: SOLBILL_PROGRAM_ADDRESS,
        seeds: [
          getBytesEncoder().encode(new Uint8Array([112, 108, 97, 110])), // "plan"
          getAddressEncoder().encode(serviceAddr),
          new Uint8Array([index & 0xff, (index >> 8) & 0xff]), // u16 LE
        ],
      });
      return pda;
    },
    []
  );

  const getSubscriptionAddress = useCallback(
    async (plan: Address, subscriber: Address) => {
      const [pda] = await getProgramDerivedAddress({
        programAddress: SOLBILL_PROGRAM_ADDRESS,
        seeds: [
          getBytesEncoder().encode(
            new Uint8Array([
              115, 117, 98, 115, 99, 114, 105, 112, 116, 105, 111, 110,
            ])
          ), // "subscription"
          getAddressEncoder().encode(subscriber),
          getAddressEncoder().encode(plan),
        ],
      });
      return pda;
    },
    []
  );

  const refresh = useCallback(
    async (merchantAuth?: Address) => {
      const authToUse = merchantAuth || walletAddress;
      if (!authToUse || !client) return;
      setLoading(true);
      try {
        const serviceAddr = await getServiceAddress(authToUse);
        const maybeService = await fetchMaybeServiceAccount(
          client.runtime.rpc,
          serviceAddr
        );

        if (maybeService.exists) {
          setService({ ...maybeService.data, address: serviceAddr });

          // Fetch plans
          const planAddresses: Address[] = [];
          for (let i = 0; i < Number(maybeService.data.planCount); i++) {
            planAddresses.push(await getPlanAddress(serviceAddr, i));
          }

          if (planAddresses.length > 0) {
            const allPlans = await fetchAllPlanAccount(
              client.runtime.rpc,
              planAddresses
            );
            const pData = allPlans.map((p, i) => ({
              ...p.data,
              address: planAddresses[i],
            }));
            setPlans(pData);

            // Fetch user subscriptions for these plans
            if (walletAddress) {
              const subAddresses: Address[] = [];
              for (const p of planAddresses) {
                subAddresses.push(
                  await getSubscriptionAddress(p, walletAddress)
                );
              }
              const allSubs = await fetchAllMaybeSubscriptionAccount(
                client.runtime.rpc,
                subAddresses
              );
              const activeSubs: Array<{
                plan: Address;
                account: SubscriptionAccount;
                address: Address;
              }> = [];
              for (let i = 0; i < allSubs.length; i++) {
                const sub = allSubs[i];
                if (sub.exists) {
                  activeSubs.push({
                    plan: planAddresses[i],
                    account: sub.data,
                    address: subAddresses[i],
                  });
                }
              }
              setUserSubscriptions(activeSubs);
            } else {
              setUserSubscriptions([]);
            }
          } else {
            setPlans([]);
            setUserSubscriptions([]);
          }
        } else {
          setService(null);
          setPlans([]);
        }
      } catch (e) {
        console.error("Failed to fetch SolBill data:", e);
      } finally {
        setLoading(false);
      }
    },
    [
      walletAddress,
      client,
      getServiceAddress,
      getPlanAddress,
      getSubscriptionAddress,
    ]
  );

  const getAssociatedTokenAddress = useCallback(
    async (mint: Address, owner: Address) => {
      const [pda] = await getProgramDerivedAddress({
        programAddress: ASSOCIATED_TOKEN_PROGRAM_ID,
        seeds: [
          getAddressEncoder().encode(owner),
          getAddressEncoder().encode(TOKEN_PROGRAM_ID),
          getAddressEncoder().encode(mint),
        ],
      });
      return pda;
    },
    []
  );

  const pollForAccount = async (
    address: Address,
    maxAttempts = 10,
    delayMs = 1000
  ) => {
    for (let i = 0; i < maxAttempts; i++) {
      try {
        if (!client) return false;
        const info = await client.runtime.rpc.getAccountInfo(address).send();
        if (info && info.value !== null) {
          return true;
        }
      } catch {
        // ignore
      }
      await new Promise((r) => setTimeout(r, delayMs));
    }
    return false;
  };

  useEffect(() => {
    if (status === "connected") {
      refresh();
    }
  }, [walletAddress, status, refresh]);

  // Transactions
  const initializeService = async (
    acceptedMint: Address,
    treasury: Address
  ) => {
    if (!wallet) return;
    try {
      setTxStatus("Initializing billing service...");
      const instruction = await getInitializeServiceInstructionAsync({
        authority: wallet.account as unknown as TransactionSigner,
        acceptedMint,
        treasury,
      });
      const signature = await send({ instructions: [instruction] });
      setTxStatus(`Service initialized! Syncing...`);
      const serviceAddr = await getServiceAddress(
        wallet.account.address as Address
      );
      await pollForAccount(serviceAddr);
      await new Promise((r) => setTimeout(r, 1000));
      await refresh();
      return signature;
    } catch (err) {
      setTxStatus(
        `Error: ${err instanceof Error ? err.message : "Unknown error"}`
      );
      throw err;
    }
  };

  const createPlan = async (args: {
    name: string;
    amount: number | bigint;
    crankReward: number | bigint;
    interval: number | bigint;
    gracePeriod: number | bigint;
    maxBillingCycles: number | bigint;
  }) => {
    if (!wallet || !service) return;
    try {
      setTxStatus("Creating plan...");
      const serviceAddr = await getServiceAddress(walletAddress!);
      const planAddr = await getPlanAddress(serviceAddr, service.planCount);

      const instruction = await getCreatePlanInstructionAsync({
        authority: wallet.account as unknown as TransactionSigner,
        plan: planAddr,
        ...args,
      });
      const signature = await send({ instructions: [instruction] });
      setTxStatus(`Plan created! Syncing...`);
      await pollForAccount(planAddr);
      await new Promise((r) => setTimeout(r, 1000));
      await refresh();
      return signature;
    } catch (err) {
      setTxStatus(
        `Error: ${err instanceof Error ? err.message : "Unknown error"}`
      );
      throw err;
    }
  };

  const createSubscription = async (
    plan: Address,
    subscriberTokenAccount: Address
  ) => {
    if (!wallet || !service) return;
    try {
      setTxStatus("Subscribing...");
      const subscriptionAddr = await getSubscriptionAddress(
        plan,
        walletAddress!
      );

      const serviceAddr = await getServiceAddress(service.authority as Address); // Or use stored addr if available
      const instruction = await getCreateSubscriptionInstructionAsync({
        subscriber: wallet.account as unknown as TransactionSigner,
        service: serviceAddr,
        plan: plan,
        subscription: subscriptionAddr,
        subscriberTokenAccount: subscriberTokenAccount,
        acceptedMint: service.acceptedMint,
        treasury: service.treasury,
      });
      const signature = await send({ instructions: [instruction] });
      setTxStatus(`Subscription successful! Syncing...`);
      await pollForAccount(subscriptionAddr);
      await new Promise((r) => setTimeout(r, 1000));
      await refresh();
      return signature;
    } catch (err) {
      setTxStatus(
        `Error: ${err instanceof Error ? err.message : "Unknown error"}`
      );
      throw err;
    }
  };

  const cancelSubscription = async (
    plan: Address,
    subscriptionAccount: Address
  ) => {
    if (!wallet || !service) return;
    try {
      setTxStatus("Canceling subscription...");
      const serviceAddr = await getServiceAddress(service.authority as Address);

      const instruction = getCancelSubscriptionInstruction({
        subscriber: wallet.account as unknown as TransactionSigner,
        service: serviceAddr,
        subscription: subscriptionAccount,
        subscriberTokenAccount: await getAssociatedTokenAddress(
          service.acceptedMint,
          walletAddress!
        ),
      });

      const signature = await send({ instructions: [instruction] });
      setTxStatus(`Subscription canceled! Syncing...`);
      // Sleep for 2.5 seconds to allow RPC to drop the deleted account before refresh
      await new Promise((resolve) => setTimeout(resolve, 2500));
      await refresh();
      return signature;
    } catch (err) {
      setTxStatus(
        `Error: ${err instanceof Error ? err.message : "Unknown error"}`
      );
      throw err;
    }
  };

  return {
    walletAddress,
    service,
    plans,
    userSubscriptions,
    loading,
    isSending,
    txStatus,
    refresh,
    initializeService,
    createPlan,
    createSubscription,
    cancelSubscription,
    getAssociatedTokenAddress,
  };
}
