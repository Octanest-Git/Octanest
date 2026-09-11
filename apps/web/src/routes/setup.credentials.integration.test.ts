import { describe, expect, it } from "vitest";

describe("/setup/credentials Wave 0 (AUTH-06 UI-SPEC)", () => {
  it("Keep current password Switch + rejects default system-administrator username copy", () => {
    // Route module `setup.credentials` lands in 06-06 — do not static-import here
    // (Vite fails the suite at transform time when the file is absent).
    const uiSpec = {
      keepPasswordLabel: "Keep current password",
      usernameError:
        "Choose a username other than the default system-administrator.",
      routeModule: "apps/web/src/routes/setup.credentials.tsrx",
    };

    expect(uiSpec.keepPasswordLabel).toBe("Keep current password");
    expect(uiSpec.usernameError).toMatch(/system-administrator/);

    // Intentional RED until CredentialsPage + Switch ship.
    expect(
      false,
      `${uiSpec.routeModule} must export CredentialsPage with Keep current password Switch (UI-SPEC)`,
    ).toBe(true);
  });
});
