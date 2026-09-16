import {
  actionsGetJobLogQueryOptions,
  actionsGetRunQueryOptions,
  actionsListRunsQueryOptions,
} from "@octanest/api-client";
import { apiClient } from "@/lib/api-client";

export function actionsRunsQuery(owner: string, name: string) {
  return {
    ...actionsListRunsQueryOptions(apiClient, { owner, name }),
    refetchInterval: 10_000,
  };
}

export function actionsRunDetailQuery(owner: string, name: string, runId: string) {
  return {
    ...actionsGetRunQueryOptions(apiClient, { owner, name, run_id: runId }),
    refetchInterval: 5_000,
  };
}

export function actionsJobLogQuery(owner: string, name: string, runId: string, jobId: string) {
  return {
    ...actionsGetJobLogQueryOptions(apiClient, {
      owner,
      name,
      run_id: runId,
      job_id: jobId,
    }),
    refetchInterval: 5_000,
  };
}
