import {
  AppWindow,
  Boxes,
  Check,
  Cloud,
  Code2,
  Container,
  Cpu,
  FileText,
  HardDrive,
  LayoutTemplate,
  Monitor,
  Scale,
  Search,
  Server,
  Smartphone,
  Sparkles,
  Wrench,
} from "@octanejs/lucide";
import type { RepoTemplateOption } from "@octanest/api-client";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

type TemplatePickerProps = {
  id: string;
  label: string;
  description: string;
  value: string;
  options: RepoTemplateOption[];
  noneLabel?: string;
  noneDescription?: string;
  /** When set and search is empty, only these groups are listed (e.g. Popular licenses). */
  browseGroups?: string[];
  searchPlaceholder?: string;
  onChange: (next: string) => void;
};

type TemplateCardProps = {
  option: RepoTemplateOption | null;
  noneLabel: string;
  noneDescription: string;
  active: boolean;
  onSelect: () => void;
};

function iconForGroup(group: string) {
  switch (group) {
    case "Systems":
      return Cpu;
    case "Backend":
      return Server;
    case "Frontend":
      return LayoutTemplate;
    case "Desktop":
      return Monitor;
    case "Mobile":
      return Smartphone;
    case "Infra":
      return Cloud;
    case "From template":
      return LayoutTemplate;
    case "Languages":
      return Code2;
    case "Tools":
      return Wrench;
    case "IDEs":
      return AppWindow;
    case "OS":
      return HardDrive;
    case "General":
      return FileText;
    case "Popular":
      return Sparkles;
    case "All SPDX":
      return Scale;
    default:
      return Boxes;
  }
}

function accentForGroup(group: string): string {
  switch (group) {
    case "Systems":
      return "bg-orange-500/15 text-orange-600 dark:text-orange-300";
    case "Backend":
      return "bg-sky-500/15 text-sky-700 dark:text-sky-300";
    case "Frontend":
      return "bg-violet-500/15 text-violet-700 dark:text-violet-300";
    case "Desktop":
      return "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300";
    case "Mobile":
      return "bg-pink-500/15 text-pink-700 dark:text-pink-300";
    case "Infra":
      return "bg-cyan-500/15 text-cyan-700 dark:text-cyan-300";
    case "Languages":
      return "bg-amber-500/15 text-amber-800 dark:text-amber-300";
    case "Tools":
      return "bg-lime-500/15 text-lime-800 dark:text-lime-300";
    case "IDEs":
      return "bg-indigo-500/15 text-indigo-700 dark:text-indigo-300";
    case "OS":
      return "bg-slate-500/15 text-slate-700 dark:text-slate-300";
    case "Popular":
      return "bg-primary/15 text-primary";
    case "All SPDX":
      return "bg-muted text-muted-foreground";
    default:
      return "bg-muted text-muted-foreground";
  }
}

function TemplateCard({
  option,
  noneLabel,
  noneDescription,
  active,
  onSelect,
}: TemplateCardProps) @{
  const group = option?.group ?? "General";
  const Icon = option ? iconForGroup(group) : Container;
  const title = option ? option.label : noneLabel;
  const provenance =
    option?.provenance === "instance"
      ? "Instance"
      : option?.provenance === "user"
        ? option.source_label
          ? `Template · ${option.source_label}`
          : "Template"
        : option
          ? "Official"
          : null;
  const body = option
    ? [provenance, option.description || option.group].filter(Boolean).join(" — ")
    : noneDescription;
  const accent = option ? accentForGroup(group) : "bg-muted text-muted-foreground";

  <button
    type="button"
    className={cn(
      "flex w-full items-start gap-3 rounded-lg border px-3 py-3 text-left outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring",
      active ? "border-primary bg-primary/5" : "border-border hover:bg-muted/50",
    )}
    onClick={onSelect}
  >
    <span
      className={cn(
        "mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-md",
        accent as string,
      )}
    >
      <Icon aria-hidden size={20} />
    </span>
    <span className="min-w-0 flex-1">
      <span className="flex items-center gap-2 text-[14px] font-semibold text-foreground">
        {title as string}
        @if (active) {
          <Check aria-hidden size={16} className="text-primary" />;
        }
      </span>
      <span className="mt-0.5 block text-[13px] leading-snug text-muted-foreground">
        {body as string}
      </span>
    </span>
  </button>;
}

/** Field trigger + searchable card modal for stack / gitignore / license catalogs. */
export function TemplatePicker({
  id,
  label,
  description,
  value,
  options,
  noneLabel = "None",
  noneDescription = "Skip this template — leave the tree unchanged for this file.",
  browseGroups,
  searchPlaceholder = "Search templates…",
  onChange,
}: TemplatePickerProps) @{
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");

  const selected = value === "none" ? null : (options.find((o) => o.id === value) ?? null);

  const q = query.trim().toLowerCase();
  const filtered = !q
    ? browseGroups && browseGroups.length > 0
      ? options.filter((o) => browseGroups.includes(o.group))
      : options
    : options.filter(
        (o) =>
          o.label.toLowerCase().includes(q) ||
          o.group.toLowerCase().includes(q) ||
          o.description.toLowerCase().includes(q) ||
          o.id.toLowerCase().includes(q),
      );

  // Prefer Popular before lexicographic for license-style catalogs.
  const groups = [...new Set(filtered.map((o) => o.group))].sort((a, b) => {
    if (a === "Popular") return -1;
    if (b === "Popular") return 1;
    return a.localeCompare(b);
  });

  function choose(next: string) {
    onChange(next);
    setOpen(false);
    setQuery("");
  }

  const triggerIconGroup = selected?.group ?? "General";
  const TriggerIcon = selected ? iconForGroup(triggerIconGroup) : Container;
  const triggerAccent = selected
    ? accentForGroup(triggerIconGroup)
    : "bg-muted text-muted-foreground";
  const triggerTitle = selected ? selected.label : noneLabel;
  const triggerBody = selected ? selected.description || selected.group : noneDescription;

  <div className="flex flex-col gap-2">
    <Label htmlFor={id}>{label as string}</Label>
    <button
      id={id}
      type="button"
      aria-label={label}
      aria-haspopup="dialog"
      aria-expanded={open}
      className="flex w-full items-start gap-3 rounded-md border border-border bg-card px-3 py-3 text-left outline-none transition-colors hover:bg-muted/60 focus-visible:ring-2 focus-visible:ring-ring"
      onClick={() => setOpen(true)}
    >
      <span
        className={cn(
          "mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-md",
          triggerAccent as string,
        )}
      >
        <TriggerIcon aria-hidden size={20} />
      </span>
      <span className="min-w-0 flex-1">
        <span className="block text-[14px] font-semibold text-foreground">
          {triggerTitle as string}
        </span>
        <span className="mt-0.5 block text-[13px] text-muted-foreground">
          {triggerBody as string}
        </span>
      </span>
      <span className="shrink-0 self-center text-[13px] font-semibold text-primary">Choose…</span>
    </button>
    <p className="text-[13px] text-muted-foreground">{description as string}</p>

    <Dialog
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) setQuery("");
      }}
    >
      <DialogContent
        className="flex max-h-[min(90vh,44rem)] w-full max-w-3xl flex-col gap-0 overflow-hidden p-0 sm:max-w-3xl"
        showCloseButton={true}
      >
        <DialogHeader className="border-b border-border px-5 py-4 pr-12">
          <DialogTitle>Choose {label as string}</DialogTitle>
          <DialogDescription>
            {
              (browseGroups && browseGroups.length > 0
                ? "Browse popular picks or search the full catalog. Selecting a card applies it immediately."
                : "Browse by category or search. Selecting a card applies it immediately.") as string
            }
          </DialogDescription>
          <div className="relative mt-3">
            <Search
              aria-hidden
              size={16}
              className="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-muted-foreground"
            />
            <Input
              type="search"
              value={query}
              placeholder={searchPlaceholder as string}
              aria-label={`Search ${label}`}
              className="pl-9"
              onInput={(ev) => setQuery((ev.currentTarget as HTMLInputElement).value)}
            />
          </div>
        </DialogHeader>

        <div className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
          <div className="mb-4">
            <TemplateCard
              option={null}
              noneLabel={noneLabel}
              noneDescription={noneDescription}
              active={value === "none"}
              onSelect={() => choose("none")}
            />
          </div>

          @if (filtered.length === 0) {
            <p className="py-8 text-center text-[14px] text-muted-foreground">
              No templates match your search.
            </p>;
          } @else {
            <div className="flex flex-col gap-6">
              @for (const group of groups; key group) {
                <section aria-labelledby={`${id}-group-${group}`}>
                  <h3
                    id={`${id}-group-${group}`}
                    className="mb-2 text-[12px] font-semibold tracking-wide text-muted-foreground uppercase"
                  >
                    {group as string}
                  </h3>
                  <div className="grid gap-2 sm:grid-cols-2">
                    @for (const opt of filtered.filter(
                        (o) => o.group === group,
                      ); key opt.id) {
                      <TemplateCard
                        option={opt}
                        noneLabel={noneLabel}
                        noneDescription={noneDescription}
                        active={value === opt.id}
                        onSelect={() => choose(opt.id)}
                      />;
                    }
                  </div>
                </section>;
              }
            </div>;
          }
        </div>

        <div className="flex justify-end border-t border-border bg-muted/30 px-5 py-3">
          <Button type="button" variant="secondary" onClick={() => setOpen(false)}>
            Close
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  </div>;
}
