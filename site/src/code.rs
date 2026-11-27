//! Code panels for the docs: a bordered block with a language label, a copy button and
//! server-side highlighting.
//!
//! The docs only show Rust, TOML, CSS and shell, so the highlighter knows those four.
//! It is a lexer, not a parser: it never fails, and whatever it doesn't recognize stays
//! plain text. The token colors follow topcoat-prompt's code block, so the two sites
//! look alike.

use topcoat::{
    Result,
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    runtime::{Event, signal},
    view::{Attributes, StaticClass, Unescaped, View, attributes, class, component, view},
};

use crate::components::{
    button::{ButtonSize, ButtonVariant, button},
    tooltip::{tooltip, tooltip_content},
};

/// The token colors, as classes for the element around highlighted code.
///
/// Each color mixes a hue with the card's text color, so one rule is darker on a light
/// card and lighter on a dark one, and follows the theme without tokens of its own.
pub const TOKEN_COLORS: StaticClass = class!(
    "[&_.tk-comment]:[color:var(--muted-foreground)]",
    "[&_.tk-meta]:[color:var(--muted-foreground)]",
    "[&_.tk-keyword]:[color:color-mix(in_oklab,oklch(0.6_0.2_15)_62%,var(--card-foreground))]",
    "[&_.tk-string]:[color:color-mix(in_oklab,oklch(0.58_0.14_245)_62%,var(--card-foreground))]",
    "[&_.tk-number]:[color:color-mix(in_oklab,oklch(0.6_0.17_255)_70%,var(--card-foreground))]",
    "[&_.tk-property]:[color:color-mix(in_oklab,oklch(0.6_0.17_255)_70%,var(--card-foreground))]",
    "[&_.tk-function]:[color:color-mix(in_oklab,oklch(0.6_0.19_300)_66%,var(--card-foreground))]",
    "[&_.tk-type]:[color:color-mix(in_oklab,oklch(0.64_0.15_55)_70%,var(--card-foreground))]",
    "[&_.tk-tag]:[color:color-mix(in_oklab,oklch(0.6_0.15_150)_66%,var(--card-foreground))]",
);

/// A block of code with a header naming its language and a copy button. `code` is
/// highlighted when `language` is one [`highlight`] knows, and shown as plain text
/// otherwise. Long lines scroll instead of wrapping.
#[component]
pub async fn code_block(
    cx: &Cx,
    /// The language: `rust`, `toml`, `css` or `sh`.
    language: &str,
    /// The code.
    code: &str,
    /// Extra attributes for the outer `<div>` element.
    #[default]
    mut attrs: Attributes,
) -> Result<impl View> {
    let highlighted = highlight(language, code);

    Ok(view! {
        <div
            data-slot="code-block"
            class=(class!(
                "flex w-full min-w-0 flex-col overflow-hidden rounded-lg border border-border",
                "bg-card text-card-foreground",
                attrs.remove("class"),
            ))
            (attrs)
        >
            <div
                class="flex h-10 items-center justify-between gap-2 border-b border-border bg-foreground/5 pr-0.5 pl-4 text-xs text-muted-foreground"
            >
                <span class="truncate font-mono lowercase">(language)</span>
                copy_button(text: code, label: "Copy code")
            </div>
            <pre class="overflow-x-auto p-4 font-mono text-sm leading-relaxed">
                <code class=(TOKEN_COLORS)>
                    match highlighted {
                        // The highlighter escapes the code: see `highlight`.
                        Some(html) => (Unescaped::new_unchecked(html)),
                        None => (code),
                    }
                </code>
            </pre>
        </div>
    })
}

/// A ghost icon button that copies `text` to the clipboard.
///
/// After a copy, the icon turns into a check mark and the tooltip reads "Copied" for
/// two seconds. Copying needs the runtime and a secure context (HTTPS or `localhost`);
/// without them the button renders but does nothing.
#[component]
pub async fn copy_button(
    cx: &Cx,
    /// The text written to the clipboard.
    #[into]
    text: String,
    /// The tooltip and accessible name of the button.
    #[into]
    label: String,
    /// Extra attributes for the `<button>` element.
    #[default]
    attrs: Attributes,
) -> Result<impl View> {
    let copied = signal(cx, || false);

    Ok(view! {
        tooltip(
            button(
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Icon,
                attrs: attributes! {
                    type="button"
                    :aria-label=$(if copied.get() {
                        "Copied".to_owned()
                    } else {
                        label.clone()
                    })
                    @click=$(async |_e: Event| {
                        // The clipboard and timers are outside the expression
                        // vocabulary. Handlers only run in the browser, so the Rust
                        // fallbacks exist for type checking.
                        let written = raw!(
                            "cx.hydrate(await navigator.clipboard.writeText(${text}.toString()).then(() => true, () => false))",
                            true,
                        );
                        copied.set(written);
                        raw!(
                            "await new Promise((resolve) => setTimeout(resolve, 2000));",
                            (),
                        );
                        copied.set(false);
                    })
                    (attrs)
                },
                <span class="contents" :hidden=$(copied.get())>
                    icon(data: iconify_icon!("lucide:copy"))
                </span>
                <span class="contents" :hidden=$(!copied.get())>
                    icon(data: iconify_icon!("lucide:check"))
                </span>
            )
            tooltip_content(
                $(if copied.get() { "Copied".to_owned() } else { label.clone() })
            )
        )
    })
}

/// Highlights `code` as `language` (`rust`, `toml`, `css`, or `sh`), returning HTML, or
/// `None` for any other language.
///
/// The output is the code with `&`, `<`, `>`, `"` and `'` escaped and its tokens wrapped
/// in `<span class="tk-*">` elements, so it is safe to render unescaped.
pub fn highlight(language: &str, code: &str) -> Option<String> {
    let mut out = Out(String::with_capacity(code.len() * 2), None);
    match language {
        "rust" | "rs" => rust(code, &mut out),
        "toml" => toml(code, &mut out),
        "css" => css(code, &mut out),
        "sh" | "shell" | "bash" => shell(code, &mut out),
        _ => return None,
    }
    Some(out.0)
}

/// The kind of a token, which is its class.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Comment,
    Meta,
    Keyword,
    String,
    Number,
    Property,
    Function,
    Type,
    Tag,
}

impl Kind {
    fn class(self) -> &'static str {
        match self {
            Self::Comment => "tk-comment",
            Self::Meta => "tk-meta",
            Self::Keyword => "tk-keyword",
            Self::String => "tk-string",
            Self::Number => "tk-number",
            Self::Property => "tk-property",
            Self::Function => "tk-function",
            Self::Type => "tk-type",
            Self::Tag => "tk-tag",
        }
    }
}

/// The highlighted HTML being written, and the kind of the span it ends with.
struct Out(String, Option<Kind>);

impl Out {
    /// Appends `text`, escaped, and wrapped in a span when it has a kind. Text of the
    /// same kind as the span before it joins that span.
    fn push(&mut self, kind: Option<Kind>, text: &str) {
        if text.is_empty() {
            return;
        }
        if kind.is_some() && kind == self.1 {
            self.0.truncate(self.0.len() - "</span>".len());
        } else if let Some(kind) = kind {
            self.0.push_str("<span class=\"");
            self.0.push_str(kind.class());
            self.0.push_str("\">");
        }
        self.1 = kind;
        for c in text.chars() {
            match c {
                '&' => self.0.push_str("&amp;"),
                '<' => self.0.push_str("&lt;"),
                '>' => self.0.push_str("&gt;"),
                '"' => self.0.push_str("&quot;"),
                '\'' => self.0.push_str("&#39;"),
                c => self.0.push(c),
            }
        }
        if kind.is_some() {
            self.0.push_str("</span>");
        }
    }
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "type", "unsafe", "use",
    "where", "while",
];

const RUST_TYPES: &[&str] = &[
    "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
    "i128", "isize", "f32", "f64",
];

/// The length in bytes of the leading run of `s` whose characters satisfy `keep`.
fn run_len(s: &str, keep: impl Fn(char) -> bool) -> usize {
    s.find(|c: char| !keep(c)).unwrap_or(s.len())
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The length of the string literal at the start of `s`, which opens with `"`. An
/// unclosed string runs to the end.
fn string_len(s: &str) -> usize {
    let mut escaped = false;
    for (index, c) in s.char_indices().skip(1) {
        match c {
            '\\' if !escaped => escaped = true,
            '"' if !escaped => return index + 1,
            _ => escaped = false,
        }
    }
    s.len()
}

/// The length of a Rust raw string, `r"…"` or `r#"…"#`, at the start of `s`.
fn raw_string_len(s: &str) -> Option<usize> {
    let rest = s.strip_prefix('r')?;
    let hashes = run_len(rest, |c| c == '#');
    if !rest[hashes..].starts_with('"') {
        return None;
    }
    let close = format!("\"{}", "#".repeat(hashes));
    let body = 1 + hashes + 1;
    Some(
        s[body..]
            .find(&close)
            .map_or(s.len(), |end| body + end + close.len()),
    )
}

/// The length of a Rust character literal at the start of `s`, such as `'a'` or
/// `'\n'`, or `None` for a lifetime such as `'_`.
fn char_len(s: &str) -> Option<usize> {
    let mut chars = s.char_indices().skip(1);
    let (_, first) = chars.next()?;
    if first == '\\' {
        let end = s[2..].find('\'')?;
        return Some(2 + end + 1);
    }
    let (index, second) = chars.next()?;
    (second == '\'').then_some(index + 1)
}

/// The length of the number at the start of `s`, with its suffix, such as `0.5` or
/// `12u32`. A `.` only continues it before a digit, so `1..=30` is two numbers.
fn number_len(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut end = 0;
    while end < bytes.len() {
        let b = bytes[end];
        let continues = b.is_ascii_alphanumeric()
            || b == b'_'
            || (b == b'.' && bytes.get(end + 1).is_some_and(u8::is_ascii_digit));
        if !continues {
            break;
        }
        end += 1;
    }
    end
}

/// The length of the bracketed text at the start of `s`, which opens with `open`,
/// through its matching `close`. Unbalanced text runs to the end.
fn bracketed_len(s: &str, open: char, close: char) -> usize {
    let mut depth = 0;
    let mut index = 0;
    while index < s.len() {
        let rest = &s[index..];
        let c = rest.chars().next().unwrap();
        if c == '"' {
            index += string_len(rest);
            continue;
        }
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return index + 1;
            }
        }
        index += c.len_utf8();
    }
    s.len()
}

fn rust(src: &str, out: &mut Out) {
    let mut i = 0;
    // The last character that wasn't whitespace, to tell a markup tag from a generic.
    let mut previous = ' ';
    while i < src.len() {
        let s = &src[i..];
        let c = s.chars().next().unwrap();
        let (kind, len) = if s.starts_with("//") {
            (Some(Kind::Comment), s.find('\n').unwrap_or(s.len()))
        } else if s.starts_with("/*") {
            (
                Some(Kind::Comment),
                s.find("*/").map_or(s.len(), |end| end + 2),
            )
        } else if s.starts_with("#[") || s.starts_with("#![") {
            let open = s.find('[').unwrap();
            (Some(Kind::Meta), open + bracketed_len(&s[open..], '[', ']'))
        } else if c == '"' {
            (Some(Kind::String), string_len(s))
        } else if let Some(len) = raw_string_len(s) {
            (Some(Kind::String), len)
        } else if c == '\'' {
            match char_len(s) {
                Some(len) => (Some(Kind::String), len),
                None => (Some(Kind::Keyword), 1 + run_len(&s[1..], is_word)),
            }
        } else if c.is_ascii_digit() {
            (Some(Kind::Number), number_len(s))
        } else if c == '<'
            && !is_word(previous)
            && s[1..].starts_with(|c: char| c.is_ascii_lowercase() || c == '/')
        {
            // A tag in `view!` markup, such as `<div` or `</span>`.
            let start = if s[1..].starts_with('/') { 2 } else { 1 };
            let name = run_len(&s[start..], |c| is_word(c) || c == '-');
            if name > 0 {
                out.push(None, &s[..start]);
                out.push(Some(Kind::Tag), &s[start..start + name]);
                previous = 'a';
                i += start + name;
                continue;
            }
            (None, 1)
        } else if is_word(c) {
            let len = run_len(s, is_word);
            let word = &s[..len];
            let after = &s[len..];
            let kind = if RUST_KEYWORDS.contains(&word) {
                Some(Kind::Keyword)
            } else if word == "true" || word == "false" {
                Some(Kind::Number)
            } else if RUST_TYPES.contains(&word) || word.starts_with(char::is_uppercase) {
                Some(Kind::Type)
            } else if after.starts_with('!') && !after.starts_with("!=") {
                // A macro: color the `!` with its name.
                out.push(Some(Kind::Function), &s[..len + 1]);
                previous = '!';
                i += len + 1;
                continue;
            } else if after.starts_with('(') || after.starts_with("::<") {
                Some(Kind::Function)
            } else {
                None
            };
            (kind, len)
        } else {
            (None, c.len_utf8())
        };
        out.push(kind, &s[..len]);
        if let Some(last) = s[..len].chars().rev().find(|c| !c.is_whitespace()) {
            previous = last;
        }
        i += len;
    }
}

fn toml(src: &str, out: &mut Out) {
    for line in src.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        out.push(None, indent);
        if trimmed.starts_with('#') {
            out.push(Some(Kind::Comment), trimmed);
        } else if trimmed.starts_with('[') {
            out.push(Some(Kind::Type), trimmed.trim_end());
            out.push(None, &trimmed[trimmed.trim_end().len()..]);
        } else if let Some(eq) = trimmed.find('=') {
            out.push(Some(Kind::Property), &trimmed[..eq]);
            out.push(None, "=");
            toml_value(&trimmed[eq + 1..], out);
        } else {
            toml_value(trimmed, out);
        }
    }
}

/// A TOML value or the rest of a line: strings, numbers, booleans and comments.
fn toml_value(src: &str, out: &mut Out) {
    let mut i = 0;
    while i < src.len() {
        let s = &src[i..];
        let c = s.chars().next().unwrap();
        let (kind, len) = if c == '"' {
            (Some(Kind::String), string_len(s))
        } else if c == '#' {
            (Some(Kind::Comment), s.find('\n').unwrap_or(s.len()))
        } else if c.is_ascii_digit() {
            (Some(Kind::Number), number_len(s))
        } else if s.starts_with("true") || s.starts_with("false") {
            (Some(Kind::Number), run_len(s, is_word))
        } else if is_word(c) {
            (None, run_len(s, is_word))
        } else {
            (None, c.len_utf8())
        };
        out.push(kind, &s[..len]);
        i += len;
    }
}

fn css(src: &str, out: &mut Out) {
    let mut i = 0;
    // Inside `{ … }`, words before a `:` are properties; outside, they are selectors.
    let mut depth = 0;
    while i < src.len() {
        let s = &src[i..];
        let c = s.chars().next().unwrap();
        let (kind, len) = if s.starts_with("/*") {
            (
                Some(Kind::Comment),
                s.find("*/").map_or(s.len(), |end| end + 2),
            )
        } else if c == '"' {
            (Some(Kind::String), string_len(s))
        } else if c == '{' || c == '}' {
            depth += if c == '{' { 1 } else { -1 };
            (None, 1)
        } else if depth == 0 && !c.is_whitespace() {
            (Some(Kind::Tag), run_len(s, |c| c != '{' && c != '\n'))
        } else if is_word(c) || c == '-' {
            let len = run_len(s, |c| is_word(c) || c == '-');
            let after = &s[len..];
            let kind = if after.starts_with(':') {
                Some(Kind::Property)
            } else if after.starts_with('(') {
                Some(Kind::Function)
            } else if c.is_ascii_digit()
                || (c == '-' && s[1..].starts_with(|c: char| c.is_ascii_digit()))
            {
                Some(Kind::Number)
            } else {
                None
            };
            (kind, len)
        } else if c == '.' && s[1..].starts_with(|c: char| c.is_ascii_digit()) {
            (Some(Kind::Number), 1 + run_len(&s[1..], is_word))
        } else {
            (None, c.len_utf8())
        };
        out.push(kind, &s[..len]);
        i += len;
    }
}

fn shell(src: &str, out: &mut Out) {
    for line in src.split_inclusive('\n') {
        let mut first = true;
        let mut i = 0;
        while i < line.len() {
            let s = &line[i..];
            let c = s.chars().next().unwrap();
            let (kind, len) = if c == '#' {
                (Some(Kind::Comment), s.trim_end_matches('\n').len())
            } else if c == '"' {
                (Some(Kind::String), string_len(s))
            } else if c.is_whitespace() {
                (None, run_len(s, char::is_whitespace))
            } else {
                let len = run_len(s, |c| !c.is_whitespace());
                let kind = if first {
                    Some(Kind::Function)
                } else if c == '-' {
                    Some(Kind::Property)
                } else {
                    None
                };
                first = false;
                (kind, len)
            };
            out.push(kind, &s[..len]);
            i += len;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::highlight;

    #[test]
    fn highlights_rust_and_escapes_it() {
        let html = highlight(
            "rust",
            "// A chart.\nlet x: Vec<f64> = vec![1.0, f64::NAN];\nlabel: \"<b>\" 'a'",
        )
        .unwrap();
        assert!(html.contains(r#"<span class="tk-comment">// A chart.</span>"#));
        assert!(html.contains(r#"<span class="tk-keyword">let</span>"#));
        assert!(
            html.contains(r#"<span class="tk-type">Vec</span>&lt;"#),
            "{html}"
        );
        assert!(html.contains(r#"<span class="tk-function">vec!</span>"#));
        assert!(html.contains(r#"<span class="tk-number">1.0</span>"#));
        assert!(html.contains(r#"<span class="tk-string">&quot;&lt;b&gt;&quot;</span>"#));
        assert!(html.contains(r#"<span class="tk-string">&#39;a&#39;</span>"#));
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn rust_tells_markup_tags_from_generics() {
        let html = highlight("rust", "<div class=\"a\">\n</div>\nOption<usize>").unwrap();
        assert!(
            html.contains(r#"&lt;<span class="tk-tag">div</span>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"&lt;/<span class="tk-tag">div</span>"#),
            "{html}"
        );
        assert!(
            !html.contains(r#"<span class="tk-tag">usize</span>"#),
            "{html}"
        );
    }

    #[test]
    fn ranges_and_lifetimes() {
        let html = highlight("rust", "(1..=30) Child<'_>").unwrap();
        assert!(
            html.contains(r#"<span class="tk-number">1</span>..="#),
            "{html}"
        );
        assert!(
            html.contains(r#"<span class="tk-keyword">&#39;_</span>"#),
            "{html}"
        );
    }

    #[test]
    fn highlights_toml_css_and_shell() {
        let toml = highlight(
            "toml",
            "[dependencies]\ntopcoat = { version = \"0.12\" } # x\n",
        )
        .unwrap();
        assert!(
            toml.contains(r#"<span class="tk-type">[dependencies]</span>"#),
            "{toml}"
        );
        assert!(
            toml.contains(r#"<span class="tk-property">topcoat </span>"#),
            "{toml}"
        );
        assert!(toml.contains(r#"<span class="tk-comment"># x"#), "{toml}");

        let css = highlight("css", ":root {\n  --chart-1: oklch(0.6 0.1 41);\n}").unwrap();
        assert!(
            css.contains(r#"<span class="tk-tag">:root </span>"#),
            "{css}"
        );
        assert!(
            css.contains(r#"<span class="tk-property">--chart-1</span>"#),
            "{css}"
        );
        assert!(
            css.contains(r#"<span class="tk-function">oklch</span>"#),
            "{css}"
        );

        let sh = highlight("sh", "cargo add topcoat --features ui\n").unwrap();
        assert!(
            sh.contains(r#"<span class="tk-function">cargo</span>"#),
            "{sh}"
        );
        assert!(
            sh.contains(r#"<span class="tk-property">--features</span>"#),
            "{sh}"
        );
    }

    #[test]
    fn unknown_languages_are_not_highlighted() {
        assert_eq!(highlight("python", "x = 1"), None);
    }
}
