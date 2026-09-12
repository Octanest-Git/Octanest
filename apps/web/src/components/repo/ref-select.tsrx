import { Check, ChevronDown } from "@octanejs/lucide";
import type { RepoRefEntry } from "@octanest/api-client";
import {
  SelectIcon,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectList,
  SelectPopup,
  SelectPortal,
  SelectPositioner,
  SelectRoot,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { shortRefName, treeHref } from "@/lib/repo-browse";

type RefSelectProps = {
  owner: string;
  repo: string;
  refs: RepoRefEntry[];
  value: string;
  /** When set, changing ref navigates to this path under the new ref (tree or blob). */
  pathUnderRef?: string;
  mode?: "tree" | "blob" | "home";
};

/** Branch/tag Select — navigates to matching tree/blob URL (D-17). */
export function RefSelect({
  owner,
  repo,
  refs,
  value,
  pathUnderRef = "",
  mode = "tree",
}: RefSelectProps) @{
  const options = refs.map((r) => {
    const short = shortRefName(r.name);
    return { value: short, label: short, full: r.name };
  });
  const unique = options.filter(
    (o, i, arr) => arr.findIndex((x) => x.value === o.value) === i,
  );
  const current =
    unique.find((o) => o.value === value) ??
    ({ value, label: value, full: value } as const);

  function navigate(next: string) {
    if (mode === "home" || !pathUnderRef) {
      window.location.assign(treeHref(owner, repo, next, ""));
      return;
    }
    if (mode === "blob") {
      window.location.assign(
        `/${owner}/${repo}/blob/${encodeURIComponent(next)}/${pathUnderRef
          .split("/")
          .map(encodeURIComponent)
          .join("/")}`,
      );
      return;
    }
    window.location.assign(treeHref(owner, repo, next, pathUnderRef));
  }

  <SelectRoot
    value={value}
    onValueChange={(next) => {
      if (typeof next === "string" && next !== value) navigate(next);
    }}
  >
    <SelectTrigger
      className="min-w-[10rem]"
      aria-label="Switch branch or tag"
    >
      <SelectValue>{current.label as string}</SelectValue>
      <SelectIcon>
        <ChevronDown aria-hidden size={16} />
      </SelectIcon>
    </SelectTrigger>
    <SelectPortal>
      <SelectPositioner sideOffset={4}>
        <SelectPopup className="max-h-72 overflow-y-auto">
          <SelectList>
            @for (const opt of unique; key opt.value) {
              <SelectItem value={opt.value}>
                <SelectItemIndicator>
                  <Check aria-hidden size={16} className="text-primary" />
                </SelectItemIndicator>
                <SelectItemText>{opt.label as string}</SelectItemText>
              </SelectItem>
            }
          </SelectList>
        </SelectPopup>
      </SelectPositioner>
    </SelectPortal>
  </SelectRoot>
}
