import { describe, expect, it } from "vitest";
import { classifyDiffLine, parseUnifiedDiffLines } from "./parse-unified-diff";

describe("classifyDiffLine", () => {
  it("tags add/del/ctx/hunk/meta", () => {
    expect(classifyDiffLine("+foo")).toBe("add");
    expect(classifyDiffLine("-bar")).toBe("del");
    expect(classifyDiffLine(" baz")).toBe("ctx");
    expect(classifyDiffLine("@@ -1,2 +3,4 @@")).toBe("hunk");
    expect(classifyDiffLine("diff --git a/x b/x")).toBe("meta");
    expect(classifyDiffLine("--- /dev/null")).toBe("meta");
    expect(classifyDiffLine("+++ b/x")).toBe("meta");
    expect(classifyDiffLine("\\ No newline at end of file")).toBe("note");
  });
});

describe("parseUnifiedDiffLines", () => {
  it("splits a sample added-file patch into styled rows", () => {
    const patch = [
      "diff --git a/README.md b/README.md",
      "new file mode 100644",
      "index 0000000..abc1234",
      "--- /dev/null",
      "+++ b/README.md",
      "@@ -0,0 +1,2 @@",
      "+# Hello",
      "+world",
      "",
    ].join("\n");
    const lines = parseUnifiedDiffLines(patch);
    expect(lines.map((l) => l.kind)).toEqual([
      "meta",
      "meta",
      "meta",
      "meta",
      "meta",
      "hunk",
      "add",
      "add",
    ]);
    expect(lines[6]?.text).toBe("+# Hello");
  });

  it("handles CRLF and mixed change hunks", () => {
    const patch = "@@ -1,3 +1,3 @@\r\n context\r\n-old\r\n+new\r\n keep\r\n";
    const lines = parseUnifiedDiffLines(patch);
    expect(lines.map((l) => l.kind)).toEqual(["hunk", "ctx", "del", "add", "ctx"]);
  });
});
