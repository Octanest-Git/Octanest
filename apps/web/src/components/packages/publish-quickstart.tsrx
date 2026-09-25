import { resolvePublicOriginClient } from "@/lib/public-origin";

/** Publish quickstart for packages empty states (oci / npm / generic). */
export function PublishQuickstart(props: { owner: string; className?: string }) @{
  const origin = resolvePublicOriginClient();
  const host = origin.replace(/^https?:\/\//, "");
  <div className={`rounded-lg border border-border bg-card ${props.className ?? ""}` as string}>
    <h3 className="border-b border-border px-4 py-2.5 text-[13px] font-semibold text-foreground">
      Publish your first package
    </h3>
    <div className="flex flex-col gap-4 px-4 py-3">
      <div>
        <p className="mb-1 text-[12px] font-medium text-muted-foreground">Container (OCI)</p>
        <pre className="overflow-x-auto rounded-md bg-muted/60 px-3 py-2 font-mono text-[12px] text-foreground">
          {
            `docker login ${host}\ndocker tag IMAGE ${host}/${props.owner}/NAME:tag\ndocker push ${host}/${props.owner}/NAME:tag` as string
          }
        </pre>
      </div>
      <div>
        <p className="mb-1 text-[12px] font-medium text-muted-foreground">npm</p>
        <pre className="overflow-x-auto rounded-md bg-muted/60 px-3 py-2 font-mono text-[12px] text-foreground">
          {`npm config set registry ${origin}/npm/${props.owner}/\nnpm publish` as string}
        </pre>
      </div>
      <div>
        <p className="mb-1 text-[12px] font-medium text-muted-foreground">Generic (curl)</p>
        <pre className="overflow-x-auto rounded-md bg-muted/60 px-3 py-2 font-mono text-[12px] text-foreground">
          {
            `curl -u USER:TOKEN -T file.tar.gz \\\n  ${origin}/generic/${props.owner}/NAME/VERSION/file.tar.gz` as string
          }
        </pre>
      </div>
      <p className="text-[12px] text-muted-foreground">
        Registry clients authenticate with a personal access token (package scope), not your
        password.
      </p>
    </div>
  </div>;
}
