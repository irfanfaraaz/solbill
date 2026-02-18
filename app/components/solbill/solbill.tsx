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
  Activity,
  ShieldCheck,
  Package,
} from "lucide-react";
import { cn } from "../../lib/utils";

const MerchantView = () => (
  <motion.div
    initial={{ opacity: 0, y: 10 }}
    animate={{ opacity: 1, y: 0 }}
    className="space-y-6"
  >
    <div className="flex items-center justify-between">
      <h3 className="text-xl font-medium tracking-tight text-foreground">
        Billing Service
      </h3>
      <button className="flex items-center gap-2 rounded-full border border-primary/20 bg-primary/10 px-4 py-2 text-sm font-medium text-primary transition hover:bg-primary/20 cursor-pointer">
        <Plus className="h-4 w-4" />
        Create Plan
      </button>
    </div>

    <div className="grid gap-4 sm:grid-cols-2">
      {[1, 2].map((i) => (
        <div
          key={i}
          className="group relative overflow-hidden rounded-2xl border border-border-low bg-card p-5 transition hover:border-primary/30"
        >
          <div className="flex items-start justify-between">
            <div className="space-y-1">
              <p className="text-xs font-medium uppercase tracking-wider text-muted font-mono">
                Plan 0{i}
              </p>
              <h4 className="text-lg font-semibold text-foreground">
                Premium Tier
              </h4>
            </div>
            <span className="rounded-full bg-green-500/10 px-2 py-1 text-[10px] font-bold uppercase text-green-500">
              Active
            </span>
          </div>
          <div className="mt-6 flex items-baseline gap-1">
            <span className="text-2xl font-bold tracking-tight text-foreground">
              $10.00
            </span>
            <span className="text-sm text-muted">/ month</span>
          </div>
          <div className="mt-6 flex items-center justify-between border-t border-border-low pt-4 text-xs text-muted">
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" /> 30 days
            </span>
            <span className="flex items-center gap-1">
              <Activity className="h-3 w-3" /> 12 Active
            </span>
          </div>
        </div>
      ))}
    </div>
  </motion.div>
);

const SubscriberView = () => (
  <motion.div
    initial={{ opacity: 0, y: 10 }}
    animate={{ opacity: 1, y: 0 }}
    className="space-y-6"
  >
    <div className="rounded-2xl border border-primary/20 bg-primary/5 p-6 backdrop-blur-sm">
      <div className="flex items-center gap-4">
        <div className="rounded-xl bg-primary/10 p-3">
          <ShieldCheck className="h-6 w-6 text-primary" />
        </div>
        <div>
          <h3 className="text-lg font-semibold text-foreground">
            Secure Billing
          </h3>
          <p className="text-sm text-muted">
            All subscriptions are secured via token delegation. No funds ever
            leave your wallet without your approval.
          </p>
        </div>
      </div>
    </div>

    <div className="space-y-4">
      <h3 className="text-xl font-medium tracking-tight text-foreground">
        My Subscriptions
      </h3>
      <div className="rounded-2xl border border-border-low bg-card divide-y divide-border-low overflow-hidden">
        {[1].map((i) => (
          <div
            key={i}
            className="flex items-center justify-between p-5 hover:bg-foreground/2 transition"
          >
            <div className="flex items-center gap-4">
              <div className="rounded-full bg-foreground/5 p-3">
                <Package className="h-5 w-5 text-foreground/70" />
              </div>
              <div>
                <p className="font-medium text-foreground">SolBill Pro</p>
                <p className="text-xs text-muted">Next billing: Mar 17, 2026</p>
              </div>
            </div>
            <div className="flex items-center gap-6">
              <span className="text-sm font-semibold text-foreground">
                $10.00
              </span>
              <button className="text-xs font-medium text-red-500 transition hover:underline cursor-pointer">
                Cancel
              </button>
            </div>
          </div>
        ))}
        <div className="flex items-center justify-center p-8 text-center bg-foreground/1">
          <button className="text-sm font-medium text-primary hover:underline flex items-center gap-2 cursor-pointer">
            <Plus className="h-4 w-4" /> Discover more plans
          </button>
        </div>
      </div>
    </div>
  </motion.div>
);

export function SolBill() {
  const { status } = useWalletConnection();
  const [activeTab, setActiveTab] = useState<"merchant" | "subscriber">(
    "subscriber"
  );

  if (status !== "connected") {
    return (
      <div className="flex flex-col items-center justify-center space-y-6 rounded-3xl border border-border-low bg-card p-12 text-center shadow-xl">
        <div className="rounded-full bg-foreground/5 p-6 ring-1 ring-inset ring-foreground/10">
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
        <div className="inline-flex items-center gap-1 rounded-full border border-border-low bg-card p-1 shadow-inner">
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

      <div className="min-h-[400px]">
        <AnimatePresence mode="wait">
          {activeTab === "merchant" ? (
            <MerchantView key="merchant" />
          ) : (
            <SubscriberView key="subscriber" />
          )}
        </AnimatePresence>
      </div>

      <div className="flex flex-col items-center gap-2 pt-8 text-center border-t border-border-low text-[10px] font-bold uppercase tracking-[0.2em] text-muted/50">
        <ArrowRight className="h-3 w-3 rotate-90" />
        Powered by SolBill Autonomous Billing
      </div>
    </div>
  );
}
