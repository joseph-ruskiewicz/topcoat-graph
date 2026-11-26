//! Pieces the gallery pages share.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

/// The code of the example `name` in a page's `source`: everything between a
/// `// example: <name>` line and the next `// end` comment, which may close the last
/// line of the example. Empty when there is no such example.
///
/// Pages pass their own source, from `include_str!`, so the code shown next to a chart
/// is the code that drew it.
pub fn snippet(source: &'static str, name: &str) -> &'static str {
    let marker = format!("// example: {name}");
    let mut start = None;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        offset += line.len();
        if line.trim() == marker {
            start = Some(offset);
            break;
        }
    }
    let Some(start) = start else { return "" };
    match source[start..].find("// end") {
        Some(len) => &source[start..start + len],
        None => "",
    }
}

/// `code` without the indentation every line shares, and without trailing blank lines.
fn dedent(code: &str) -> String {
    let indent = code
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    code.lines()
        .map(|line| line.get(indent..).unwrap_or("").trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

/// One example on a gallery page: a heading, a line about it, the chart in a card, and
/// the code that drew it.
#[component]
pub async fn example(
    cx: &Cx,
    /// The example's heading.
    title: &str,
    /// What the example shows.
    #[default]
    note: &str,
    /// The chart's code, from [`snippet`].
    code: &'static str,
    /// The chart.
    #[default]
    child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <section class="flex flex-col gap-3">
            <div>
                <h2 class="text-base font-medium">(title)</h2>
                if !note.is_empty() {
                    <p class="text-sm text-muted-foreground">(note)</p>
                }
            </div>
            <div class="flex flex-col rounded-lg border bg-card">
                <div class="p-4">(child)</div>
                <details class="group border-t">
                    <summary
                        class="cursor-pointer px-4 py-2 text-sm text-muted-foreground select-none hover:text-foreground"
                    >
                        "Code"
                    </summary>
                    <pre class="overflow-x-auto border-t px-4 py-3 text-xs leading-5">
                        <code>(dedent(code))</code>
                    </pre>
                </details>
            </div>
        </section>
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str =
        "a\n    // example: one\n        call(\n            x\n        )\n    // end\nb\n";

    #[test]
    fn a_snippet_is_the_code_between_its_markers() {
        assert_eq!(
            snippet(SOURCE, "one"),
            "        call(\n            x\n        )\n    "
        );
        // The end marker can also close the example's last line.
        assert_eq!(
            snippet("// example: two\n    f(x) // end\n", "two"),
            "    f(x) "
        );
        assert_eq!(dedent(snippet(SOURCE, "one")), "call(\n    x\n)");
    }

    #[test]
    fn an_unknown_or_unclosed_example_has_no_code() {
        assert_eq!(snippet(SOURCE, "two"), "");
        assert_eq!(snippet("// example: open\nx\n", "open"), "");
    }
}
