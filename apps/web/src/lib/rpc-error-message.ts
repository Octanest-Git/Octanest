/** Map RPC failure payloads to user-facing copy (loaders / setup). */

export type RpcErrorLike = {
  code?: string;
  message?: string;
};

const NETWORK_ERROR = "Can't reach Oxidean. Check your connection and try again.";

const DB_NOT_READY =
  "Database not ready. Migrations may still be running — wait a moment and refresh.";

/** Generic auth.internal messages that should get the migrate/schema hint. */
const GENERIC_INTERNAL = new Set([
  "authentication failed",
  "bootstrap operation failed",
  "profile operation failed",
]);

/**
 * Prefer the API error message; hint migrate/schema issues for generic
 * `auth.internal` payloads. Use {@link networkErrorMessage} only for thrown
 * fetch/connection failures.
 */
export function rpcErrorMessage(error: RpcErrorLike | null | undefined): string {
  if (!error) return NETWORK_ERROR;
  const msg = error.message?.trim();
  if (error.code === "auth.internal") {
    if (msg && !GENERIC_INTERNAL.has(msg.toLowerCase())) return msg;
    return DB_NOT_READY;
  }
  if (msg) return msg;
  return NETWORK_ERROR;
}

export function networkErrorMessage(): string {
  return NETWORK_ERROR;
}
