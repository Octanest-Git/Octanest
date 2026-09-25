import { createServerFn } from "@octanejs/tanstack-start";
import { getRequestHeader } from "@octanejs/tanstack-start/server";
import {
  createClient,
  type OxideanClient,
  type PublicUserProfile,
  type RepoPublic,
} from "@oxidean/api-client";
import { fetchUserProfileReadme, type ProfileReadme } from "@/lib/profile-readme";

function ssrApiOrigin(): string {
  return (
    process.env.OXIDEAN_API_ORIGIN?.replace(/\/$/, "") ||
    process.env.OXIDEAN_E2E_API_ORIGIN?.replace(/\/$/, "") ||
    "http://127.0.0.1:8080"
  );
}

function createSsrClient(cookie: string): OxideanClient {
  return createClient({
    baseUrl: ssrApiOrigin(),
    credentials: "include",
    fetch: (input, init) => {
      const headers = new Headers(init?.headers);
      if (cookie) headers.set("cookie", cookie);
      return fetch(input, { ...init, headers });
    },
  });
}

function incomingCookie(): string {
  return getRequestHeader("cookie") ?? "";
}

export type UserProfilePayload = {
  profile: PublicUserProfile;
  repos: RepoPublic[];
  starred: RepoPublic[];
  isSelf: boolean;
  /** Public `username/username` root README, or null. */
  profileReadme: ProfileReadme | null;
};

/** SSR: public user profile + ACL-filtered repos (D-SOC-05…08). */
export const fetchUserProfile = createServerFn({ method: "GET" })
  .validator((data: { username: string }) => ({
    username: String(data?.username ?? ""),
  }))
  .handler(async ({ data }): Promise<UserProfilePayload | null> => {
    const client = createSsrClient(incomingCookie());
    const username = data.username.trim();
    if (!username) return null;

    const got = await client.user.getPublicProfile({ username });
    if (!got.ok) return null;

    let repos: RepoPublic[] = [];
    const listed = await client.repo.listByOwner({ owner: username });
    if (listed.ok) {
      repos = listed.data.repos;
    }

    let starred: RepoPublic[] = [];
    let isSelf = false;
    const me = await client.auth.me();
    if (me.ok && me.data && me.data.username === username) {
      isSelf = true;
      const stars = await client.user.listStarred({ offset: 0, limit: 30 });
      if (stars.ok) {
        starred = stars.data.repos;
      }
    }

    const profileReadme = await fetchUserProfileReadme(client, username);

    return { profile: got.data, repos, starred, isSelf, profileReadme };
  });
