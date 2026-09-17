import { useEffect, useState } from "octane";
import type { LfsUsageChartRow } from "@/lib/byte-units";

type ChartProps = {
  title: string;
  testId: string;
  data: LfsUsageChartRow[];
  emptyMessage: string;
};

type ChartFn = (props: ChartProps) => unknown;

/**
 * Defers `@octanejs/recharts` until this island mounts.
 *
 * The route tree eagerly imports `/admin/lfs`; a static recharts import there
 * pulled CJS deps (`decimal.js-light`, `eventemitter3`) into every page's
 * client graph and broke hydration (template picker onClick appeared dead).
 */
export function LfsUsageBarChartGate(props: ChartProps) @{
  const [Chart, setChart] = useState<ChartFn | null>(null);

  useEffect(() => {
    let cancelled = false;
    void import("@/components/admin/lfs-usage-charts").then((mod) => {
      if (!cancelled) setChart(() => mod.LfsUsageBarChart as ChartFn);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  @if (Chart) {
    <Chart
      title={props.title}
      testId={props.testId}
      data={props.data}
      emptyMessage={props.emptyMessage}
    />;
  } @else {
    <p className="text-[13px] text-muted-foreground" data-testid={props.testId}>
      Loading usage charts…
    </p>;
  }
}
