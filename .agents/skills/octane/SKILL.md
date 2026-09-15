---
name: octane
description: >-
  Octane UI authoring for Octanest (`.tsrx`, Rivet templates, native events,
  TanStack Query via @octanejs). Use when editing apps/web components, routes,
  SSR loaders, or when tempted to write React JSX patterns. Prefer this over
  treating the UI as React.
---

# Octane (Octanest)

Official reference: [octanejs.dev/llms.txt](https://octanejs.dev/llms.txt) · [Differences from React](https://octanejs.dev/docs/differences-from-react) · [TSRX vs TSX](https://octanejs.dev/docs/tsrx-vs-tsx)

Octane is Inferno’s successor with a React-*shaped* programming model (hooks, Suspense, transitions) but **AOT compilation**, **no virtual DOM**, and **native DOM events**. Octanest ships UI almost entirely as **`.tsrx`**.

## Project defaults

| Do | Don’t |
|----|--------|
| Author UI in `.tsrx` with `function X() @{ … }` | Treat files as React and `return (` mixed with Rivet |
| Use `@if` / `@else`, `@for (…; key …)`, `@switch` | Invent `@else if` (unsupported — nest `@if` or use `@switch`) |
| Text inputs: `onInput` + controlled `value` | React synthetic `onChange` for per-edit text |
| Session/server data via `@octanejs/tanstack-query` | New global stores for `auth.me` / admin settings |
| Forms via `@octanejs/tanstack-form` (`useForm`, `onInput` + `field.handleChange`) | Parallel `useState` per field for multi-field forms |
| File pickers via `@octanejs/dropzone` / `FileDropzone` | Ad-hoc hidden `<input type="file">` without dropzone |
| Forms: `method="post" action="#"` + `type="button"` where needed | Rely on GET navigations from submit |
| Anonymous auth pages: SSR loaders, no form skeletons | Skeleton-first anonymous login/signup |

Shared session helpers: `apps/web/src/lib/session-queries.ts`, `use-chrome-account.ts`, `query-client.ts`. Test with `apps/web/src/test/render-with-query.ts`.

Before committing web UI changes: `make web-lint` and `make web-format-check` (`@tsrx/oxc` — type-aware oxlint + oxfmt). See [AGENTS.md](../../../AGENTS.md).

## Authoring `.tsrx`

```tsrx
export function Example(props: { title: string }) {
  const [open, setOpen] = useState(false);

  @{
    <section>
      <h1>{props.title as string}</h1>
      @if (open) {
        <p>Visible</p>
      } @else {
        <p>Hidden</p>
      }
      <button type="button" onClick={() => setOpen(!open)}>Toggle</button>
    </section>
  }
}
```

- `@{ … }` is the template return shorthand — **one** output node (element or `<>…</>`).
- Dynamic text often needs `{expr as string}` when not provably a string.
- Hooks may be conditional (compiler call-site slots). Do **not** put a hook in a plain JS `for` loop — use `@for` or a child component.
- `useState` / `useReducer` expose a stable third tuple member `[state, set, getState]` when observed — prefer that over a ref for latest async state.
- Omit dependency arrays on `useEffect` / `useMemo` / `useCallback` when the compiler can infer them; explicit arrays keep React semantics; `null` means every render.
- Refs are props (`ref={…}`); no `forwardRef`.
- Use `class` / `className` (clsx-style). Prefer project Tailwind / existing chrome styles over new design systems.

## Events (native, not synthetic)

- Per-edit text: `onInput`.
- Commit-on-blur with native `change`: keep `onChange` and add `suppressNativeChangeWarning` — never add a noop `onInput` to silence warnings.
- Checkbox/radio: cancel activation in `onClick` if needed; native `change` is not cancelable the React way.

## Control flow

Template directives only inside `@{ }` / directive bodies:

- `@if (c) { } @else { }` — nest for else-if
- `@for (const x of xs; key x.id) { } @empty { }`
- `@switch (v) { @case (a) { } @default { } }`
- `@try { } @pending { } @catch (e) { }`

Plain JS control flow belongs in setup (above `@{`), not mixed as React ternary soup inside broken `return (` templates.

## Mixing React (rare)

Only if required: `ReactCompat` / `OctaneCompat` from `octane/react`. Do **not** alias `react` to Octane. Prefer `@octanejs/*` bindings (e.g. `@octanejs/tanstack-query`, `@octanejs/tanstack-router`) already used in-repo.

## Failure modes seen in this repo

1. **Export undefined / no hydration** — component used `return (` with `@if` / `@{` fragments → Vite import protection. Fix: full Rivet `@{` body.
2. **GET form submits** — missing `method="post" action="#"` or button `type="button"` on SPA forms.
3. **Duplicate `auth.me`** — bypass shared Query helpers; always go through session query options / cache helpers.

## When stuck

1. Re-read https://octanejs.dev/llms.txt
2. Mirror an existing route under `apps/web/src/routes/*.tsrx` or `components/*.tsrx`
3. Run the matching Vitest project (`*.integration.test.ts` / e2e) before claiming UI done
