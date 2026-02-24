"use client";

import { useState } from "react";
import { useWalletConnection } from "@solana/react-hooks";
import { motion, AnimatePresence } from "framer-motion";
import {
  CreditCard,
  Store,
  Plus,
  ArrowRight,
  Clock,
  ShieldCheck,
  Package,
  Loader2,
} from "lucide-react";
import { cn } from "../../lib/utils";
import { CreatePlanModal } from "./CreatePlanModal";
import { useSolbill, type PlanWithAddress } from "../../lib/use-solbill";
import { USDC_MINT, SOL_MINT } from "../../lib/solbill-config";
import type {
  PlanAccount,
  ServiceAccount,
  SubscriptionAccount,
} from "../../generated/solbill";
import type { Address } from "@solana/kit";

const LoadingSpinner = () => (
  <div className="flex items-center justify-center p-12">
    <Loader2 className="h-8 w-8 animate-spin text-primary" />
  </div>
);

const formatAmount = (amount: bigint | number, mint?: string) => {
  if (mint === SOL_MINT) {
    return `${Number(amount) / 1e9} SOL`;
  }
  return `${Number(amount) / 1e6} USDC`;
};

const MerchantView = ({
  service,
  plans,
  onInitialize,
  onCreatePlan,
  loading,
  isSending,
}: {
  service: ServiceAccount | null;
  plans: PlanAccount[];
  onInitialize: (mint: Address) => void;
  onCreatePlan: () => void;
  loading: boolean;
  isSending: boolean;
}) => {
  const [selectedMint, setSelectedMint] = useState<Address>(
    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU" as Address
  );

  if (loading) return <LoadingSpinner />;

  if (!service) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="flex flex-col items-center justify-center space-y-6 rounded-3xl border border-dashed border-border-low bg-card/50 p-12 text-center"
      >
        <div className="rounded-full bg-primary/10 p-6">
          <Store className="h-10 w-10 text-primary" />
        </div>
        <div className="space-y-2">
          <h3 className="text-xl font-semibold text-foreground">
            Setup Billing Service
          </h3>
          <p className="max-w-xs text-sm text-muted">
            Initialize your on-chain billing service to start creating
            subscription plans.
          </p>
        </div>
        <div className="space-y-4 w-full max-w-sm">
          <div className="flex flex-col text-left space-y-1.5 w-full">
            <label className="text-sm font-medium text-foreground">
              Select Payment Token
            </label>
            <select
              value={selectedMint}
              onChange={(e) => setSelectedMint(e.target.value as Address)}
              className="w-full rounded-xl border border-border-low bg-background px-4 py-3 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
            >
              <option value={USDC_MINT}>Devnet USDC</option>
              <option value={SOL_MINT}>Native Solana (WSOL)</option>
            </select>
          </div>
          <button
            onClick={() => onInitialize(selectedMint)}
            disabled={isSending}
            className="flex w-full items-center justify-center gap-2 rounded-full bg-blue-600 px-8 py-3 font-semibold text-white transition hover:bg-blue-700 disabled:opacity-50 cursor-pointer"
          >
            {isSending ? "Initializing..." : "Initialize Service"}
          </button>
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="space-y-6"
    >
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          <h3 className="text-xl font-medium tracking-tight text-foreground">
            Billing Service
          </h3>
          <p className="text-xs text-muted font-mono truncate max-w-[200px]">
            Mint: {service.acceptedMint.toString()}
          </p>
        </div>
        <button
          onClick={onCreatePlan}
          className="flex items-center gap-2 rounded-full bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition hover:bg-blue-700 cursor-pointer"
        >
          <Plus className="h-4 w-4" />
          Create Plan
        </button>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        {plans.length === 0 ? (
          <div className="col-span-2 flex flex-col items-center justify-center p-12 rounded-2xl border border-dashed border-border-low text-muted text-sm">
            No plans created yet.
          </div>
        ) : (
          plans.map((plan, index) => (
            <div
              key={index}
              className="flex flex-col rounded-2xl border border-border-low bg-card p-6 shadow-sm transition hover:shadow-md"
            >
              <div className="flex items-start justify-between">
                <div className="rounded-lg bg-foreground/5 p-2.5">
                  <Package className="h-5 w-5 text-foreground/70" />
                </div>
                {plan.isActive ? (
                  <span className="rounded-full bg-emerald-500/10 px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider text-emerald-600">
                    Active
                  </span>
                ) : (
                  <span className="rounded-full bg-orange-500/10 px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider text-orange-600">
                    Paused
                  </span>
                )}
              </div>
              <div className="mt-4">
                <h3 className="font-bold text-foreground">
                  {new TextDecoder()
                    .decode(plan.name as unknown as Uint8Array)
                    .replace(/\0/g, "")}
                </h3>
                <p className="text-xs text-muted">Plan ID: {plan.planIndex}</p>
              </div>
              <div className="mt-6 flex items-baseline gap-1">
                <span className="text-2xl font-bold tracking-tight text-foreground">
                  {formatAmount(plan.amount, service?.acceptedMint?.toString())}
                </span>
                <span className="text-sm text-muted">
                  {Number(plan.maxBillingCycles) === 1
                    ? "/ one-time"
                    : "/ term"}
                </span>
              </div>
              <div className="mt-6 flex items-center justify-between border-t border-border-low pt-4 text-xs text-muted">
                {Number(plan.maxBillingCycles) === 1 ? (
                  <span className="flex items-center gap-1 text-primary">
                    <Package className="h-3 w-3" /> One-Time Purchase
                  </span>
                ) : (
                  <>
                    <span className="flex items-center gap-1">
                      <Clock className="h-3 w-3" />{" "}
                      {Number(plan.interval) / 86400} days
                    </span>
                    <span className="flex items-center gap-1 text-primary">
                      <Package className="h-3 w-3" />{" "}
                      {Number(plan.maxBillingCycles) === 0
                        ? "Infinite"
                        : `${plan.maxBillingCycles} cycles`}
                    </span>
                  </>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </motion.div>
  );
};

const SubscriberView = ({
  allPlans,
  subscriptions,
  loading,
  onSubscribe,
  onCancelSubscription,
  serviceMint,
}: {
  allPlans: PlanWithAddress[];
  subscriptions: Array<{
    plan: Address;
    account: SubscriptionAccount;
    address: Address;
  }>;
  loading: boolean;
  onSubscribe: (plan: Address) => void;
  onCancelSubscription: (plan: Address, subscription: Address) => void;
  serviceMint?: string;
}) => {
  if (loading) return <LoadingSpinner />;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-bold text-foreground">Available Plans</h3>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        {allPlans.map((plan, i) => {
          const isSubscribed = subscriptions.some(
            (s) => s.plan === plan.address
          );
          return (
            <div
              key={i}
              className="flex flex-col rounded-2xl border border-border-low bg-card p-6 shadow-sm"
            >
              <h4 className="font-bold text-foreground">
                {new TextDecoder()
                  .decode(new Uint8Array(Object.values(plan.name)))
                  .replace(/\0/g, "")}
              </h4>
              <p className="mt-2 text-2xl font-bold">
                {formatAmount(plan.amount, serviceMint)}
                <span className="text-sm text-muted font-normal ml-1">
                  {Number(plan.maxBillingCycles) === 1
                    ? "/ one-time"
                    : "/ term"}
                </span>
              </p>
              <button
                disabled={isSubscribed}
                onClick={() => onSubscribe(plan.address)}
                className="mt-6 rounded-xl bg-blue-600 py-2 text-white font-semibold disabled:opacity-50 transition hover:bg-blue-700 cursor-pointer"
              >
                {isSubscribed ? "Subscribed" : "Subscribe Now"}
              </button>
            </div>
          );
        })}
      </div>

      <div className="flex items-center justify-between mt-10">
        <h3 className="text-xl font-bold text-foreground">My Subscriptions</h3>
      </div>

      {subscriptions.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-2xl border border-dashed border-border-low bg-card p-12 text-center">
          <div className="mb-4 rounded-full bg-foreground/5 p-4">
            <CreditCard className="h-8 w-8 text-foreground/40" />
          </div>
          <h4 className="text-lg font-semibold text-foreground">
            No active subscriptions
          </h4>
          <p className="mt-1 text-sm text-muted">
            Browse plans to get started.
          </p>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {subscriptions.map((sub, i) => (
            <div
              key={i}
              className="flex flex-col rounded-2xl border border-border-low bg-card p-6 shadow-sm"
            >
              <div className="flex items-center justify-between">
                <div className="rounded-lg bg-primary/10 p-2">
                  <ShieldCheck className="h-5 w-5 text-primary" />
                </div>
                <span className="rounded-full bg-emerald-500/10 px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider text-emerald-600">
                  {sub.account.status}
                </span>
              </div>
              <div className="mt-4">
                <h4 className="font-bold text-foreground">
                  Plan: {sub.plan.slice(0, 8)}...
                </h4>
                <p className="text-sm text-muted">
                  Auto-renews:{" "}
                  {new Date(
                    Number(sub.account.nextBillingTimestamp) * 1000
                  ).toLocaleDateString()}
                </p>
              </div>
              <div className="mt-6 pt-4 border-t border-border-low flex items-center justify-between">
                <div>
                  <p className="text-xs text-muted uppercase tracking-wider">
                    Amount
                  </p>
                  <p className="font-bold text-foreground">
                    {formatAmount(sub.account.amount, serviceMint)}
                  </p>
                </div>
                <button
                  disabled={loading}
                  onClick={() => {
                    if (
                      window.confirm(
                        "Are you sure you want to cancel this subscription?"
                      )
                    ) {
                      onCancelSubscription(sub.plan, sub.address);
                    }
                  }}
                  className="text-xs font-semibold text-red-500 hover:text-red-600 hover:underline cursor-pointer disabled:opacity-50"
                >
                  Cancel
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export function SolBill() {
  const { status } = useWalletConnection();
  const solbill = useSolbill();

  const [activeTab, setActiveTab] = useState<"merchant" | "subscriber">(
    "subscriber"
  );
  const [isPlanModalOpen, setPlanModalOpen] = useState(false);
  const [lookupAddress, setLookupAddress] = useState("");

  // Handlers
  const handleInitializeService = async (mint: Address) => {
    const treasury = await solbill.getAssociatedTokenAddress(
      mint,
      solbill.walletAddress!
    );
    await solbill.initializeService(mint, treasury);
    await solbill.refresh();
  };

  const handleCreatePlan = async (args: {
    name: string;
    amount: number | bigint;
    crankReward: number | bigint;
    interval: number | bigint;
    gracePeriod: number | bigint;
    maxBillingCycles: number | bigint;
  }) => {
    await solbill.createPlan(args);
    setPlanModalOpen(false);
  };

  const handleSubscribe = async (plan: Address) => {
    if (!solbill.service) return;
    const subscriberAta = await solbill.getAssociatedTokenAddress(
      solbill.service.acceptedMint,
      solbill.walletAddress!
    );
    await solbill.createSubscription(plan, subscriberAta);
  };

  const handleCancelSubscription = async (
    plan: Address,
    subscription: Address
  ) => {
    await solbill.cancelSubscription(plan, subscription);
  };

  if (status !== "connected") {
    return (
      <div className="flex flex-col items-center justify-center space-y-6 rounded-3xl border border-border-low bg-card p-12 text-center shadow-xl">
        <div className="rounded-full bg-foreground/5 p-6 ring-1 ring-inset ring-foreground/10 shadow-inner">
          <CreditCard className="h-10 w-10 text-muted" />
        </div>
        <div className="space-y-2">
          <h2 className="text-2xl font-semibold text-foreground tracking-tight">
            Autonomous Billing
          </h2>
          <p className="max-w-xs text-sm text-muted leading-relaxed">
            Connect your wallet to manage your subscriptions or setup your
            merchant dashboard.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="w-full space-y-8">
      <div className="flex justify-center">
        <div className="inline-flex items-center gap-1 rounded-full border border-border-low bg-card p-1 shadow-inner backdrop-blur-sm">
          <button
            onClick={() => setActiveTab("subscriber")}
            className={cn(
              "flex items-center gap-2 rounded-full px-6 py-2.5 text-sm font-medium transition-all duration-300 cursor-pointer",
              activeTab === "subscriber"
                ? "bg-foreground text-background shadow-lg"
                : "text-muted hover:text-foreground hover:bg-foreground/5"
            )}
          >
            <CreditCard className="h-4 w-4" />
            Subscriber
          </button>
          <button
            onClick={() => setActiveTab("merchant")}
            className={cn(
              "flex items-center gap-2 rounded-full px-6 py-2.5 text-sm font-medium transition-all duration-300 cursor-pointer",
              activeTab === "merchant"
                ? "bg-foreground text-background shadow-lg"
                : "text-muted hover:text-foreground hover:bg-foreground/5"
            )}
          >
            <Store className="h-4 w-4" />
            Merchant
          </button>
        </div>
      </div>

      <AnimatePresence mode="wait">
        <div className="min-h-[400px]">
          {activeTab === "merchant" ? (
            <MerchantView
              key="merchant"
              service={solbill.service}
              plans={solbill.plans}
              loading={solbill.loading}
              isSending={solbill.isSending}
              onInitialize={handleInitializeService}
              onCreatePlan={() => setPlanModalOpen(true)}
            />
          ) : (
            <div className="space-y-6">
              {!solbill.service && (
                <div className="flex flex-col space-y-2 rounded-2xl border border-border-low bg-card p-6 shadow-sm">
                  <h4 className="font-semibold text-foreground">
                    Browse Merchant
                  </h4>
                  <p className="text-xs text-muted">
                    Enter the merchant&apos;s authority address to view their
                    plans.
                  </p>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      placeholder="Merchant Solana Address"
                      className="flex-1 rounded-xl border border-border-low bg-background px-4 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                      value={lookupAddress}
                      onChange={(e) => setLookupAddress(e.target.value)}
                    />
                    <button
                      onClick={() => solbill.refresh(lookupAddress as Address)}
                      className="rounded-xl bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground hover:opacity-90 transition"
                    >
                      Lookup
                    </button>
                  </div>
                </div>
              )}
              <SubscriberView
                key="subscriber"
                allPlans={solbill.plans}
                subscriptions={solbill.userSubscriptions}
                loading={solbill.loading || solbill.isSending}
                onSubscribe={handleSubscribe}
                onCancelSubscription={handleCancelSubscription}
                serviceMint={solbill.service?.acceptedMint?.toString()}
              />
            </div>
          )}
        </div>
      </AnimatePresence>

      <CreatePlanModal
        isOpen={isPlanModalOpen}
        onClose={() => setPlanModalOpen(false)}
        onSubmit={handleCreatePlan}
        isSending={solbill.isSending}
      />

      {(solbill.txStatus || solbill.loading) && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="fixed bottom-6 right-6 max-w-sm rounded-2xl border border-primary/20 bg-card p-4 shadow-2xl backdrop-blur-md z-50"
        >
          <div className="flex items-center gap-3">
            {solbill.loading || solbill.isSending ? (
              <Loader2 className="h-4 w-4 animate-spin text-primary" />
            ) : (
              <ShieldCheck className="h-4 w-4 text-green-500" />
            )}
            <p className="text-xs font-medium text-foreground">
              {solbill.txStatus || "Syncing with blockchain..."}
            </p>
          </div>
        </motion.div>
      )}

      <div className="flex flex-col items-center gap-2 pt-8 text-center border-t border-border-low text-[10px] font-bold uppercase tracking-[0.2em] text-muted/30">
        <ArrowRight className="h-3 w-3 rotate-90" />
        Powered by SolBill Autonomous Billing
      </div>
    </div>
  );
}
