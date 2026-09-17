/**
 * ESM shim for `attr-accept` — the published `module` build still uses
 * CommonJS `exports` and throws `exports is not defined` in the browser,
 * which aborts Vite client hydration for any route that pulls in dropzone
 * (eager routeTree → settings/profile).
 *
 * Logic matches attr-accept@2.2.5.
 */
type AcceptFile = { name?: string; type?: string };

export default function attrAccept(
  file: AcceptFile | null | undefined,
  acceptedFiles: string | string[] | null | undefined,
): boolean {
  if (!file || !acceptedFiles) return true;
  const acceptedFilesArray = Array.isArray(acceptedFiles)
    ? acceptedFiles
    : acceptedFiles.split(",");
  if (acceptedFilesArray.length === 0) return true;

  const fileName = file.name || "";
  const mimeType = (file.type || "").toLowerCase();
  const baseMimeType = mimeType.replace(/\/.*$/, "");

  return acceptedFilesArray.some((type) => {
    const validType = type.trim().toLowerCase();
    if (validType.charAt(0) === ".") {
      return fileName.toLowerCase().endsWith(validType);
    }
    if (validType.endsWith("/*")) {
      return baseMimeType === validType.replace(/\/.*$/, "");
    }
    return mimeType === validType;
  });
}
