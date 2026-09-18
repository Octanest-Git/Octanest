import type { OctanestClient, RepoPublic, RepoTreeEntry } from "@octanest/api-client";
import { findReadmeName, joinRepoPath } from "@/lib/repo-browse";
import { renderGfm } from "@/lib/markdown";

/** Sanitized profile README ready for `ReadmePanel`. */
export type ProfileReadme = {
  name: string;
  html: string;
  sourceRepo: string;
};

/**
 * Org profile special-repo preference order.
 * Prefer Octanest’s `.octanest`, then GitHub-compatible `.github`.
 */
export const ORG_PROFILE_SPECIAL_REPOS = [".octanest", ".github"] as const;

/** Directory inside the special repo that holds the org profile README (GitHub pattern). */
export const ORG_PROFILE_README_DIR = "profile";

type SpecialRepoCandidate = {
  name: string;
  visibility: string;
};

/**
 * Pure: pick the preferred public special repo for an org profile README.
 * Returns null when none of the candidates exist as public.
 */
export function pickOrgProfileSpecialRepo(
  repos: ReadonlyArray<SpecialRepoCandidate>,
): (typeof ORG_PROFILE_SPECIAL_REPOS)[number] | null {
  for (const candidate of ORG_PROFILE_SPECIAL_REPOS) {
    const hit = repos.find((r) => r.name === candidate && r.visibility === "public");
    if (hit) return candidate;
  }
  return null;
}

function isPublicRepo(repo: RepoPublic): boolean {
  return repo.visibility === "public";
}

async function loadReadmeFromTree(
  client: OctanestClient,
  owner: string,
  repoName: string,
  ref: string,
  treePath: string,
  entries: RepoTreeEntry[],
): Promise<ProfileReadme | null> {
  const readmeName = findReadmeName(entries);
  if (!readmeName) return null;

  const blobPath = treePath ? joinRepoPath(treePath, readmeName) : readmeName;
  const blobRes = await client.repo.blob({
    owner,
    name: repoName,
    ref,
    path: blobPath,
  });
  if (
    !blobRes.ok ||
    blobRes.data.is_binary ||
    blobRes.data.encoding !== "utf-8" ||
    !blobRes.data.content
  ) {
    return null;
  }

  return {
    name: readmeName,
    html: await renderGfm(blobRes.data.content, { owner, repo: repoName }),
    sourceRepo: `${owner}/${repoName}`,
  };
}

/**
 * Load a public repo’s README at `treePath` (empty = repo root).
 * Private / missing / errors → null (never surface private profile content).
 */
async function loadPublicRepoReadme(
  client: OctanestClient,
  owner: string,
  repoName: string,
  treePath: string,
): Promise<ProfileReadme | null> {
  try {
    const got = await client.repo.get({ owner, name: repoName });
    if (!got.ok || !isPublicRepo(got.data)) return null;

    const ref = got.data.default_branch || "main";
    const treeRes = await client.repo.tree({
      owner,
      name: repoName,
      ref,
      path: treePath,
    });
    if (!treeRes.ok || treeRes.data.empty) return null;

    return loadReadmeFromTree(client, owner, repoName, ref, treePath, treeRes.data.entries);
  } catch {
    return null;
  }
}

/**
 * User profile README: public special repo `username/username` with root `README.md`
 * (GitHub user profile README pattern).
 */
export async function fetchUserProfileReadme(
  client: OctanestClient,
  username: string,
): Promise<ProfileReadme | null> {
  const owner = username.trim();
  if (!owner) return null;
  return loadPublicRepoReadme(client, owner, owner, "");
}

/**
 * Org profile README: public `.octanest` then `.github`, file under `profile/`
 * (GitHub org profile README pattern; Octanest-preferred special repo name).
 */
export async function fetchOrgProfileReadme(
  client: OctanestClient,
  orgSlug: string,
): Promise<ProfileReadme | null> {
  const owner = orgSlug.trim();
  if (!owner) return null;

  for (const special of ORG_PROFILE_SPECIAL_REPOS) {
    const readme = await loadPublicRepoReadme(client, owner, special, ORG_PROFILE_README_DIR);
    if (readme) return readme;
  }
  return null;
}
