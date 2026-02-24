import { useState } from "react";
import { X, Loader2 } from "lucide-react";

type PlanType = "recurring" | "installment" | "one-time";

export function CreatePlanModal({
  isOpen,
  onClose,
  onSubmit,
  isSending,
}: {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (args: {
    name: string;
    amount: number | bigint;
    crankReward: number | bigint;
    interval: number | bigint;
    gracePeriod: number | bigint;
    maxBillingCycles: number | bigint;
  }) => Promise<void>;
  isSending: boolean;
}) {
  const [name, setName] = useState("Premium Plan");
  const [planType, setPlanType] = useState<PlanType>("recurring");

  // using human decimal for UI
  const [amount, setAmount] = useState<number>(10);
  const [intervalDays, setIntervalDays] = useState<number>(30);
  const [gracePeriodDays, setGracePeriodDays] = useState<number>(7);
  const [maxCycles, setMaxCycles] = useState<number>(12);
  const crankReward = 0.5;

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (
      !name ||
      amount <= 0 ||
      (planType !== "one-time" && intervalDays <= 0) ||
      gracePeriodDays < 0
    )
      return;

    let finalInterval = intervalDays;
    let finalMaxCycles = maxCycles;

    if (planType === "recurring") {
      finalMaxCycles = 0;
    } else if (planType === "one-time") {
      finalInterval = 365; // Arbitrary long time
      finalMaxCycles = 1;
    }

    onSubmit({
      name,
      amount: BigInt(Math.floor(amount * 10 ** 6)), // Convert to USDC decimals
      crankReward: BigInt(Math.floor(crankReward * 10 ** 6)),
      interval: BigInt(Math.floor(finalInterval * 86400)), // Convert days to seconds
      gracePeriod: BigInt(Math.floor(gracePeriodDays * 86400)),
      maxBillingCycles: BigInt(finalMaxCycles),
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-2xl border border-border-low bg-card p-6 shadow-xl">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold text-foreground">Create New Plan</h2>
          <button
            onClick={onClose}
            className="p-2 rounded-full hover:bg-foreground/5 transition cursor-pointer text-muted hover:text-foreground"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-1.5">
            <label className="text-sm font-medium text-foreground">
              Plan Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-xl border border-border-low bg-background px-4 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
              maxLength={32}
              required
            />
          </div>

          <div className="flex gap-4">
            <div className="space-y-1.5 flex-1">
              <label className="text-sm font-medium text-foreground">
                Plan Type
              </label>
              <select
                value={planType}
                onChange={(e) => setPlanType(e.target.value as PlanType)}
                className="w-full rounded-xl border border-border-low bg-background px-4 py-3 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
              >
                <option value="recurring">Recurring (Infinite)</option>
                <option value="installment">Fixed-Term (Installments)</option>
                <option value="one-time">One-Time Payment</option>
              </select>
            </div>
            <div className="space-y-1.5 flex-1">
              <label className="text-sm font-medium text-foreground">
                Amount (USDC)
              </label>
              <input
                type="number"
                min="0.01"
                step="0.01"
                value={amount}
                onChange={(e) => setAmount(Number(e.target.value))}
                className="w-full rounded-xl border border-border-low bg-background px-4 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 h-[46px]"
                required
              />
            </div>
          </div>

          <div className="flex gap-4">
            {planType !== "one-time" && (
              <div className="space-y-1.5 flex-1 animate-in fade-in slide-in-from-top-2">
                <label className="text-sm font-medium text-foreground">
                  Billing Interval (Days)
                </label>
                <input
                  type="number"
                  min="1"
                  value={intervalDays}
                  onChange={(e) => setIntervalDays(Number(e.target.value))}
                  className="w-full rounded-xl border border-border-low bg-background px-4 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
                  required
                />
              </div>
            )}

            {planType === "installment" && (
              <div className="space-y-1.5 flex-1 animate-in fade-in slide-in-from-top-2">
                <label className="text-sm font-medium text-foreground">
                  Max Cycles
                </label>
                <input
                  type="number"
                  min="1"
                  value={maxCycles}
                  onChange={(e) => setMaxCycles(Number(e.target.value))}
                  className="w-full rounded-xl border border-border-low bg-background px-4 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
                  required
                />
              </div>
            )}

            {planType === "one-time" && (
              <div className="space-y-1.5 flex-1 text-xs text-muted/80 flex items-center px-1">
                One-time plans execute a single payment collection and then
                automatically complete.
              </div>
            )}
          </div>

          <div className="space-y-1.5">
            <label className="text-sm font-medium text-foreground">
              Grace Period (Days)
            </label>
            <input
              type="number"
              min="0"
              value={gracePeriodDays}
              onChange={(e) => setGracePeriodDays(Number(e.target.value))}
              className="w-full rounded-xl border border-border-low bg-background px-4 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
              required
            />
            <p className="text-[10px] text-muted">
              Time allowed past due date before active status is lost.
            </p>
          </div>

          <div className="pt-4">
            <button
              type="submit"
              disabled={isSending}
              className="flex w-full items-center justify-center gap-2 rounded-xl bg-blue-600 px-8 py-3 font-semibold text-white transition hover:bg-blue-700 disabled:opacity-50 cursor-pointer"
            >
              {isSending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Creating...
                </>
              ) : (
                "Create Plan"
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
