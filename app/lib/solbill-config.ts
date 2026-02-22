import { Address } from "@solana/kit";

export const SOLANA_RPC_URL =
  process.env.NEXT_PUBLIC_SOLANA_RPC_URL || "https://api.devnet.solana.com";

export const SOLBILL_PROGRAM_ID =
  (process.env.NEXT_PUBLIC_SOLBILL_PROGRAM_ID as Address) ||
  ("AK2xA7SHMKPqvQEirLUNf4gRQjzpQZT3q6v3d62kLyzx" as Address);

export const USDC_MINT =
  (process.env.NEXT_PUBLIC_USDC_MINT as Address) ||
  ("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU" as Address);

export const SOL_MINT =
  (process.env.NEXT_PUBLIC_SOL_MINT as Address) ||
  ("So11111111111111111111111111111111111111112" as Address);
