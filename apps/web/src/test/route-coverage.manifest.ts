/**
 * Route coverage manifest (G-11.1-15 / Phase 11.1-08).
 *
 * Every user-facing apps/web src/routes page (.tsrx) must appear here with at least
 * one of: happy-dom render test, stack-browser suite, or documented skip.
 * Outlet-only layouts and `__root` are marked `layoutOnly` and excluded from
 * the required set by `scripts/route-coverage-check.sh`.
 *
 * Paths are repo-relative from the Octanest root.
 */

export type RouteCoverageKind = "happy-dom" | "stack-browser" | "skip";

export type RouteCoverageEvidence =
  | { kind: "happy-dom"; test: string }
  | { kind: "stack-browser"; test: string }
  | { kind: "skip"; rationale: string };

export type RouteCoverageEntry = {
  /** Path relative to `apps/web/src/routes/` */
  route: string;
  /** When true, excluded from the required coverage set (shell / Outlet-only). */
  layoutOnly?: boolean;
  coverage: RouteCoverageEvidence[];
};

const AUTH_UI = "apps/web/e2e/stack-browser/auth-ui.stack.browser.test.tsx";
const FORGE_REPO = "apps/web/e2e/stack-browser/forge-repo.stack.browser.test.tsx";
const FORGE_ISSUES =
  "apps/web/e2e/stack-browser/forge-issues-releases.stack.browser.test.tsx";
const FORGE_SSH_ORGS =
  "apps/web/e2e/stack-browser/forge-packages-ssh-orgs.stack.browser.test.tsx";
const FORGE_ADMIN =
  "apps/web/e2e/stack-browser/forge-admin.stack.browser.test.tsx";

export const routeCoverageManifest: RouteCoverageEntry[] = [
  // --- shells / Outlet-only layouts (excluded from required set) ---
  { route: "__root.tsrx", layoutOnly: true, coverage: [] },
  { route: "$owner.tsrx", layoutOnly: true, coverage: [] },
  { route: "$owner.$repo.issues.tsrx", layoutOnly: true, coverage: [] },
  { route: "$owner.$repo.releases.tsrx", layoutOnly: true, coverage: [] },
  { route: "setup.tsrx", layoutOnly: true, coverage: [] },

  // --- auth + marketing (11.1-04) ---
  {
    route: "login.tsrx",
    coverage: [
      { kind: "happy-dom", test: "apps/web/src/routes/login.integration.test.ts" },
      { kind: "stack-browser", test: AUTH_UI },
    ],
  },
  {
    route: "signup.tsrx",
    coverage: [
      { kind: "happy-dom", test: "apps/web/src/routes/signup.integration.test.ts" },
      { kind: "stack-browser", test: AUTH_UI },
    ],
  },
  {
    route: "verify.tsrx",
    coverage: [
      { kind: "happy-dom", test: "apps/web/src/routes/verify.integration.test.ts" },
    ],
  },
  {
    route: "reset-password.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/reset-password.integration.test.ts",
      },
    ],
  },
  {
    route: "status.tsrx",
    coverage: [
      { kind: "happy-dom", test: "apps/web/src/routes/status.integration.test.ts" },
      { kind: "stack-browser", test: AUTH_UI },
    ],
  },
  {
    route: "index.tsrx",
    coverage: [
      { kind: "stack-browser", test: AUTH_UI },
      {
        kind: "skip",
        rationale:
          "happy-dom covers tree-gate helpers + SignedInHome module, not full page mount; browser covers signed-in home via auth.me dedupe",
      },
    ],
  },

  // --- setup wizard ---
  {
    route: "setup.index.tsrx",
    coverage: [
      { kind: "happy-dom", test: "apps/web/src/routes/setup.integration.test.ts" },
    ],
  },
  {
    route: "setup.credentials.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/setup.credentials.integration.test.ts",
      },
    ],
  },

  // --- user settings ---
  {
    route: "settings/profile.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/settings/profile.integration.test.ts",
      },
    ],
  },
  {
    route: "settings/ssh-keys.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/settings/ssh-keys.integration.test.ts",
      },
      { kind: "stack-browser", test: FORGE_SSH_ORGS },
    ],
  },
  {
    route: "settings/tokens.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/settings/tokens.integration.test.ts",
      },
    ],
  },
  {
    route: "settings/tokens.new.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Create-token form covered transitively by tokens.integration.test.ts PAT flows; dedicated mount deferred",
      },
    ],
  },
  {
    route: "settings/tokens.new.fine-grained.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Fine-grained token wizard deferred; classic tokens happy-dom + packages scope tests cover PAT surface",
      },
    ],
  },

  // --- create flows ---
  {
    route: "new.tsrx",
    coverage: [
      { kind: "happy-dom", test: "apps/web/src/routes/new.integration.test.ts" },
    ],
  },
  {
    route: "orgs.new.tsrx",
    coverage: [
      { kind: "happy-dom", test: "apps/web/src/routes/orgs.new.integration.test.ts" },
    ],
  },
  {
    route: "dashboard.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Hard-404 beforeLoad only (notFound); no UI to mount — covered by dashboard.integration.test.ts contract",
      },
    ],
  },
  {
    route: "invites.$token.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Invite accept page not yet in forge stack-browser matrix; follow-up with org invite e2e",
      },
    ],
  },

  // --- admin (G-11.1-15 / 11.1-07) ---
  {
    route: "admin/lfs.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/admin/lfs.integration.test.ts",
      },
      { kind: "stack-browser", test: FORGE_ADMIN },
    ],
  },
  {
    route: "admin/packages.tsrx",
    coverage: [
      { kind: "stack-browser", test: FORGE_ADMIN },
      {
        kind: "skip",
        rationale:
          "happy-dom still export/hint smoke only (not a render mount); stack-browser visits /admin/packages — upgrade packages.integration.test.ts to mount when touching quotas UI",
      },
    ],
  },
  {
    route: "admin/auth.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/admin/auth.integration.test.ts",
      },
    ],
  },

  // --- forge repo chrome + code browse (11.1-04) ---
  {
    route: "$owner.$repo.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/$owner.$repo.integration.test.ts",
      },
      { kind: "stack-browser", test: FORGE_REPO },
    ],
  },
  {
    route: "$owner.$repo.index.tsrx",
    coverage: [
      { kind: "stack-browser", test: FORGE_REPO },
      {
        kind: "skip",
        rationale:
          "Code home exercised via forge-repo stack-browser + layout chrome happy-dom; dedicated index mount deferred",
      },
    ],
  },
  {
    route: "$owner.$repo.packages.tsrx",
    coverage: [{ kind: "stack-browser", test: FORGE_REPO }],
  },
  {
    route: "$owner.$repo.tree.$.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/$owner.$repo.integration.test.ts",
      },
    ],
  },
  {
    route: "$owner.$repo.blob.$.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/$owner.$repo.integration.test.ts",
      },
    ],
  },
  {
    route: "$owner.$repo.blame.$.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Blame view deferred behind blob/tree happy-dom; add mount when blame UX changes",
      },
    ],
  },
  {
    route: "$owner.$repo.commits.$.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale: "Commits list not in D-QH-03 matrix; deferred stack-browser",
      },
    ],
  },
  {
    route: "$owner.$repo.commit.$sha.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale: "Commit detail not in D-QH-03 matrix; deferred stack-browser",
      },
    ],
  },
  {
    route: "$owner.$repo.compare.$.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale: "Compare view deferred; no happy-dom mount yet",
      },
    ],
  },
  {
    route: "$owner.$repo.branches.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale: "Branches list deferred behind refs chrome on code home",
      },
    ],
  },
  {
    route: "$owner.$repo.tags.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale: "Tags list deferred; release create covers tag selection path",
      },
    ],
  },
  {
    route: "$owner.$repo.settings.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Repo settings panels covered by Wave 0 raw-source suites (collaborators/lfs/rename); render mount + browser deferred",
      },
    ],
  },

  // --- issues ---
  {
    route: "$owner.$repo.issues.index.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/$owner.$repo.issues.integration.test.ts",
      },
    ],
  },
  {
    route: "$owner.$repo.issues.new.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/$owner.$repo.issues.integration.test.ts",
      },
      { kind: "stack-browser", test: FORGE_ISSUES },
    ],
  },
  {
    route: "$owner.$repo.issues.$n.tsrx",
    coverage: [
      {
        kind: "happy-dom",
        test: "apps/web/src/routes/$owner.$repo.issues.integration.test.ts",
      },
      { kind: "stack-browser", test: FORGE_ISSUES },
    ],
  },
  {
    route: "$owner.$repo.issues.labels.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Repo labels settings UI deferred; issue label attach covered in issues happy-dom",
      },
    ],
  },

  // --- releases ---
  {
    route: "$owner.$repo.releases.index.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Releases list covered transitively by create→detail stack-browser; dedicated list mount deferred",
      },
    ],
  },
  {
    route: "$owner.$repo.releases.new.tsrx",
    coverage: [{ kind: "stack-browser", test: FORGE_ISSUES }],
  },
  {
    route: "$owner.$repo.releases.$tag.tsrx",
    coverage: [{ kind: "stack-browser", test: FORGE_ISSUES }],
  },

  // --- owner / org ---
  {
    route: "$owner.index.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Org overview not in D-QH-03 matrix; members settings browser covers org admin path",
      },
    ],
  },
  {
    route: "$owner.packages.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale:
          "Owner packages list has export/raw-source suite only; stack-browser covers repo-linked packages",
      },
    ],
  },
  {
    route: "$owner.settings.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale: "Org settings hub deferred; members child has stack-browser",
      },
    ],
  },
  {
    route: "$owner.settings.members.tsrx",
    coverage: [{ kind: "stack-browser", test: FORGE_SSH_ORGS }],
  },
  {
    route: "$owner.settings.labels.tsrx",
    coverage: [
      {
        kind: "skip",
        rationale: "Org-wide labels settings deferred behind repo issue labels",
      },
    ],
  },
];
