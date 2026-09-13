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

type CloneBoxProps = {
  owner: string;
  repo: string;
  /** Current treeish used for archive download links. */
  refName: string;
  /** When true, ZIP / tar.gz items are disabled (GIT-07 empty edge). */
  empty: boolean;
};

/** Code clone / download dropdown (D-22 / D-29 / E12). */
export function CloneBox({ owner, repo, refName, empty }: CloneBoxProps) @{
  const origin =
    typeof window !== "undefined" ? window.location.origin : "";
  const httpsUrl = `${origin}/${owner}/${repo}.git`;
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
    <DropdownMenuContent align="end" className="w-80 p-3">
      <div className="px-1 py-1">
        <p className="text-[14px] font-medium text-foreground">Clone with HTTPS</p>
        <p className="mt-0.5 text-[13px] text-muted-foreground">
          Use this URL from any Git client.
        </p>
        <div className="mt-3 flex items-stretch gap-2">
          <code
            className="block min-w-0 flex-1 truncate rounded-md border border-border bg-muted/40 px-2.5 py-2 font-mono text-[13px] leading-none text-foreground"
            title={httpsUrl as string}
          >
            {httpsUrl as string}
          </code>
          <Button
            type="button"
            variant="secondary"
            size="icon"
            className="shrink-0"
            aria-label="Copy HTTPS URL"
            title={(copyState === "copied" ? "Copied" : "Copy HTTPS URL") as string}
            onClick={() => void copyHttps()}
          >
            <Copy
              aria-hidden
              size={16}
              className={(copyState === "copied"
                ? "hidden"
                : "text-secondary") as string}
            />
            <Check
              aria-hidden
              size={16}
              className={(copyState === "copied"
                ? "text-secondary"
                : "hidden") as string}
            />
          </Button>
        </div>
        @if (copyState === "error") {
          <p role="alert" className="mt-2 text-[14px] text-destructive">
            Couldn’t copy. Select the URL and copy manually.
          </p>
        }
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
