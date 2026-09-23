import { describe, expect, it } from "@octanest/web/test-runner";
import { createRepoStore } from "./repo-store";

describe("createRepoStore (@octanejs/zustand)", () => {
  it("seeds from SSR loader payload and setRepo updates", () => {
    const store = createRepoStore({
      owner: "ada",
      repoName: "hello",
      status: "ok",
      repo: {
        id: "r1",
        owner_id: "u1",
        owner_type: "user",
        owner_username: "ada",
        name: "hello",
        description: "",
        visibility: "private",
        default_branch: "main",
        updated_at: "2026-09-13T00:00:00Z",
      },
      me: {
        id: "u1",
        email: "ada@example.com",
        username: "ada",
        display_name: "Ada",
        bio: "",
        role: "user",
        profile_incomplete: false,
        email_verified: true,
        must_change_credentials: false,
      },
      publicOrigin: "http://localhost",
      sshHost: "localhost",
      sshPort: 22,
    });

    expect(store.getState().publicOrigin).toBe("http://localhost");
    expect(store.getState().sshHost).toBe("localhost");
    expect(store.getState().sshPort).toBe(22);
    expect(store.getState().repo?.visibility).toBe("private");
    store.getState().setRepo({
      ...store.getState().repo!,
      visibility: "public",
    });
    expect(store.getState().repo?.visibility).toBe("public");
    expect(store.getState().status).toBe("ok");
  });
});
