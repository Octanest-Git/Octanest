import { createElement } from "octane";
import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

/**
 * GIT-04 /settings/ssh-keys list / add / revoke UI
 * (D-SSH-05, D-SSH-06 / T-09-03 / 09-UI-SPEC).
 *
 * Wave 0 RED stubs — greened by 09-07 when ssh-keys.tsrx lands.
 */

const listMock = vi.fn();
const gpgListMock = vi.fn();
const revokeMock = vi.fn();
const addMock = vi.fn();
const meMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: (...args: unknown[]) => meMock(...args),
    },
    sshKey: {
      list: (...args: unknown[]) => listMock(...args),
      revoke: (...args: unknown[]) => revokeMock(...args),
      add: (...args: unknown[]) => addMock(...args),
    },
    gpgKey: {
      list: (...args: unknown[]) => gpgListMock(...args),
      revoke: vi.fn(),
      add: vi.fn(),
    },
  },
}));

type LoaderShape =
  | { kind: "unauthenticated" }
  | { kind: "error"; message: string }
  | {
      kind: "ready";
      user: {
        id: string;
        email: string;
        username: string;
        display_name: string;
        bio: string;
        avatar_url: null;
        role: string;
        profile_incomplete: boolean;
        email_verified: boolean;
        must_change_credentials: boolean;
      };
      keys?: unknown[];
      gpgKeys?: unknown[];
    };

let loaderData: LoaderShape;

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@octanejs/tanstack-router")>();
  return {
    ...actual,
    useLoaderData: () => loaderData,
    Link: (props: {
      to?: string;
      children?: unknown;
      className?: string;
      "aria-current"?: string;
    }) =>
      createElement(
        "a",
        {
          href: props.to ?? "#",
          className: props.className,
          "aria-current": props["aria-current"],
        },
        props.children as never,
      ),
  };
});

const verifiedUser = {
  id: "u1",
  email: "ada@example.com",
  username: "ada",
  display_name: "Ada",
  bio: "",
  avatar_url: null as null,
  role: "user",
  profile_incomplete: false,
  email_verified: true,
  must_change_credentials: false,
};

beforeEach(() => {
  listMock.mockReset();
  gpgListMock.mockReset();
  revokeMock.mockReset();
  addMock.mockReset();
  meMock.mockReset();
  loaderData = { kind: "ready", user: verifiedUser, keys: [], gpgKeys: [] };
  listMock.mockResolvedValue({ ok: true, data: [] });
  gpgListMock.mockResolvedValue({ ok: true, data: [] });
  meMock.mockResolvedValue({ ok: true, data: verifiedUser });
});

afterEach(cleanup);

/** Load ssh-keys page; @vite-ignore keeps the suite collectable before ./ssh-keys exists. */
async function loadSshKeysModule(): Promise<Record<string, unknown>> {
  const rel = "./ssh-keys";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /settings/ssh-keys route missing — implement in 09-07 (GIT-04 / D-SSH-06). Expected list title SSH keys. ${(err as Error).message}`,
    );
  }
}

describe("/settings/ssh-keys (GIT-04 / D-SSH-06 list)", () => {
  it("list title SSH and GPG keys + empty hero No SSH keys + Add SSH key", async () => {
    const mod = await loadSshKeysModule();
    const SshKeysPage = (mod.SshKeysPage ?? mod.default) as unknown;
    const { container } = renderWithQueryClient(SshKeysPage);

    await waitFor(
      () => {
        expect(container.querySelector("h1")?.textContent).toBe("SSH and GPG keys");
      },
      { timeout: 20_000 },
    );
    // Prefer exact empty-state titles; fall back to body text for diagnostics.
    const body = document.body.textContent ?? "";
    expect(body).toContain("No SSH keys");
    const add = screen.getAllByRole("button", {
      name: /Add SSH key/i,
    })[0]!;
    expect(add).toBeInTheDocument();
    expect(add).not.toBeDisabled();
  }, 30_000);

  it("unverified: list visible with Add disabled + Verify your email to add keys.", async () => {
    loaderData = {
      kind: "ready",
      user: { ...verifiedUser, email_verified: false },
      keys: [],
      gpgKeys: [],
    };
    meMock.mockResolvedValue({
      ok: true,
      data: { ...verifiedUser, email_verified: false },
    });

    const mod = await loadSshKeysModule();
    const SshKeysPage = (mod.SshKeysPage ?? mod.default) as unknown;
    const { container } = renderWithQueryClient(SshKeysPage);

    await waitFor(
      () => {
        expect(container.querySelector("h1")?.textContent).toBe("SSH and GPG keys");
      },
      { timeout: 20_000 },
    );

    const add = screen.getAllByRole("button", {
      name: /Add SSH key/i,
    })[0]!;
    expect(add).toBeDisabled();
    expect(screen.getAllByText("Verify your email to add keys.").length).toBeGreaterThan(0);
    await waitFor(() => {
      expect(screen.getByText("No SSH keys")).toBeInTheDocument();
    });
  }, 30_000);

  it("settings secondary nav General | Profile | Personal access tokens | SSH and GPG keys", async () => {
    const mod = await loadSshKeysModule();
    const SshKeysPage = (mod.SshKeysPage ?? mod.default) as unknown;
    const { container } = renderWithQueryClient(SshKeysPage);

    await waitFor(() => {
      expect(container.querySelector('nav[aria-label="Account settings"]')).toBeTruthy();
    });

    const nav = container.querySelector('nav[aria-label="Account settings"]')!;
    const general = nav.querySelector('a[href="/settings/general"]');
    const profile = nav.querySelector('a[href="/settings/profile"]');
    const tokens = nav.querySelector('a[href="/settings/tokens"]');
    const sshKeys = nav.querySelector('a[href="/settings/ssh-keys"]');
    expect(general?.textContent).toBe("General");
    expect(profile?.textContent).toBe("Profile");
    expect(tokens?.textContent).toBe("Personal access tokens");
    expect(sshKeys?.textContent).toBe("SSH and GPG keys");
    expect(sshKeys?.getAttribute("aria-current")).toBe("page");
  }, 15_000);

  it("list rows show SHA256 fingerprint", async () => {
    listMock.mockResolvedValue({
      ok: true,
      data: [
        {
          id: "key-1",
          title: "laptop",
          fingerprint: "SHA256:nThbg6kXUpJWGl7E1IGOCspRomTxdCARLviKw6E5SY8",
          key_type: "ssh-ed25519",
          can_authenticate: true,
          can_sign: true,
          last_used_at: null,
          created_at: "2026-01-01T00:00:00Z",
        },
      ],
    });

    const mod = await loadSshKeysModule();
    const SshKeysPage = (mod.SshKeysPage ?? mod.default) as unknown;
    renderWithQueryClient(SshKeysPage);

    await waitFor(() => {
      expect(screen.getByText("laptop")).toBeInTheDocument();
    });
    expect(
      screen.getByText("SHA256:nThbg6kXUpJWGl7E1IGOCspRomTxdCARLviKw6E5SY8"),
    ).toBeInTheDocument();
  }, 15_000);

  it("GPG empty hero + Commit signing setup + Add GPG key", async () => {
    const mod = await loadSshKeysModule();
    const SshKeysPage = (mod.SshKeysPage ?? mod.default) as unknown;
    const { container } = renderWithQueryClient(SshKeysPage);

    await waitFor(() => {
      expect(container.querySelector("h1")?.textContent).toBe("SSH and GPG keys");
    });
    expect(screen.getByText("No GPG keys")).toBeInTheDocument();
    expect(screen.getByText("Commit signing setup")).toBeInTheDocument();
    expect(screen.getByText("SSH signing")).toBeInTheDocument();
    expect(screen.getByText("GPG signing")).toBeInTheDocument();
    expect(document.body.textContent).toMatch(/gpg\.format ssh/);
    const addGpg = screen.getAllByRole("button", { name: /Add GPG key/i })[0]!;
    expect(addGpg).toBeInTheDocument();
    expect(addGpg).not.toBeDisabled();
  }, 15_000);

  it("GPG list rows show fingerprint and key id", async () => {
    gpgListMock.mockResolvedValue({
      ok: true,
      data: [
        {
          id: "gpg-1",
          title: "laptop-gpg",
          fingerprint: "ABCD1234EFGH5678",
          key_id: "EFGH5678",
          uid_emails: ["ada@example.com"],
          created_at: "2026-01-01T00:00:00Z",
        },
      ],
    });

    const mod = await loadSshKeysModule();
    const SshKeysPage = (mod.SshKeysPage ?? mod.default) as unknown;
    renderWithQueryClient(SshKeysPage);

    await waitFor(() => {
      expect(screen.getByText("laptop-gpg")).toBeInTheDocument();
    });
    expect(screen.getByText("ABCD1234EFGH5678")).toBeInTheDocument();
    expect(screen.getByText("EFGH5678")).toBeInTheDocument();
    expect(screen.getByText("ada@example.com")).toBeInTheDocument();
  }, 15_000);
});

describe("/settings/ssh-keys (GIT-04 / D-SSH-05 revoke)", () => {
  it("revoke AlertDialog copy Revoke SSH key? / Keep key / Revoke key", async () => {
    listMock.mockResolvedValue({
      ok: true,
      data: [
        {
          id: "key-1",
          title: "laptop",
          fingerprint: "SHA256:nThbg6kXUpJWGl7E1IGOCspRomTxdCARLviKw6E5SY8",
          key_type: "ssh-ed25519",
          can_authenticate: true,
          can_sign: true,
          last_used_at: null,
          created_at: "2026-01-01T00:00:00Z",
        },
      ],
    });

    const mod = await loadSshKeysModule();
    const SshKeysPage = (mod.SshKeysPage ?? mod.default) as unknown;
    renderWithQueryClient(SshKeysPage);

    await waitFor(() => {
      expect(screen.getByText("laptop")).toBeInTheDocument();
    });

    const revokeTriggers = screen.getAllByRole("button", {
      name: /Revoke|Delete/i,
    });
    revokeTriggers[0]!.click();

    await waitFor(() => {
      expect(screen.getByText("Revoke SSH key?")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Keep key" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Revoke key" })).toBeInTheDocument();
    expect(document.body.textContent).not.toMatch(/\bCancel\b/);
  }, 15_000);
});
