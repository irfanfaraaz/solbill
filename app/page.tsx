"use client";
import { useWalletConnection } from "@solana/react-hooks";
import { SolBill } from "./components/solbill/solbill";

export default function Home() {
  const { connectors, connect, disconnect, wallet, status } =
    useWalletConnection();

  const address = wallet?.account.address.toString();

  return (
    <div className="relative min-h-screen overflow-x-clip bg-bg1 text-foreground">
      <main className="relative z-10 mx-auto flex min-h-screen max-w-4xl flex-col gap-10 border-x border-border-low px-6 py-16">
        <header className="space-y-4">
          <div className="inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/5 px-3 py-1 text-[10px] font-bold uppercase tracking-widest text-primary">
            <span className="relative flex h-2 w-2">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-primary opacity-75"></span>
              <span className="relative inline-flex h-2 w-2 rounded-full bg-primary"></span>
            </span>
            Alpha Release
          </div>
          <h1 className="text-4xl font-bold tracking-tight text-foreground sm:text-5xl">
            SolBill Billing Engine
          </h1>
          <p className="max-w-2xl text-lg leading-relaxed text-muted">
            The first truly autonomous, serverless billing protocol on Solana.
            Enable recurring payments without escrow or custody.
          </p>
        </header>

        <section className="w-full max-w-3xl space-y-4 rounded-2xl border border-border-low bg-card p-6 shadow-2xl">
          <div className="flex items-start justify-between gap-4">
            <div className="space-y-1">
              <p className="text-lg font-semibold text-foreground">
                Wallet connection
              </p>
              <p className="text-sm text-muted">
                Connect your wallet to interact with the SolBill protocol.
              </p>
            </div>
            <span
              className={
                status === "connected"
                  ? "text-green-500 font-bold text-xs uppercase"
                  : "text-muted font-bold text-xs uppercase"
              }
            >
              {status === "connected" ? "Online" : "Offline"}
            </span>
          </div>

          <div className="grid gap-3 sm:grid-cols-2">
            {connectors.map((connector) => (
              <button
                key={connector.id}
                onClick={() => connect(connector.id)}
                disabled={status === "connecting"}
                className="group flex items-center justify-between rounded-xl border border-border-low bg-card px-4 py-3 text-left text-sm font-medium transition hover:border-foreground/20 hover:shadow-lg cursor-pointer disabled:cursor-not-allowed disabled:opacity-60"
              >
                <span className="flex flex-col">
                  <span className="text-base text-foreground">
                    {connector.name}
                  </span>
                  <span className="text-xs text-muted">
                    {status === "connecting"
                      ? "Connecting…"
                      : status === "connected" &&
                          wallet?.connector.id === connector.id
                        ? "Active"
                        : "Tap to connect"}
                  </span>
                </span>
              </button>
            ))}
          </div>

          <div className="flex flex-wrap items-center gap-3 border-t border-border-low pt-4 text-sm">
            <span className="rounded-lg border border-border-low bg-cream px-3 py-2 font-mono text-xs text-muted">
              {address ?? "No wallet connected"}
            </span>
            {status === "connected" && (
              <button
                onClick={() => disconnect()}
                className="inline-flex items-center gap-2 rounded-lg border border-border-low bg-card px-3 py-2 text-xs font-semibold text-red-500 transition hover:bg-red-500/5 cursor-pointer"
              >
                Disconnect
              </button>
            )}
          </div>
        </section>

        <SolBill />
      </main>
    </div>
  );
}
