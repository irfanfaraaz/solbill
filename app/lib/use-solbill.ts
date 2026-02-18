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
  type PlanAccount,
  type ServiceAccount,
  type SubscriptionAccount,
} from "../generated/solbill";

export function useSolbill() {
  const { wallet, status } = useWalletConnection();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const client = useSolanaClient() as any;
  const { send, isSending } = useSendTransaction();

  const [loading, setLoading] = useState(false);
  const [service, setService] = useState<ServiceAccount | null>(null);
  const [plans, setPlans] = useState<PlanAccount[]>([]);
  const [userSubscriptions, setUserSubscriptions] = useState<
    Array<{ plan: Address; account: SubscriptionAccount }>
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
          getAddressEncoder().encode(plan),
          getAddressEncoder().encode(subscriber),
        ],
      });
      return pda;
    },
    []
  );

  const refresh = useCallback(async () => {
    if (!walletAddress || !client) return;
    setLoading(true);
    try {
      const serviceAddr = await getServiceAddress(walletAddress);
      const maybeService = await fetchMaybeServiceAccount(
        client.rpc,
        serviceAddr
      );

      if (maybeService.exists) {
        setService(maybeService.data);

        // Fetch plans
        const planAddresses: Address[] = [];
        for (let i = 0; i < Number(maybeService.data.planCount); i++) {
          planAddresses.push(await getPlanAddress(serviceAddr, i));
        }

        if (planAddresses.length > 0) {
          const allPlans = await fetchAllPlanAccount(client.rpc, planAddresses);
          const pData = allPlans.map((p) => p.data);
          setPlans(pData);

          // Fetch user subscriptions for these plans
          const subAddresses: Address[] = [];
          for (const p of planAddresses) {
            subAddresses.push(await getSubscriptionAddress(p, walletAddress));
          }
          const allSubs = await fetchAllMaybeSubscriptionAccount(
            client.rpc,
            subAddresses
          );
          const activeSubs: Array<{
            plan: Address;
            account: SubscriptionAccount;
          }> = [];
          for (let i = 0; i < allSubs.length; i++) {
            const sub = allSubs[i];
            if (sub.exists) {
              activeSubs.push({
                plan: planAddresses[i],
                account: sub.data,
              });
            }
          }
          setUserSubscriptions(activeSubs);
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
  }, [
    walletAddress,
    client,
    getServiceAddress,
    getPlanAddress,
    getSubscriptionAddress,
  ]);

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
      setTxStatus(`Service initialized! ${signature?.slice(0, 10)}...`);
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
      setTxStatus(`Plan created! ${signature?.slice(0, 10)}...`);
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
      setTxStatus(`Subscription successful! ${signature?.slice(0, 10)}...`);
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
  };
}
