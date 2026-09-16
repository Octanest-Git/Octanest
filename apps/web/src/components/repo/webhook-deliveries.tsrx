import type { WebhookDeliveryPublic } from "@octanest/api-client";
import { useQuery, useQueryClient } from "@octanejs/tanstack-query";
import { useState } from "octane";
import { Button } from "@/components/ui/button";
import { apiClient } from "@/lib/api-client";
import { toastSuccess } from "@/lib/toast";

const NETWORK_ERROR = "Can't reach Octanest. Check your connection and try again.";

/** Recent delivery attempts for one webhook (HOOK-03 / D-HOOK-20 / D-HOOK-21). */
export function WebhookDeliveries(props: {
  owner: string;
  name: string;
  webhookId: string;
}) @{
  const owner = props.owner;
  const name = props.name;
  const webhookId = props.webhookId;
  const queryClient = useQueryClient();
  const [busy, setBusy] = useState("");
  const [error, setError] = useState("");

  const listQ = useQuery({
    queryKey: ["webhook", "deliveries", owner, name, webhookId],
    queryFn: async () => {
      const res = await apiClient.webhook.deliveries.list({
        owner,
        name,
        webhook_id: webhookId,
        limit: 25,
      });
      if (!res.ok) throw new Error(res.error.message || "Could not load deliveries.");
      return res.data;
    },
  });

  async function ping() {
    setError("");
    setBusy("ping");
    try {
      const res = await apiClient.webhook.ping({ owner, name, id: webhookId });
      if (!res.ok) {
        setError(res.error.message || "Ping failed.");
        setBusy("");
        return;
      }
      await queryClient.invalidateQueries({
        queryKey: ["webhook", "deliveries", owner, name, webhookId],
      });
      toastSuccess("Ping enqueued.");
      setBusy("");
    } catch {
      setError(NETWORK_ERROR);
      setBusy("");
    }
  }

  async function redeliver(deliveryId: string) {
    setError("");
    setBusy(deliveryId);
    try {
      const res = await apiClient.webhook.redeliver({
        owner,
        name,
        webhook_id: webhookId,
        delivery_id: deliveryId,
      });
      if (!res.ok) {
        setError(res.error.message || "Redeliver failed.");
        setBusy("");
        return;
      }
      await queryClient.invalidateQueries({
        queryKey: ["webhook", "deliveries", owner, name, webhookId],
      });
      toastSuccess("Redelivery enqueued.");
      setBusy("");
    } catch {
      setError(NETWORK_ERROR);
      setBusy("");
    }
  }

  const deliveries: WebhookDeliveryPublic[] = listQ.data?.deliveries ?? [];

  <div className="mt-4 flex flex-col gap-3" data-testid="webhook-deliveries">
    <div className="flex items-center justify-between gap-3">
      <h4 className="text-[15px] font-semibold text-foreground">Recent deliveries</h4>
      <Button
        type="button"
        variant="secondary"
        className="h-9 px-3"
        disabled={busy === "ping"}
        onClick={() => void ping()}
      >
        {(busy === "ping" ? "Pinging…" : "Ping") as string}
      </Button>
    </div>
    @if (error) {
      <p role="alert" className="text-[14px] text-destructive">
        {error as string}
      </p>;
    }
    @if (listQ.isPending) {
      <p className="text-[14px] text-muted-foreground">Loading deliveries…</p>;
    } @else {
      @if (listQ.isError) {
        <p role="alert" className="text-[14px] text-destructive">
          {
            (listQ.error instanceof Error
              ? listQ.error.message
              : "Could not load deliveries.") as string
          }
        </p>;
      } @else {
        @if (deliveries.length === 0) {
          <p className="text-[14px] text-muted-foreground">No deliveries yet.</p>;
        } @else {
          <div className="overflow-x-auto">
            <table className="w-full text-left text-[13px]">
              <thead>
                <tr className="border-b border-border text-muted-foreground">
                  <th className="py-2 pr-3 font-medium">Event</th>
                  <th className="py-2 pr-3 font-medium">Status</th>
                  <th className="py-2 pr-3 font-medium">HTTP</th>
                  <th className="py-2 pr-3 font-medium">When</th>
                  <th className="py-2 font-medium"></th>
                </tr>
              </thead>
              <tbody>
                @for (const d of deliveries; key d.id) {
                  <tr className="border-b border-border/60">
                    <td className="py-2 pr-3">
                      {(d.action ? `${d.event}.${d.action}` : d.event) as string}
                    </td>
                    <td className="py-2 pr-3">{d.status as string}</td>
                    <td className="py-2 pr-3">
                      {(d.http_status != null ? String(d.http_status) : "—") as string}
                    </td>
                    <td className="py-2 pr-3 text-muted-foreground">
                      {d.created_at as string}
                    </td>
                    <td className="py-2">
                      <Button
                        type="button"
                        variant="ghost"
                        className="h-9 px-3"
                        disabled={busy === d.id}
                        onClick={() => void redeliver(d.id)}
                      >
                        Redeliver
                      </Button>
                    </td>
                  </tr>;
                }
              </tbody>
            </table>
          </div>;
        }
      }
    }
  </div>;
}
