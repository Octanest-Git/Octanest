import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "@octanejs/recharts";
import type { LfsUsageChartRow } from "@/lib/byte-units";

type LfsUsageBarChartProps = {
  title: string;
  testId: string;
  data: LfsUsageChartRow[];
  emptyMessage: string;
};

/** Horizontal bar chart of logical LFS usage (GiB) via @octanejs/recharts. */
export function LfsUsageBarChart({
  title,
  testId,
  data,
  emptyMessage,
}: LfsUsageBarChartProps) @{
  const height = Math.max(160, data.length * 40 + 48);

  <div className="flex min-w-0 flex-col gap-2" data-testid={testId}>
    <h3 className="text-[15px] font-medium">{title as string}</h3>
    @if (data.length === 0) {
      <p className="text-[13px] text-muted-foreground">{emptyMessage as string}</p>;
    } @else {
      <div className="w-full overflow-x-auto" role="img" aria-label={title}>
        <BarChart
          layout="vertical"
          width={520}
          height={height}
          data={data}
          margin={{ top: 8, right: 24, left: 8, bottom: 8 }}
        >
          <CartesianGrid strokeDasharray="3 3" horizontal={false} />
          <XAxis type="number" dataKey="giB" unit=" GiB" tick={{ fontSize: 12 }} />
          <YAxis type="category" dataKey="label" width={140} tick={{ fontSize: 12 }} />
          <Bar
            dataKey="giB"
            name="Logical"
            fill="var(--primary)"
            isAnimationActive={false}
            radius={[0, 4, 4, 0]}
          />
        </BarChart>
      </div>;
    }
  </div>;
}
