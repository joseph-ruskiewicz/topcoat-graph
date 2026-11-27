//! The playgrounds at the top of each chart's page.
//!
//! A playground is a shard: it keeps one signal per control, draws the chart from their
//! values, and shows the code for the chart on screen. A control sets its signal and the
//! shard renders again on the server. What the browser sends back is untrusted, so each
//! value is held to its allowed range before it is used.

use topcoat::{
    Result,
    context::Cx,
    view::{Attributes, Child, View, class, component, view},
};

use crate::components::chart::ChartSeries;

pub mod area;
pub mod bar;
pub mod line;
pub mod pie;

/// The categories of the bar, line and area playgrounds.
pub const WEEKS: [&str; 6] = ["Week 1", "Week 2", "Week 3", "Week 4", "Week 5", "Week 6"];

/// The series of the bar, line and area playgrounds, in the order they are added.
pub const GAMES: [(&str, [f64; 6]); 3] = [
    ("Wins", [3.0, 5.0, 4.0, 7.0, 6.0, 8.0]),
    ("Losses", [4.0, 3.0, 4.0, 2.0, 3.0, 2.0]),
    ("Draws", [1.0, 0.0, 2.0, 1.0, 1.0, 0.0]),
];

/// How many series to draw: what the browser sent, held to 1 to 3.
pub fn series_count(sent: usize) -> usize {
    sent.clamp(1, GAMES.len())
}

/// The first `count` series.
pub fn game_series(count: usize) -> Vec<ChartSeries> {
    GAMES
        .iter()
        .take(series_count(count))
        .map(|(name, values)| ChartSeries::new(*name, *values))
        .collect()
}

/// A list of numbers as Rust source: `3.0, 5.0`.
fn numbers(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{value:.1}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The `categories` prop as source.
pub fn weeks_prop() -> String {
    let quoted: Vec<String> = WEEKS.iter().map(|week| format!("\"{week}\"")).collect();
    format!("categories: labels([{}])", quoted.join(", "))
}

/// The `series` prop as source, with the first `count` series.
pub fn series_prop(count: usize) -> String {
    let mut out = String::from("series: vec![\n");
    for (name, values) in GAMES.iter().take(series_count(count)) {
        out += &format!("    ChartSeries::new(\"{name}\", [{}]),\n", numbers(values));
    }
    out + "]"
}

/// The source of a chart call: its imports, then the call in a `view!`.
///
/// `graph` names what the call uses from the `topcoat_graph` crate and `uses` what it
/// uses from the app's components. Each prop may span lines. `children` follow the
/// props.
pub fn source(
    graph: &[&str],
    component: &str,
    uses: &[&str],
    props: &[String],
    children: &[String],
) -> String {
    let indent = |text: &str| -> String {
        text.lines()
            .map(|line| format!("        {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut out = String::new();
    match graph {
        [] => {}
        [one] => out += &format!("use topcoat_graph::{one};\n\n"),
        many => out += &format!("use topcoat_graph::{{{}}};\n\n", many.join(", ")),
    }
    let mut uses = uses.to_vec();
    uses.sort_unstable();
    match uses.as_slice() {
        [one] => out += &format!("use crate::components::{one};\n\n"),
        many => {
            out += "use crate::components::{\n";
            for item in many {
                out += &format!("    {item},\n");
            }
            out += "};\n\n";
        }
    }
    out += &format!("view! {{\n    {component}(\n");
    out += &props
        .iter()
        .map(|prop| indent(prop))
        .collect::<Vec<_>>()
        .join(",\n");
    if !children.is_empty() {
        out += ",\n";
        out += &children
            .iter()
            .map(|child| indent(child))
            .collect::<Vec<_>>()
            .join("\n");
    }
    out + "\n    )\n}\n"
}

/// The component imports of a chart that takes `ChartSeries`.
pub fn series_uses(component: &str) -> [String; 2] {
    [
        format!("{component}::{component}"),
        String::from("chart::{ChartSeries, labels}"),
    ]
}

const CHOICE: topcoat::view::StaticClass = class!(
    "rounded-md border border-border px-2.5 py-1 text-xs font-medium text-muted-foreground",
    "hover:bg-foreground/5 hover:text-foreground",
    "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring",
    "aria-pressed:border-foreground aria-pressed:bg-foreground aria-pressed:text-background",
);

/// One choice of a control: a button that is pressed when its choice is the current one.
/// Pass the click handler in `attrs`.
#[component]
pub async fn choice(
    cx: &Cx,
    label: &str,
    pressed: bool,
    #[default] mut attrs: Attributes,
) -> Result<impl View> {
    Ok(view! {
        <button
            type="button"
            aria-pressed=(if pressed { "true" } else { "false" })
            class=(class!(CHOICE, attrs.remove("class")))
            (attrs)
        >
            (label)
        </button>
    })
}

/// A named group of choices.
#[component]
pub async fn control(cx: &Cx, label: &str, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground">(label)</span>
            <div role="group" aria-label=(label) class="flex gap-1">(child)</div>
        </div>
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_series_count_from_the_browser_is_held_to_the_data() {
        assert_eq!(series_count(0), 1);
        assert_eq!(series_count(2), 2);
        assert_eq!(series_count(usize::MAX), 3);
        assert_eq!(game_series(usize::MAX).len(), 3);
        assert_eq!(game_series(0).len(), 1);
    }

    #[test]
    fn source_lists_imports_then_the_call() {
        let props = [
            String::from("label: \"A\""),
            String::from("series: vec![\n    1,\n]"),
        ];
        assert_eq!(
            source(
                &[],
                "bar_chart",
                &["chart::{ChartSeries, labels}", "bar_chart::bar_chart"],
                &props,
                &[]
            ),
            "use crate::components::{\n    bar_chart::bar_chart,\n    chart::{ChartSeries, labels},\n};\n\nview! {\n    bar_chart(\n        label: \"A\",\n        series: vec![\n            1,\n        ]\n    )\n}\n"
        );
    }

    #[test]
    fn source_imports_from_the_crate_and_sorts_component_imports() {
        let props = [String::from("donut: true")];
        let children = [String::from("<span>\"32\"</span>")];
        assert_eq!(
            source(
                &["Curve"],
                "pie_chart",
                &["pie_chart::{PieSlice, pie_chart}"],
                &props,
                &children
            ),
            "use topcoat_graph::Curve;\n\nuse crate::components::pie_chart::{PieSlice, pie_chart};\n\nview! {\n    pie_chart(\n        donut: true,\n        <span>\"32\"</span>\n    )\n}\n"
        );
        assert!(
            source(
                &["BarMode", "Orientation"],
                "bar_chart",
                &["x"],
                &props,
                &[]
            )
            .starts_with("use topcoat_graph::{BarMode, Orientation};\n\n")
        );
    }
}
