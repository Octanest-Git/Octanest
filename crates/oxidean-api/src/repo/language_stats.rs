//! Linguist-lite language breakdown for the About sidebar.
//!
//! GitHub Linguist aggregates included file bytes by detected language. We approximate
//! that with extension (+ a few filenames), vendored-path skips, and prose/data exclusions —
//! enough for a GitHub-shaped bar without shipping full linguist.

use std::collections::HashMap;

use oxidean_core::RepoLanguageStat;

/// Soft cap on blobs considered (matches git backend clamp).
pub const MAX_BLOBS: u32 = 50_000;

/// Max distinct languages returned (GitHub shows the long tail as “Other”).
const MAX_NAMED: usize = 8;

/// Aggregate sized blobs into sorted language stats (largest first).
pub fn aggregate_language_stats<'a, I>(blobs: I) -> Vec<RepoLanguageStat>
where
    I: IntoIterator<Item = (&'a str, u64)>,
{
    let mut totals: HashMap<&'static str, u64> = HashMap::new();
    for (path, size) in blobs {
        if size == 0 || is_vendored_path(path) {
            continue;
        }
        if let Some(lang) = detect_language(path) {
            *totals.entry(lang).or_insert(0) += size;
        }
    }

    let mut rows: Vec<(&str, u64)> = totals.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    if rows.is_empty() {
        return Vec::new();
    }

    let mut languages = Vec::new();
    let mut other_bytes: u64 = 0;
    for (i, (name, bytes)) in rows.into_iter().enumerate() {
        if i < MAX_NAMED {
            languages.push(RepoLanguageStat {
                name: name.to_string(),
                bytes,
                color: language_color(name).map(str::to_string),
            });
        } else {
            other_bytes = other_bytes.saturating_add(bytes);
        }
    }
    if other_bytes > 0 {
        languages.push(RepoLanguageStat {
            name: "Other".into(),
            bytes: other_bytes,
            color: Some("#ededed".into()),
        });
    }
    languages
}

fn is_vendored_path(path: &str) -> bool {
    let lower = path.replace('\\', "/").to_ascii_lowercase();
    let segments: Vec<&str> = lower.split('/').filter(|s| !s.is_empty()).collect();
    for seg in &segments {
        match *seg {
            "node_modules" | "vendor" | "third_party" | "third-party" | "bower_components"
            | "dist" | "build" | "target" | "out" | ".yarn" | "__pycache__" | ".tox"
            | "coverage" | ".next" | "pods" | "carthage" | ".git" | "venv" | ".venv" => {
                return true;
            }
            _ => {}
        }
    }
    let file = segments.last().copied().unwrap_or("");
    matches!(
        file,
        "package-lock.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "bun.lock"
            | "bun.lockb"
            | "cargo.lock"
            | "composer.lock"
            | "gemfile.lock"
            | "poetry.lock"
            | "go.sum"
    ) || file.ends_with(".min.js")
        || file.ends_with(".min.css")
        || file.ends_with(".map")
}

/// Map path → language name, or `None` when excluded (prose/data/binary/unknown).
fn detect_language(path: &str) -> Option<&'static str> {
    let path = path.replace('\\', "/");
    let file = path.rsplit('/').next().unwrap_or(path.as_str());
    let lower = file.to_ascii_lowercase();

    // Special filenames (linguist-style).
    match lower.as_str() {
        "dockerfile" | "containerfile" => return Some("Dockerfile"),
        "makefile" | "gnumakefile" => return Some("Makefile"),
        "cmakelists.txt" => return Some("CMake"),
        "go.mod" | "go.sum" => return None,
        _ => {}
    }

    let ext = match lower.rsplit_once('.') {
        Some((_, e)) if !e.is_empty() && e != lower => e,
        _ => return None,
    };

    match ext {
        // Programming
        "rs" => Some("Rust"),
        "tsrx" => Some("TSRX"),
        "ts" | "mts" | "cts" | "tsx" => Some("TypeScript"),
        "js" | "mjs" | "cjs" | "jsx" => Some("JavaScript"),
        "py" | "pyi" => Some("Python"),
        "go" => Some("Go"),
        "rb" => Some("Ruby"),
        "php" => Some("PHP"),
        "java" => Some("Java"),
        "kt" | "kts" => Some("Kotlin"),
        "swift" => Some("Swift"),
        "scala" | "sc" => Some("Scala"),
        "c" | "h" => Some("C"),
        "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "hh" => Some("C++"),
        "cs" => Some("C#"),
        "fs" | "fsi" | "fsx" => Some("F#"),
        "ex" | "exs" => Some("Elixir"),
        "erl" | "hrl" => Some("Erlang"),
        "hs" | "lhs" => Some("Haskell"),
        "clj" | "cljs" | "cljc" => Some("Clojure"),
        "lua" => Some("Lua"),
        "r" => Some("R"),
        "dart" => Some("Dart"),
        "zig" => Some("Zig"),
        "nim" => Some("Nim"),
        "v" => Some("V"),
        "pl" | "pm" => Some("Perl"),
        "jl" => Some("Julia"),
        "groovy" | "gradle" => Some("Groovy"),
        "m" => Some("Objective-C"),
        "mm" => Some("Objective-C++"),
        "sh" | "bash" | "zsh" | "ksh" | "fish" => Some("Shell"),
        "ps1" | "psm1" => Some("PowerShell"),
        "bat" | "cmd" => Some("Batchfile"),
        "vim" => Some("Vim Script"),
        "el" => Some("Emacs Lisp"),
        "cmake" => Some("CMake"),
        "proto" => Some("Protocol Buffer"),
        "graphql" | "gql" => Some("GraphQL"),
        "sql" => Some("SQL"),
        // Markup / style (GitHub includes these in the bar)
        "html" | "htm" | "xhtml" => Some("HTML"),
        "css" => Some("CSS"),
        "scss" => Some("SCSS"),
        "sass" => Some("Sass"),
        "less" => Some("Less"),
        "vue" => Some("Vue"),
        "svelte" => Some("Svelte"),
        "astro" => Some("Astro"),
        // Prose / data / binary — excluded from stats (linguist-style)
        "md" | "mdx" | "markdown" | "rst" | "txt" | "adoc" | "asciidoc" => None,
        "json" | "jsonc" | "json5" | "yaml" | "yml" | "toml" | "xml" | "csv" | "tsv" | "ini"
        | "cfg" | "conf" | "env" | "properties" | "lock" => None,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "svg" | "pdf" | "zip" | "gz" | "tgz"
        | "bz2" | "xz" | "7z" | "rar" | "woff" | "woff2" | "ttf" | "otf" | "eot" | "mp3"
        | "mp4" | "webm" | "wasm" | "bin" | "exe" | "dll" | "so" | "dylib" | "o" | "a"
        | "class" | "jar" | "pyc" | "pyo" => None,
        _ => None,
    }
}

fn language_color(name: &str) -> Option<&'static str> {
    // Subset of github-linguist colors (languages.yml).
    Some(match name {
        "Rust" => "#dea584",
        "TSRX" => "#6f00ff", // tsrx.dev brand mark purple
        "TypeScript" => "#3178c6",
        "JavaScript" => "#f1e05a",
        "Python" => "#3572a5",
        "Go" => "#00add8",
        "Ruby" => "#701516",
        "PHP" => "#4f5d95",
        "Java" => "#b07219",
        "Kotlin" => "#a97bff",
        "Swift" => "#f05138",
        "Scala" => "#c22d40",
        "C" => "#555555",
        "C++" => "#f34b7d",
        "C#" => "#178600",
        "F#" => "#b845fc",
        "Elixir" => "#6e4a7e",
        "Erlang" => "#b83998",
        "Haskell" => "#5e5086",
        "Clojure" => "#db5855",
        "Lua" => "#000080",
        "R" => "#198ce7",
        "Dart" => "#00b4ab",
        "Zig" => "#ec915c",
        "Nim" => "#ffc200",
        "Perl" => "#0298c3",
        "Julia" => "#a270ba",
        "Groovy" => "#4298b8",
        "Objective-C" => "#438eff",
        "Objective-C++" => "#6866fb",
        "Shell" => "#89e051",
        "PowerShell" => "#012456",
        "Batchfile" => "#c1f12e",
        "Vim Script" => "#199f4b",
        "Emacs Lisp" => "#c065db",
        "Dockerfile" => "#384d54",
        "Makefile" => "#427819",
        "CMake" => "#da3434",
        "Protocol Buffer" => "#ededed",
        "GraphQL" => "#e10098",
        "SQL" => "#e38c00",
        "HTML" => "#e34c26",
        "CSS" => "#563d7c",
        "SCSS" => "#c6538c",
        "Sass" => "#a53b70",
        "Less" => "#1d365d",
        "Vue" => "#41b883",
        "Svelte" => "#ff3e00",
        "Astro" => "#ff5a03",
        "Other" => "#ededed",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_by_extension_and_sorts() {
        let blobs = [
            ("src/main.rs", 1000u64),
            ("src/lib.rs", 500),
            ("web/app.ts", 200),
            ("web/util.js", 100),
            ("README.md", 9999), // prose — excluded
            ("data.json", 8000), // data — excluded
            ("node_modules/x.js", 50_000), // vendored
        ];
        let stats = aggregate_language_stats(blobs.iter().map(|(p, s)| (*p, *s)));
        assert_eq!(stats[0].name, "Rust");
        assert_eq!(stats[0].bytes, 1500);
        assert_eq!(stats[1].name, "TypeScript");
        assert_eq!(stats[1].bytes, 200);
        assert_eq!(stats[2].name, "JavaScript");
        assert_eq!(stats[2].bytes, 100);
        assert!(stats.iter().all(|s| s.name != "Markdown"));
    }

    #[test]
    fn empty_when_no_code() {
        let blobs = [("README.md", 10u64), ("a.json", 20)];
        assert!(aggregate_language_stats(blobs.iter().map(|(p, s)| (*p, *s))).is_empty());
    }

    #[test]
    fn folds_tail_into_other() {
        let mut blobs = Vec::new();
        // 10 languages with decreasing sizes
        let langs = [
            ("a.rs", 1000u64),
            ("b.ts", 900),
            ("c.py", 800),
            ("d.go", 700),
            ("e.rb", 600),
            ("f.java", 500),
            ("g.kt", 400),
            ("h.swift", 300),
            ("i.lua", 200),
            ("j.dart", 100),
        ];
        for (p, s) in langs {
            blobs.push((p, s));
        }
        let stats = aggregate_language_stats(blobs.iter().map(|(p, s)| (*p, *s)));
        assert_eq!(stats.len(), 9); // 8 named + Other
        assert_eq!(stats.last().unwrap().name, "Other");
        assert_eq!(stats.last().unwrap().bytes, 300); // lua+dart
    }

    #[test]
    fn tsrx_is_first_class_language() {
        let blobs = [
            ("ui/app.tsrx", 500u64),
            ("ui/util.ts", 100),
            ("src/main.rs", 50),
        ];
        let stats = aggregate_language_stats(blobs.iter().map(|(p, s)| (*p, *s)));
        assert_eq!(stats[0].name, "TSRX");
        assert_eq!(stats[0].bytes, 500);
        assert_eq!(stats[0].color.as_deref(), Some("#6f00ff"));
        assert_eq!(stats[1].name, "TypeScript");
        assert_eq!(stats[1].bytes, 100);
    }
}
