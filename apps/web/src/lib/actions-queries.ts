import {
  actionsCancelRunMutationOptions,
  actionsDispatchWorkflowMutationOptions,
  actionsGetJobLogQueryOptions,
  actionsGetRunQueryOptions,
  actionsListRunsQueryOptions,
  actionsListWorkflowsQueryOptions,
  actionsRerunRunMutationOptions,
} from "@oxidean/api-client";
import { apiClient } from "@/lib/api-client";

export const ACTIONS_RUNS_PER_PAGE = 25;

export function actionsRunsQuery(owner: string, name: string, page = 1) {
  return {
    ...actionsListRunsQueryOptions(apiClient, {
      owner,
      name,
      page,
      per_page: ACTIONS_RUNS_PER_PAGE,
    }),
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

export function actionsWorkflowsQuery(owner: string, name: string, gitRef?: string) {
  return actionsListWorkflowsQueryOptions(apiClient, { owner, name, git_ref: gitRef });
}

export function actionsDispatchWorkflowMutation() {
  return actionsDispatchWorkflowMutationOptions(apiClient);
}

export function actionsRerunRunMutation() {
  return actionsRerunRunMutationOptions(apiClient);
}

export function actionsCancelRunMutation() {
  return actionsCancelRunMutationOptions(apiClient);
}
