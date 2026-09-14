import { Toast } from "@octanejs/base-ui/toast";
import { appToastManager } from "@/lib/toast";
import { cn } from "@/lib/utils";

/**
 * Octanest toast chrome — Base UI Toast with forge styling.
 * Mount once under {@link AppToaster} (Provider + viewport).
 */
function ToastList() @{
  const { toasts } = Toast.useToastManager();

  <Toast.Portal>
    <Toast.Viewport
      className={cn(
        "fixed inset-x-0 top-3 z-[100] mx-auto flex w-[min(100%-1.5rem,24rem)] flex-col gap-2 outline-none",
        "md:top-4",
      )}
    >
      @for (const toast of toasts; key toast.id) {
        <Toast.Root
          toast={toast}
          className={cn(
            "oct-toast group relative flex w-full origin-top flex-col rounded-lg border bg-card/95 p-3 text-foreground shadow-lg backdrop-blur-sm",
            "outline-none transition-[transform,opacity] duration-200 ease-out",
            "data-[starting-style]:translate-y-[-6px] data-[starting-style]:opacity-0",
            "data-[ending-style]:translate-y-[-4px] data-[ending-style]:opacity-0",
            "data-[type=success]:border-primary/40",
            "data-[type=error]:border-destructive/50",
            "border-border",
          )}
        >
          <Toast.Content className="flex items-start gap-3">
            <span
              aria-hidden="true"
              className={cn(
                "mt-0.5 w-1 shrink-0 self-stretch rounded-full bg-muted-foreground/40",
                "group-data-[type=success]:bg-primary",
                "group-data-[type=error]:bg-destructive",
              )}
            />
            <div className="min-w-0 flex-1">
              <Toast.Title className="text-[14px] font-medium leading-snug text-foreground" />
              <Toast.Description className="mt-0.5 text-[13px] font-normal leading-snug text-muted-foreground" />
            </div>
            <Toast.Close
              className={cn(
                "inline-flex h-8 shrink-0 items-center rounded-md px-2 text-[13px] font-medium text-muted-foreground",
                "hover:bg-muted hover:text-foreground",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              )}
              aria-label="Dismiss"
            >
              Dismiss
            </Toast.Close>
          </Toast.Content>
        </Toast.Root>
      }
    </Toast.Viewport>
  </Toast.Portal>
}

/** Root provider + viewport. Wrap the app once in `__root`. */
export function AppToaster({ children }: { children?: unknown }) @{
  <Toast.Provider toastManager={appToastManager} timeout={5000} limit={3}>
    {children}
    <ToastList />
  </Toast.Provider>
}
