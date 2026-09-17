/**
 * Capture window error / unhandledrejection messages during an interaction.
 * Used to catch Octane DOM races (insertBefore / HierarchyRequestError) that
 * happy-dom may not always surface as a failed assertion otherwise.
 */
export function trackDomErrors() {
  const errors: string[] = [];

  const onError = (ev: ErrorEvent) => {
    const msg = ev.message || (ev.error instanceof Error ? ev.error.message : String(ev.error));
    if (msg) errors.push(msg);
  };
  const onRejection = (ev: PromiseRejectionEvent) => {
    const reason = ev.reason;
    errors.push(reason instanceof Error ? reason.message : String(reason));
  };

  window.addEventListener("error", onError);
  window.addEventListener("unhandledrejection", onRejection);

  return {
    errors,
    /** Messages that indicate Octane/portal DOM hierarchy races. */
    domRaceErrors() {
      return errors.filter((m) =>
        /insertBefore|HierarchyRequestError|NotFoundError|The node before which/i.test(m),
      );
    },
    dispose() {
      window.removeEventListener("error", onError);
      window.removeEventListener("unhandledrejection", onRejection);
    },
  };
}
