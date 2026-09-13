import { Check, ChevronDown, Code, Copy } from "@octanejs/lucide";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { PatHowTo } from "@/components/repo/pat-how-to";
import { httpsCloneUrl } from "@/lib/public-origin";
import { useRepoStore } from "@/lib/repo-store";

type CloneBoxProps = {
  owner: string;
  repo: string;
  /** Current treeish used for archive download links. */
  refName: string;
  /** When true, ZIP / tar.gz items are disabled (GIT-07 empty edge). */
  empty: boolean;
  /** Optional override for tests; otherwise SSR layout store origin. */
  publicOrigin?: string;
};

/** Code clone / download dropdown (D-22 / D-29 / E12). */
export function CloneBox({
  owner,
  repo,
  refName,
  empty,
  publicOrigin: publicOriginProp,
}: CloneBoxProps) @{
  const storeOrigin = useRepoStore((s) => s.publicOrigin);
  const httpsUrl = httpsCloneUrl(
    publicOriginProp || storeOrigin,
    owner,
    repo,
  );
  const zipHref = `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/archive/${encodeURIComponent(refName)}.zip`;
  const tarHref = `/api/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/archive/${encodeURIComponent(refName)}.tar.gz`;
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">(
    "idle",
  );

  async function copyHttps() {
    try {
      if (!navigator.clipboard?.writeText) {
        throw new Error("clipboard unavailable");
      }
      await navigator.clipboard.writeText(httpsUrl);
      setCopyState("copied");
      window.setTimeout(() => setCopyState("idle"), 2000);
    } catch {
      setCopyState("error");
    }
  }

  <DropdownMenu>
    <DropdownMenuTrigger
      className="inline-flex h-11 items-center gap-2 rounded-md border border-border bg-card px-4 text-[14px] font-semibold text-foreground outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring"
      aria-label="Clone or download"
    >
      <Code aria-hidden size={16} className="text-foreground" />
      Code
      <ChevronDown aria-hidden size={16} className="text-muted-foreground" />
    </DropdownMenuTrigger>
    <DropdownMenuContent
      align="end"
      className="w-[min(100vw-2rem,32rem)] p-3"
    >
      <div className="px-1 py-1">
        <p className="text-[14px] font-medium text-foreground">Clone with HTTPS</p>
        <p className="mt-0.5 text-[13px] text-muted-foreground">
          Use this URL from any Git client.
        </p>
        <div className="mt-3 flex items-stretch gap-2">
          <input
            type="text"
            readOnly
            value={httpsUrl as string}
            aria-label="HTTPS clone URL"
            title={httpsUrl as string}
            onFocus={(e) => {
              const el = e.currentTarget;
              el.select();
              // select() can scroll long URLs so the scheme disappears — pin start.
              requestAnimationFrame(() => {
                el.scrollLeft = 0;
              });
            }}
            className="min-w-0 flex-1 rounded-md border border-border bg-muted/40 px-2.5 py-2 font-mono text-[13px] leading-none text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-auto shrink-0 border border-border hover:bg-muted"
            aria-label="Copy HTTPS URL"
            title={(copyState === "copied" ? "Copied" : "Copy HTTPS URL") as string}
            onClick={() => void copyHttps()}
          >
            <Copy
              aria-hidden
              size={16}
              className={(copyState === "copied"
                ? "hidden"
                : "text-foreground") as string}
            />
            <Check
              aria-hidden
              size={16}
              className={(copyState === "copied"
                ? "text-foreground"
                : "hidden") as string}
            />
          </Button>
        </div>
        @if (copyState === "error") {
          <p role="alert" className="mt-2 text-[14px] text-destructive">
            Couldn’t copy. Select the URL and copy manually.
          </p>
        }

        <PatHowTo httpsUrl={httpsUrl as string} compact />
      </div>

      <DropdownMenuSeparator />

      <p className="px-1 py-2 text-[13px] text-muted-foreground">
        SSH cloning arrives in a later phase.
      </p>

      <DropdownMenuSeparator />

      @if (empty) {
        <>
          <DropdownMenuItem disabled aria-disabled="true">
            Download ZIP
          </DropdownMenuItem>
          <DropdownMenuItem disabled aria-disabled="true">
            Download tar.gz
          </DropdownMenuItem>
        </>
      } @else {
        <>
          <DropdownMenuItem
            onClick={() => {
              window.location.assign(zipHref);
            }}
          >
            Download ZIP
          </DropdownMenuItem>
          <DropdownMenuItem
            onClick={() => {
              window.location.assign(tarHref);
            }}
          >
            Download tar.gz
          </DropdownMenuItem>
        </>
      }
    </DropdownMenuContent>
  </DropdownMenu>
}
