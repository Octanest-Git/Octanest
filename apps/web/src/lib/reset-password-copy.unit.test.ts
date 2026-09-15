import { describe, expect, it } from "vitest";
import { SSO_MODE_BODY, SUCCESS_BODY, SUCCESS_HEADING } from "./reset-password-copy";

describe("reset-password-copy", () => {
  it("matches 05-UI-SPEC anti-enumeration and SSO strings", () => {
    expect(SUCCESS_HEADING).toBe("Check your email");
    expect(SUCCESS_BODY).toBe(
      "If an account exists for that email, we sent password reset instructions. Check your inbox and spam folder.",
    );
    expect(SSO_MODE_BODY).toBe("Password reset is managed by your identity provider.");
  });
});
