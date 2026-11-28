//! Source code as data for `pkcode`: a small tokenizer that classifies
//! enough to colour a listing, and a line diff.
//!
//! This is a highlighter for pictures, not a parser: it knows comments,
//! strings, numbers, a keyword list per language, and guesses types (a
//! capitalised word) and calls (a word followed by an opening parenthesis).

/// A language the tokenizer has a keyword list and comment rules for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Python,
    JavaScript,
    C,
    Go,
    PostScript,
    Shell,
    Json,
    Plain,
}

pub const LANG_NAMES: &str =
    "/rust /python /javascript /typescript /c /go /postscript /shell /json /plain";

impl Lang {
    pub fn from_name(name: &str) -> Option<Lang> {
        Some(match name {
            "rust" | "rs" => Lang::Rust,
            "python" | "py" => Lang::Python,
            "javascript" | "js" | "typescript" | "ts" => Lang::JavaScript,
            "c" | "cpp" | "java" => Lang::C,
            "go" => Lang::Go,
            "postscript" | "ps" => Lang::PostScript,
            "shell" | "sh" | "bash" => Lang::Shell,
            "json" => Lang::Json,
            "plain" | "text" => Lang::Plain,
            _ => return None,
        })
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Lang::Rust => &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
                "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            ],
            Lang::Python => &[
                "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                "del", "elif", "else", "except", "False", "finally", "for", "from", "global", "if",
                "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise",
                "return", "True", "try", "while", "with", "yield",
            ],
            Lang::JavaScript => &[
                "async",
                "await",
                "break",
                "case",
                "catch",
                "class",
                "const",
                "continue",
                "default",
                "delete",
                "do",
                "else",
                "export",
                "extends",
                "false",
                "finally",
                "for",
                "from",
                "function",
                "if",
                "import",
                "in",
                "instanceof",
                "interface",
                "let",
                "new",
                "null",
                "of",
                "return",
                "static",
                "super",
                "switch",
                "this",
                "throw",
                "true",
                "try",
                "type",
                "typeof",
                "undefined",
                "var",
                "void",
                "while",
                "yield",
            ],
            Lang::C => &[
                "auto",
                "bool",
                "break",
                "case",
                "char",
                "class",
                "const",
                "continue",
                "default",
                "do",
                "double",
                "else",
                "enum",
                "extern",
                "false",
                "final",
                "float",
                "for",
                "if",
                "import",
                "inline",
                "int",
                "long",
                "namespace",
                "new",
                "null",
                "nullptr",
                "private",
                "protected",
                "public",
                "return",
                "short",
                "signed",
                "sizeof",
                "static",
                "struct",
                "switch",
                "template",
                "this",
                "throw",
                "true",
                "try",
                "typedef",
                "union",
                "unsigned",
                "using",
                "virtual",
                "void",
                "volatile",
                "while",
            ],
            Lang::Go => &[
                "break",
                "case",
                "chan",
                "const",
                "continue",
                "default",
                "defer",
                "else",
                "fallthrough",
                "false",
                "for",
                "func",
                "go",
                "goto",
                "if",
                "import",
                "interface",
                "map",
                "nil",
                "package",
                "range",
                "return",
                "select",
                "struct",
                "switch",
                "true",
                "type",
                "var",
            ],
            Lang::PostScript => &[
                "def",
                "bind",
                "exch",
                "dup",
                "pop",
                "roll",
                "index",
                "copy",
                "if",
                "ifelse",
                "for",
                "forall",
                "repeat",
                "loop",
                "exit",
                "exec",
                "begin",
                "end",
                "dict",
                "array",
                "gsave",
                "grestore",
                "moveto",
                "lineto",
                "curveto",
                "arc",
                "closepath",
                "newpath",
                "stroke",
                "fill",
                "clip",
                "show",
                "translate",
                "rotate",
                "scale",
                "setrgbcolor",
                "setgray",
                "setlinewidth",
                "findfont",
                "scalefont",
                "setfont",
                "add",
                "sub",
                "mul",
                "div",
                "neg",
                "true",
                "false",
                "null",
            ],
            Lang::Shell => &[
                "if", "then", "else", "elif", "fi", "for", "in", "do", "done", "while", "case",
                "esac", "function", "return", "export", "local", "echo", "cd", "set",
            ],
            Lang::Json => &["true", "false", "null"],
            Lang::Plain => &[],
        }
    }

    /// What starts a comment that runs to the end of the line.
    fn line_comment(self) -> &'static [&'static str] {
        match self {
            Lang::Rust | Lang::JavaScript | Lang::C | Lang::Go => &["//"],
            Lang::Python | Lang::Shell => &["#"],
            Lang::PostScript => &["%"],
            Lang::Json | Lang::Plain => &[],
        }
    }

    fn block_comments(self) -> bool {
        matches!(self, Lang::Rust | Lang::JavaScript | Lang::C | Lang::Go)
    }
}

/// One line as runs of text, each with a kind: `plain` (space and
/// anything unclassified), `keyword`, `string`, `number`, `comment`,
/// `type`, `call`, `name`, `punct`.
pub type Line = Vec<(String, &'static str)>;

/// Split `source` into lines of classified runs. Concatenating a line's
/// runs gives the line back exactly.
/// The runs of a line with no character split between two of them: where a
/// run starts inside a grapheme cluster (a combining accent after a letter
/// of another kind, the rest of a flag, a skin tone, a ZWJ sequence), the
/// part that finishes the cluster moves to the run before it. A code block
/// then reveals, counts and colours the cluster as the one character it is.
fn whole_graphemes(runs: Line) -> Line {
    use unicode_segmentation::UnicodeSegmentation;
    let line: String = runs.iter().map(|(t, _)| t.as_str()).collect();
    if line.is_ascii() {
        return runs;
    }
    let bounds: std::collections::BTreeSet<usize> = line
        .grapheme_indices(true)
        .map(|(k, _)| k)
        .chain([line.len()])
        .collect();
    let mut out: Line = Vec::new();
    let mut at = 0;
    for (mut text, kind) in runs {
        let start = at;
        at += text.len();
        if !bounds.contains(&start)
            && let Some(prev) = out.last_mut()
        {
            let next = bounds.range(start..).next().copied().unwrap_or(line.len());
            let cut = (next - start).min(text.len());
            prev.0.push_str(&text[..cut]);
            text = text[cut..].to_string();
        }
        if !text.is_empty() {
            out.push((text, kind));
        }
    }
    out
}

pub fn tokenize(source: &str, lang: Lang) -> Vec<Line> {
    let mut out = Vec::new();
    let mut in_block = false;
    let source = source.strip_suffix('\n').unwrap_or(source);
    for line in source.split('\n') {
        let chars: Vec<char> = line.chars().collect();
        let mut runs: Line = Vec::new();
        let mut push = |text: String, kind: &'static str| {
            if text.is_empty() {
                return;
            }
            match runs.last_mut() {
                Some((t, k)) if *k == kind => t.push_str(&text),
                _ => runs.push((text, kind)),
            }
        };
        let mut k = 0;
        let starts = |k: usize, pat: &str| {
            let p: Vec<char> = pat.chars().collect();
            chars.len() >= k + p.len() && chars[k..k + p.len()] == p[..]
        };
        while k < chars.len() {
            let c = chars[k];
            if in_block {
                let mut end = k;
                while end < chars.len() && !starts(end, "*/") {
                    end += 1;
                }
                if end < chars.len() {
                    end += 2;
                    in_block = false;
                }
                push(chars[k..end].iter().collect(), "comment");
                k = end;
            } else if lang == Lang::Plain {
                push(chars[k..].iter().collect(), "plain");
                k = chars.len();
            } else if lang.block_comments() && starts(k, "/*") {
                push("/*".into(), "comment");
                k += 2;
                in_block = true;
            } else if lang.line_comment().iter().any(|p| starts(k, p)) {
                push(chars[k..].iter().collect(), "comment");
                k = chars.len();
            } else if c.is_whitespace() {
                let mut end = k;
                while end < chars.len() && chars[end].is_whitespace() {
                    end += 1;
                }
                push(chars[k..end].iter().collect(), "plain");
                k = end;
            } else if c == '"'
                || (c == '\'' && lang != Lang::Rust && lang != Lang::PostScript)
                || (c == '`' && lang == Lang::JavaScript)
                || (c == '(' && lang == Lang::PostScript)
            {
                // A string, to its closing quote on this line.
                let close = if c == '(' { ')' } else { c };
                let mut end = k + 1;
                let mut depth = 1;
                while end < chars.len() {
                    if chars[end] == '\\' {
                        end += 2;
                        continue;
                    }
                    if c == '(' && chars[end] == '(' {
                        depth += 1;
                    } else if chars[end] == close {
                        depth -= 1;
                        if depth == 0 || c != '(' {
                            end += 1;
                            break;
                        }
                    }
                    end += 1;
                }
                let end = end.min(chars.len());
                push(chars[k..end].iter().collect(), "string");
                k = end;
            } else if c.is_ascii_digit()
                || (c == '.' && chars.get(k + 1).is_some_and(char::is_ascii_digit))
            {
                let mut end = k;
                while end < chars.len()
                    && (chars[end].is_ascii_alphanumeric()
                        || chars[end] == '.'
                        || chars[end] == '_')
                {
                    end += 1;
                }
                push(chars[k..end].iter().collect(), "number");
                k = end;
            } else if c.is_alphabetic() || c == '_' || (c == '/' && lang == Lang::PostScript) {
                let mut end = k + 1;
                let word_char = |ch: char| {
                    ch.is_alphanumeric()
                        || ch == '_'
                        || (lang == Lang::PostScript && (ch == '-' || ch == '.'))
                        || (lang == Lang::Shell && ch == '-')
                };
                while end < chars.len() && word_char(chars[end]) {
                    end += 1;
                }
                let word: String = chars[k..end].iter().collect();
                let kind = if lang.keywords().contains(&word.as_str()) {
                    "keyword"
                } else if lang == Lang::PostScript {
                    if c == '/' { "type" } else { "name" }
                } else if lang == Lang::Json {
                    "name"
                } else if chars.get(end) == Some(&'(')
                    || (lang == Lang::Rust && chars.get(end) == Some(&'!'))
                {
                    "call"
                } else if c.is_uppercase() {
                    "type"
                } else {
                    "name"
                };
                push(word, kind);
                k = end;
            } else {
                push(c.to_string(), "punct");
                k += 1;
            }
        }
        // A JSON key is a string followed by a colon: colour it as a name.
        if lang == Lang::Json {
            for j in 0..runs.len() {
                if runs[j].1 == "string"
                    && runs[j + 1..]
                        .iter()
                        .find(|r| r.1 != "plain")
                        .is_some_and(|r| r.0.starts_with(':'))
                {
                    runs[j].1 = "type";
                }
            }
        }
        out.push(whole_graphemes(runs));
    }
    out
}

/// One row of a line diff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffRow {
    /// `same`, `del` (only in the old text) or `add` (only in the new).
    pub kind: &'static str,
    pub text: String,
    /// The line's index in the old text, or -1.
    pub old: i64,
    /// The line's index in the new text, or -1.
    pub new: i64,
}

/// The rows that turn `old` into `new`, by longest common subsequence of
/// lines. Deletions come before the additions that replace them.
pub fn diff_lines(old: &str, new: &str) -> Vec<DiffRow> {
    let split = |s: &str| -> Vec<String> {
        let s = s.strip_suffix('\n').unwrap_or(s);
        if s.is_empty() {
            Vec::new()
        } else {
            s.split('\n').map(str::to_string).collect()
        }
    };
    let (a, b) = (split(old), split(new));
    let (n, m) = (a.len(), b.len());
    // lcs[i][j]: the LCS length of a[i..] and b[j..].
    let mut lcs = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            out.push(DiffRow {
                kind: "same",
                text: a[i].clone(),
                old: i as i64,
                new: j as i64,
            });
            i += 1;
            j += 1;
        } else if i < n && (j == m || lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.push(DiffRow {
                kind: "del",
                text: a[i].clone(),
                old: i as i64,
                new: -1,
            });
            i += 1;
        } else {
            out.push(DiffRow {
                kind: "add",
                text: b[j].clone(),
                old: -1,
                new: j as i64,
            });
            j += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(line: &Line) -> Vec<(&str, &str)> {
        line.iter()
            .filter(|r| r.1 != "plain")
            .map(|r| (r.0.as_str(), r.1))
            .collect()
    }

    #[test]
    fn rust_is_classified_and_lines_round_trip() {
        let src = "fn main() {\n    let x = Vec::new(); // go\n    println!(\"hi {}\", 3.5);\n}\n";
        let lines = tokenize(src, Lang::Rust);
        assert_eq!(lines.len(), 4);
        for (line, text) in lines.iter().zip(src.lines()) {
            let joined: String = line.iter().map(|r| r.0.as_str()).collect();
            assert_eq!(joined, text);
        }
        assert_eq!(kinds(&lines[0])[..2], [("fn", "keyword"), ("main", "call")]);
        assert!(kinds(&lines[1]).contains(&("Vec", "type")));
        assert!(kinds(&lines[1]).contains(&("// go", "comment")));
        assert!(kinds(&lines[2]).contains(&("println", "call")));
        assert!(kinds(&lines[2]).contains(&("\"hi {}\"", "string")));
        assert!(kinds(&lines[2]).contains(&("3.5", "number")));
    }

    #[test]
    fn block_comments_span_lines_and_other_languages_have_their_own_rules() {
        let lines = tokenize("a /* one\ntwo */ b", Lang::C);
        assert_eq!(kinds(&lines[0]), [("a", "name"), ("/* one", "comment")]);
        assert_eq!(kinds(&lines[1]), [("two */", "comment"), ("b", "name")]);
        let py = tokenize("def f(x):  # note\n    return 'a'", Lang::Python);
        assert_eq!(kinds(&py[0])[0], ("def", "keyword"));
        assert_eq!(kinds(&py[0]).last().unwrap(), &("# note", "comment"));
        assert!(kinds(&py[1]).contains(&("'a'", "string")));
        let ps = tokenize("/scene { (a (b) c) show } def % x", Lang::PostScript);
        assert_eq!(kinds(&ps[0])[0], ("/scene", "type"));
        assert!(kinds(&ps[0]).contains(&("(a (b) c)", "string")));
        assert!(kinds(&ps[0]).contains(&("def", "keyword")));
        let js = tokenize("{\"a\": 1}", Lang::Json);
        assert!(kinds(&js[0]).contains(&("\"a\"", "type")));
        assert_eq!(
            tokenize("x = 1", Lang::Plain)[0],
            vec![("x = 1".to_string(), "plain")]
        );
        assert_eq!(tokenize("a\n\nb", Lang::Plain).len(), 3);
    }

    #[test]
    fn diff_finds_the_changed_lines() {
        let rows = diff_lines("a\nb\nc\nd", "a\nB\nc\nd\ne");
        let got: Vec<(&str, &str)> = rows.iter().map(|r| (r.kind, r.text.as_str())).collect();
        assert_eq!(
            got,
            [
                ("same", "a"),
                ("del", "b"),
                ("add", "B"),
                ("same", "c"),
                ("same", "d"),
                ("add", "e")
            ]
        );
        assert_eq!((rows[2].old, rows[2].new), (-1, 1));
        assert_eq!((rows[3].old, rows[3].new), (2, 2));
        assert!(diff_lines("", "").is_empty());
        assert_eq!(diff_lines("x", "x").len(), 1);
    }
}
