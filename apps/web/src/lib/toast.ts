import { createToastManager } from "@octanejs/base-ui/toast";

/** Module-scope manager so any client code can enqueue toasts. */
export const appToastManager = createToastManager();

export function toastSuccess(title: string, description?: string): string {
  return appToastManager.add({
    title,
    description,
    type: "success",
    timeout: 4000,
  });
}

export function toastError(title: string, description?: string): string {
  return appToastManager.add({
    title,
    description,
    type: "error",
    timeout: 6000,
    priority: "high",
  });
}
