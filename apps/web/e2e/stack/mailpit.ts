import { mailpitOrigin } from "./env";

export type MailpitMessage = {
  ID: string;
  To: { Address: string }[];
  Subject: string;
  Text?: string;
  Snippet?: string;
};

export async function mailpitDeleteAll(): Promise<void> {
  const res = await fetch(`${mailpitOrigin()}/api/v1/messages`, {
    method: "DELETE",
  });
  if (!res.ok) {
    throw new Error(`mailpit delete failed: ${res.status}`);
  }
}

export async function mailpitMessages(): Promise<MailpitMessage[]> {
  const res = await fetch(`${mailpitOrigin()}/api/v1/messages`);
  if (!res.ok) {
    throw new Error(`mailpit list failed: ${res.status}`);
  }
  const body = (await res.json()) as { messages?: MailpitMessage[] };
  return body.messages ?? [];
}

export async function waitForMailpit(
  predicate: (m: MailpitMessage) => boolean,
  timeoutMs = 15_000,
): Promise<MailpitMessage> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const messages = await mailpitMessages();
    const hit = messages.find(predicate);
    if (hit) return hit;
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error("timed out waiting for Mailpit message");
}
