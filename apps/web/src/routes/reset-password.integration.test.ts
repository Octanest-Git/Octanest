import { cleanup, fireEvent, render, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  SUCCESS_BODY,
  SUCCESS_HEADING,
  SSO_MODE_BODY,
  REDEEM_SUPPORT_CODE,
  REDEEM_SUPPORT_TOKEN,
} from "@/lib/reset-password-copy";

const providerConfig = vi.fn();
const requestPasswordReset = vi.fn();
const resetPassword = vi.fn();

vi.mock("@/lib/api-client", () => ({
  apiClient: {
    auth: {
      providerConfig: (...args: unknown[]) => providerConfig(...args),
      requestPasswordReset: (...args: unknown[]) => requestPasswordReset(...args),
      resetPassword: (...args: unknown[]) => resetPassword(...args),
    },
  },
}));

import { ResetPasswordPage } from "./reset-password";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  window.history.replaceState({}, "", "/reset-password");
});

beforeEach(() => {
  window.history.replaceState({}, "", "/reset-password");
});

describe("ResetPasswordPage AUTH-12 UI", () => {
  it("shows identical anti-enumeration success panel after request (never not-found)", async () => {
    providerConfig.mockResolvedValue({ ok: true, data: { mode: "local" } });
    requestPasswordReset.mockResolvedValue({ ok: true, data: { ok: true } });

    render(ResetPasswordPage);

    const email = await screen.findByLabelText("Email");
    fireEvent.input(email, { target: { value: "nobody@example.com" } });
    fireEvent.submit(email.closest("form")!);

    await waitFor(() => {
      expect(screen.getByRole("status")).toBeInTheDocument();
    });
    expect(screen.getByText(SUCCESS_HEADING)).toBeInTheDocument();
    expect(screen.getByText(SUCCESS_BODY)).toBeInTheDocument();
    expect(screen.queryByText(/not.?found/i)).not.toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(requestPasswordReset).toHaveBeenCalledWith({
      email: "nobody@example.com",
    });
  });

  it("maps unexpected request errors to network copy — never not-found", async () => {
    providerConfig.mockResolvedValue({ ok: true, data: { mode: "local" } });
    requestPasswordReset.mockResolvedValue({
      ok: false,
      error: { code: "auth.taken", message: "email not found" },
    });

    render(ResetPasswordPage);

    const email = await screen.findByLabelText("Email");
    fireEvent.input(email, { target: { value: "x@example.com" } });
    fireEvent.submit(email.closest("form")!);

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent(/Can't reach Octanest/);
    expect(alert).not.toHaveTextContent(/not found/i);
    expect(screen.queryByText(SUCCESS_HEADING)).not.toBeInTheDocument();
  });

  it("SSO mode shows IdP-only message without email form", async () => {
    providerConfig.mockResolvedValue({ ok: true, data: { mode: "workos" } });

    render(ResetPasswordPage);

    await waitFor(() => {
      expect(screen.getByText(SSO_MODE_BODY)).toBeInTheDocument();
    });
    expect(
      screen.getByRole("button", { name: "Continue with WorkOS" }),
    ).toBeInTheDocument();
    expect(screen.queryByLabelText("Email")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Send reset email" }),
    ).not.toBeInTheDocument();
  });

  it("redeem via code shows OTP + password fields", async () => {
    providerConfig.mockResolvedValue({ ok: true, data: { mode: "local" } });

    render(ResetPasswordPage);

    const enterCode = await screen.findByRole("button", {
      name: "Enter it here",
    });
    fireEvent.click(enterCode);

    await waitFor(() => {
      expect(screen.getByText(REDEEM_SUPPORT_CODE)).toBeInTheDocument();
    });
    expect(document.getElementById("reset-code")).toBeTruthy();
    expect(screen.getByLabelText("New password")).toBeInTheDocument();
    expect(screen.getByLabelText("Confirm password")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Update password" }),
    ).toBeDisabled();
  });

  it("token query hides OTP and uses token redeem support copy", async () => {
    window.history.replaceState({}, "", "/reset-password?token=abc123token");
    providerConfig.mockResolvedValue({ ok: true, data: { mode: "local" } });

    render(ResetPasswordPage);

    await waitFor(() => {
      expect(screen.getByText(REDEEM_SUPPORT_TOKEN)).toBeInTheDocument();
    });
    expect(document.getElementById("reset-code")).toBeNull();
    expect(screen.getByLabelText("New password")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Choose a new password" }),
    ).toBeInTheDocument();
  });

  it("Enter reset code from anti-enumeration panel opens redeem", async () => {
    providerConfig.mockResolvedValue({ ok: true, data: { mode: "local" } });
    requestPasswordReset.mockResolvedValue({ ok: true, data: { ok: true } });

    render(ResetPasswordPage);

    const email = await screen.findByLabelText("Email");
    fireEvent.input(email, { target: { value: "user@example.com" } });
    fireEvent.submit(email.closest("form")!);

    const enter = await screen.findByRole("button", { name: "Enter reset code" });
    fireEvent.click(enter);

    await waitFor(() => {
      expect(document.getElementById("reset-code")).toBeTruthy();
    });
    expect(screen.getByText(REDEEM_SUPPORT_CODE)).toBeInTheDocument();
  });
});
