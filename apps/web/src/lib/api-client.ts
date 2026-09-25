import { createClient } from "@oxidean/api-client";

/** Shared browser client — cookies included for session RPCs. */
export const apiClient = createClient({
  baseUrl: typeof window !== "undefined" ? window.location.origin : "http://127.0.0.1:8080",
  credentials: "include",
});
