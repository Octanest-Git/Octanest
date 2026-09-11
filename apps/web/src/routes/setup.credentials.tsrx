import { createFileRoute } from "@octanejs/tanstack-router";

/**
 * RED stub — CredentialsPage intentionally empty until GREEN implements UI-SPEC.
 * Non-nested path via createFileRoute; file name matches Wave 0 import.
 */
export const Route = createFileRoute("/setup/credentials")({
  component: CredentialsPage,
  head: () => ({ meta: [{ title: "Confirm admin · Octanest" }] }),
});

/** Exported for AUTH-06 integration tests. */
export function CredentialsPage() {
  return null;
}
