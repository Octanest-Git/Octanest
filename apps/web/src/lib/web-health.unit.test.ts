import { describe, expect, it } from "@octanest/web/test-runner";
import {
  WEB_HEALTH_PROBE_HEADER,
  WEB_HEALTH_PROBE_VALUE,
  isAuthorizedWebHealthProbe,
} from "../../vite-plugins/web-health.ts";

describe("web health probe auth", () => {
  it("accepts Octanest-Health-Probe: 1", () => {
    expect(isAuthorizedWebHealthProbe({ [WEB_HEALTH_PROBE_HEADER]: WEB_HEALTH_PROBE_VALUE })).toBe(
      true,
    );
  });

  it("rejects missing header", () => {
    expect(isAuthorizedWebHealthProbe({})).toBe(false);
  });

  it("rejects wrong value", () => {
    expect(isAuthorizedWebHealthProbe({ [WEB_HEALTH_PROBE_HEADER]: "yes" })).toBe(false);
  });
});
