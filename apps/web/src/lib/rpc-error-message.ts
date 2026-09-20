/** Map RPC failure payloads to user-facing copy (loaders / setup). */

export type RpcErrorLike = {
  code?: string;
  message?: string;
};

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

const DB_NOT_READY =
  "Database not ready. Migrations may still be running — wait a moment and refresh.";

/**
 * Prefer the API error message; hint migrate/schema issues for `auth.internal`.
 * Use {@link networkErrorMessage} only for thrown fetch/connection failures.
 */
export function rpcErrorMessage(error: RpcErrorLike | null | undefined): string {
  if (!error) return NETWORK_ERROR;
  if (error.code === "auth.internal") return DB_NOT_READY;
  const msg = error.message?.trim();
  if (msg) return msg;
  return NETWORK_ERROR;
}

export function networkErrorMessage(): string {
  return NETWORK_ERROR;
}
