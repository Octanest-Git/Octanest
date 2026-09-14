import type { RepoLfsObjectEntry } from "@octanest/api-client";
import { useQuery } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";

function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "0 B";
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GiB`;
}

/** In-app LFS object browser for an enabled repository (D-LFS-16). */
export function LfsBrowser(props: { owner: string; name: string }) @{
  const owner = props.owner;
  const name = props.name;
  const [dlOid, setDlOid] = useState<string | null>(null);
  const [dlError, setDlError] = useState("");

  const listQ = useQuery({
    queryKey: ["repo", "lfs", "list", owner, name] as const,
    queryFn: async () => {
      const res = await apiClient.repo.lfs.listObjects({ owner, name });
      if (!res.ok) throw new Error(res.error.message || "Could not list LFS objects.");
      return res.data;
    },
  });

  async function downloadOid(oid: string) {
    setDlError("");
    setDlOid(oid);
    try {
      const res = await apiClient.repo.lfs.download({ owner, name, oid });
      if (!res.ok) {
        setDlError(res.error.message || "Download failed.");
        setDlOid(null);
        return;
      }
      const bin = Uint8Array.from(atob(res.data.content), (c) => c.charCodeAt(0));
      const url = URL.createObjectURL(
        new Blob([bin], { type: "application/octet-stream" }),
      );
      const a = document.createElement("a");
      a.href = url;
      a.download = oid.slice(0, 12);
      a.click();
      URL.revokeObjectURL(url);
      setDlOid(null);
    } catch {
      setDlError("Can't reach Octanest. Check your connection and try again.");
      setDlOid(null);
    }
  }

  const objects: RepoLfsObjectEntry[] = listQ.data?.objects ?? [];
  const enabled = listQ.data?.enabled ?? false;

  <section
    className="mt-8 flex flex-col gap-3"
    aria-labelledby="lfs-browser-heading"
    data-testid="lfs-browser"
  >
    <h3
      id="lfs-browser-heading"
      className="text-[15px] font-medium text-foreground"
    >
      LFS objects
    </h3>
    @if (listQ.isPending) {
      <p className="text-[14px] text-muted-foreground">Loading objects…</p>
    } @else if (listQ.isError) {
      <p role="alert" className="text-[14px] text-destructive">
        {(listQ.error instanceof Error
          ? listQ.error.message
          : "Could not load objects.") as string}
      </p>
    } @else if (!enabled) {
      <p className="text-[14px] text-muted-foreground">
        Git LFS is disabled for this repository. Enable it above to browse
        objects.
      </p>
    } @else if (objects.length === 0) {
      <p className="text-[14px] text-muted-foreground">
        No LFS objects linked yet. Push tracked files with{" "}
        <code className="text-[13px]">git lfs</code>.
      </p>
    } @else {
      <>
        @if (dlError) {
          <p role="alert" className="text-[14px] text-destructive">
            {dlError as string}
          </p>
        }
        <ul className="flex flex-col gap-2">
          @for (const obj of objects; key obj.oid) {
            <li className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-border px-3 py-2 text-[13px]">
              <span className="font-mono text-muted-foreground">
                {(obj.oid.slice(0, 16) + "… · " + formatBytes(obj.size)) as string}
              </span>
              <Button
                type="button"
                variant="secondary"
                className="h-8"
                disabled={dlOid === obj.oid}
                onClick={() => void downloadOid(obj.oid)}
              >
                {(dlOid === obj.oid ? "Downloading…" : "Download") as string}
              </Button>
            </li>
          }
        </ul>
      </>
    }
  </section>
}
