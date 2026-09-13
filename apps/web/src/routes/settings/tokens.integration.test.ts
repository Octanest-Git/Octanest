import { createElement } from "octane";
import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";

/**
 * GIT-11 /settings/tokens list / create / revoke UI
 * (D-14, D-15, D-17, D-24 / T-08-01 / T-08-03).
 *
 * List + nav greened in 08-09-T1; revoke dialog in 08-09-T2; create/reveal in 08-10/08-11.
 */

const listMock = vi.fn();
const revokeMock = vi.fn();
const meMock = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      me: (...args: unknown[]) => meMock(...args),
    },
    pat: {
      list: (...args: unknown[]) => listMock(...args),
      revoke: (...args: unknown[]) => revokeMock(...args),
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
    };

let loaderData: LoaderShape;

vi.mock("@octanejs/tanstack-router", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@octanejs/tanstack-router")>();
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
  revokeMock.mockReset();
  meMock.mockReset();
  loaderData = { kind: "ready", user: verifiedUser };
  listMock.mockResolvedValue({ ok: true, data: [] });
  meMock.mockResolvedValue({ ok: true, data: verifiedUser });
});

afterEach(cleanup);

/** Load tokens page; @vite-ignore keeps the suite collectable before ./tokens exists. */
async function loadTokensModule(): Promise<Record<string, unknown>> {
  const rel = "./tokens";
  try {
    return (await import(/* @vite-ignore */ rel)) as Record<string, unknown>;
  } catch (err) {
    throw new Error(
      `Wave 0: /settings/tokens route missing — implement in 08-09 (GIT-11 / D-14). Expected list title Personal access tokens. ${(err as Error).message}`,
    );
  }
}

describe("/settings/tokens (GIT-11 / D-14 list)", () => {
  it(
    "list title Personal access tokens + empty hero No personal access tokens + Generate new token",
    async () => {
      const mod = await loadTokensModule();
      const TokensPage = (mod.TokensPage ?? mod.default) as unknown;
      const { container } = renderWithQueryClient(TokensPage);

      await waitFor(() => {
        expect(
          container.querySelector("h1")?.textContent,
        ).toBe("Personal access tokens");
      });

      await waitFor(() => {
        expect(
          screen.getByText("No personal access tokens"),
        ).toBeInTheDocument();
      });
      expect(
        screen.getByText("Create a token to clone, fetch, and push over HTTPS."),
      ).toBeInTheDocument();
      const generate = screen.getAllByRole("button", {
        name: /Generate new token/i,
      })[0]!;
      expect(generate).toBeInTheDocument();
      expect(generate).not.toBeDisabled();
      generate.click();
      await waitFor(() => {
        expect(screen.getByText("Classic token")).toBeInTheDocument();
      });
      expect(screen.getByText("Fine-grained token")).toBeInTheDocument();
      // T-08-01: no plaintext secrets on list
      expect(container.textContent).not.toMatch(
        /octanest_pat_[a-f0-9]{16,}|octanest_fg_[a-f0-9]{16,}/i,
      );
    },
    15_000,
  );

  it(
    "unverified: list visible with Generate disabled + Verify your email to create a token.",
    async () => {
      loaderData = {
        kind: "ready",
        user: { ...verifiedUser, email_verified: false },
      };
      meMock.mockResolvedValue({
        ok: true,
        data: { ...verifiedUser, email_verified: false },
      });

      const mod = await loadTokensModule();
      const TokensPage = (mod.TokensPage ?? mod.default) as unknown;
      const { container } = renderWithQueryClient(TokensPage);

      await waitFor(() => {
        expect(
          container.querySelector("h1")?.textContent,
        ).toBe("Personal access tokens");
      });

      const generate = screen.getAllByRole("button", {
        name: /Generate new token/i,
      })[0]!;
      expect(generate).toBeDisabled();
      expect(
        screen.getByText("Verify your email to create a token."),
      ).toBeInTheDocument();
      await waitFor(() => {
        expect(
          screen.getByText("No personal access tokens"),
        ).toBeInTheDocument();
      });
    },
    15_000,
  );

  it(
    "settings secondary nav Profile | Personal access tokens",
    async () => {
      const mod = await loadTokensModule();
      const TokensPage = (mod.TokensPage ?? mod.default) as unknown;
      const { container } = renderWithQueryClient(TokensPage);

      await waitFor(() => {
        expect(
          container.querySelector('nav[aria-label="Settings"]'),
        ).toBeTruthy();
      });

      const nav = container.querySelector('nav[aria-label="Settings"]')!;
      const profile = nav.querySelector('a[href="/settings/profile"]');
      const tokens = nav.querySelector('a[href="/settings/tokens"]');
      expect(profile?.textContent).toBe("Profile");
      expect(tokens?.textContent).toBe("Personal access tokens");
      expect(tokens?.getAttribute("aria-current")).toBe("page");
    },
    15_000,
  );
});

describe("/settings/tokens (GIT-11 / D-17 revoke)", () => {
  it("revoke AlertDialog copy Revoke token? / Keep token", async () => {
    // Expanded assertions land with pat-revoke-dialog in 08-09-T2.
    const mod = await loadTokensModule();
    expect(
      mod.TokensPage ?? mod.default,
      "Wave 0: revoke dialog must use Revoke token? / Keep token (not Cancel)",
    ).toBeTruthy();
  });
});

describe("/settings/tokens (GIT-11 / D-15 one-time reveal)", () => {
  // Deferred to 08-10 — classic create + reveal route.
  it.skip("one-time reveal Make sure to copy your personal access token now", async () => {
    const rel = "./tokens.new";
    const mod = (await import(/* @vite-ignore */ rel)) as Record<
      string,
      unknown
    >;
    expect(
      mod.TokensNewPage ?? mod.ClassicCreatePage ?? mod.default,
      "Wave 0: create reveal must show Make sure to copy your personal access token now",
    ).toBeTruthy();
  });
});
