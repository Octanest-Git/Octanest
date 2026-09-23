import { cleanup, screen, waitFor } from "@octanejs/testing-library";
import { afterEach, describe, expect, it } from "@octanest/web/test-runner";
import { vi } from "vitest";
import { renderWithQueryClient } from "@/test/render-with-query";
import { AvatarCropDialog } from "./avatar-crop-dialog";
import { ProfileAvatarField } from "./profile-avatar-field";

afterEach(cleanup);

const user = {
  display_name: "Ada",
  username: "ada",
  avatar_url: null as string | null,
};

/** 1×1 PNG */
const TINY_PNG =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

describe("AvatarCropDialog", () => {
  it("renders crop chrome when open with an image URL (happy)", async () => {
    renderWithQueryClient(AvatarCropDialog, {
      props: {
        open: true,
        imageUrl: TINY_PNG,
        onConfirm: vi.fn(),
        onOpenChange: vi.fn(),
      },
    });

    await waitFor(() => {
      expect(screen.getByText("Crop profile picture")).toBeInTheDocument();
    });
    expect(screen.getByText(/Drag to reposition/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Save picture/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Cancel/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/Zoom/i)).toBeInTheDocument();
  });

  it("does not show crop chrome when closed (unhappy / idle)", () => {
    renderWithQueryClient(AvatarCropDialog, {
      props: {
        open: false,
        imageUrl: null,
        onConfirm: vi.fn(),
        onOpenChange: vi.fn(),
      },
    });

    expect(screen.queryByText("Crop profile picture")).not.toBeInTheDocument();
  });
});

describe("ProfileAvatarField", () => {
  it("shows upload affordance and hides remove when no picture (happy empty)", async () => {
    renderWithQueryClient(ProfileAvatarField, {
      props: {
        user,
        src: null,
        onCroppedFile: vi.fn(),
        onRemove: vi.fn(),
        onReject: vi.fn(),
      },
    });

    await waitFor(() => {
      expect(screen.getByText("Profile picture")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: /Upload new picture/i })).toBeInTheDocument();
    // Remove stays mounted (invisible) so empty↔filled does not shift layout.
    const remove = screen.getByText("Remove picture").closest("button");
    expect(remove).toBeTruthy();
    expect(remove).toHaveAttribute("aria-hidden", "true");
    expect(remove?.className).toMatch(/invisible/);
    expect(document.querySelector("#profile-avatar")).toBeTruthy();
  });

  it("shows remove when a picture is present (happy with avatar)", async () => {
    const onRemove = vi.fn();
    renderWithQueryClient(ProfileAvatarField, {
      props: {
        user: { ...user, avatar_url: "/uploads/avatars/u1.webp" },
        src: "/uploads/avatars/u1.webp",
        onCroppedFile: vi.fn(),
        onRemove,
        onReject: vi.fn(),
      },
    });

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Remove picture/i })).toBeInTheDocument();
    });
    screen.getByRole("button", { name: /Remove picture/i }).click();
    expect(onRemove).toHaveBeenCalledTimes(1);
  });

  it("does not open the file picker when the Profile picture label is clicked (#5)", async () => {
    const clickSpy = vi.spyOn(HTMLInputElement.prototype, "click").mockImplementation(() => {});

    renderWithQueryClient(ProfileAvatarField, {
      props: {
        user,
        src: null,
        onCroppedFile: vi.fn(),
        onRemove: vi.fn(),
        onReject: vi.fn(),
      },
    });

    await waitFor(() => {
      expect(screen.getByText("Profile picture")).toBeInTheDocument();
    });

    const label = screen.getByText("Profile picture");
    expect(label).not.toHaveAttribute("for");

    label.click();
    expect(clickSpy).not.toHaveBeenCalled();

    screen.getByRole("button", { name: /Upload new picture/i }).click();
    expect(clickSpy).toHaveBeenCalled();

    clickSpy.mockRestore();
  });
});
