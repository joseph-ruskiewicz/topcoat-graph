//! The guide pages under `/docs/`: Introduction, Installation, Theming and the
//! geometry crate. Component pages are in `docs/slug.rs`.

use topcoat::{
    Result,
    context::Cx,
    view::{View, class, component, view},
};

use crate::{
    GITHUB_URL, api,
    catalog::{self, GEOMETRY_EXAMPLE, Group, THEMING_EXAMPLES},
    components::chart::CHART_VARS,
    shell::{code_panel, command, example_section, item_entry, page_header, prose},
};

/// The README, which some pages quote so the two say the same thing.
const README: &str = include_str!("../../README.md");

/// The first fenced block in the README that opens with `fence`, such as "```css".
fn readme_block(fence: &str) -> &'static str {
    let open = format!("{fence}\n");
    let start = README
        .find(&open)
        .map_or(README.len(), |at| at + open.len());
    let end = README[start..].find("```").map_or(start, |len| start + len);
    &README[start..end]
}

#[component]
pub async fn introduction(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        page_header(
            title: "Introduction",
            description: "Bar, line, area and pie charts for Topcoat apps, drawn on the server."
        )
        prose(
            <p>
                "topcoat-graph draws charts for "
                <a href="https://github.com/tokio-rs/topcoat">"Topcoat"</a>
                " apps. Charts render on the server as SVG with HTML labels, so a page \
                 shows them in full before any script loads. With the Topcoat runtime on \
                 the page, pointing at a category or tabbing to it shows its values in a \
                 tooltip."
            </p>
            <p>
                "It's a community project. It isn't part of Topcoat and isn't maintained \
                 by tokio-rs."
            </p>
            <h2>"Two parts"</h2>
            <p>
                "The first part is a "
                <code>"topcoat ui"</code>
                " component registry. "
                <code>"topcoat ui add"</code>
                " copies a chart into your app as a Rust source file, the same way it \
                 installs Topcoat's built-in components. From then on it's your code: \
                 restyle it, rearrange it, or delete what you don't use."
            </p>
            <p>
                "The second part is a small crate of chart geometry: scales, ticks, \
                 stacking, line and area paths, and pie slices. The components call it to \
                 work out where everything goes. It's plain Rust with no dependencies, and \
                 fixes to it arrive with "
                <code>"cargo update"</code>
                ". You can also "
                <a href="/docs/geometry">"use it on its own"</a>
                " to draw charts your own way."
            </p>
            <h2>"How a chart is drawn"</h2>
            <ul>
                <li>
                    "The marks are an SVG in a "
                    <code>"0 0 100 100"</code>
                    " box, stretched to fill the chart. Lines keep their width at any size."
                </li>
                <li>
                    "Axis labels, the legend and tooltips are HTML laid over the plot, so \
                     text stays sharp and wraps or truncates like any other text."
                </li>
                <li>
                    "Colors come from your theme's tokens, in light and dark mode. See "
                    <a href="/docs/theming">"Theming"</a>
                    "."
                </li>
                <li>
                    "Without the runtime, everything still renders: marks, axes, labels \
                     and the data table. Only the tooltips and the hover highlight are \
                     missing."
                </li>
            </ul>
            <h2>"Accessibility"</h2>
            <ul>
                <li>
                    "Each chart is a figure named by its "
                    <code>"label"</code>
                    " prop."
                </li>
                <li>
                    "Its data is also in a visually hidden table, so the numbers are there \
                     without sight or hover. Bars, lines and axis labels are hidden from \
                     screen readers. Pie slices are named and take focus."
                </li>
                <li>
                    "Each category, or each pie slice, has a named hover target that \
                     takes keyboard focus. Tab moves through them and shows each tooltip. \
                     A chart with 30 categories adds 30 tab stops."
                </li>
                <li>
                    "Every word a chart shows or announces comes from a prop, so you can \
                     pass localized strings. Numbers are written with a decimal point and \
                     no digit grouping. To change that in tooltips and the data table, \
                     edit "
                    <code>"category_tooltips"</code>
                    ", "
                    <code>"chart_table"</code>
                    " and the tooltip row in "
                    <code>"pie_chart"</code>
                    " in your copy. Axis tick labels come from the geometry crate, so \
                     your copy can't change them."
                </li>
            </ul>
            <h2>"What's here"</h2>
            <p>"Four charts:"</p>
            <ul>
                for doc in catalog::COMPONENTS
                    .iter()
                    .filter(|doc| doc.group == Group::Chart) {
                    <li>
                        <a href=(format!("/docs/{}", doc.name))>(doc.title)</a>
                        ": "
                        (doc.description)
                    </li>
                }
            </ul>
            <p>
                "And the parts they're built from, which you can use for charts of your \
                 own:"
            </p>
            <ul>
                for doc in catalog::COMPONENTS
                    .iter()
                    .filter(|doc| doc.group == Group::Part) {
                    <li>
                        <a href=(format!("/docs/{}", doc.name))>(doc.title)</a>
                        ": "
                        (doc.description)
                    </li>
                }
            </ul>
            <p>
                "The source is on "
                <a href=(GITHUB_URL)>"GitHub"</a>
                ", under the MIT license."
            </p>
        )
    })
}

/// The commands the Installation page shows, in order. A test checks that the README
/// shows each of them too.
pub const INSTALL_COMMANDS: &[&str] = &[
    "cargo add topcoat --features ui,tailwind",
    "cargo add topcoat-graph --git https://github.com/joseph-ruskiewicz/topcoat-graph",
    "topcoat ui init --theme neutral",
    "topcoat ui add bar_chart --registry topcoat-graph",
    "topcoat ui add --all --registry topcoat-graph",
    "cargo update -p topcoat-graph",
    "topcoat ui add --all --registry topcoat-graph --overwrite",
];

/// The `build.rs` the Installation page shows.
const BUILD_RS: &str = r#"fn main() {
    topcoat::tailwind::BuildConfig::new()
        .input("styles.css")
        .render()
        .unwrap();
}
"#;

/// The build dependency the Installation page shows.
const BUILD_DEPENDENCIES: &str = r#"[build-dependencies]
topcoat = { version = "0.12", default-features = false, features = ["tailwind"] }
"#;

/// The router the Installation page shows, for tooltips.
const ROUTER: &str = r#"use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt, module_router},
    runtime::RuntimeLayer,
};

fn router() -> Router {
    module_router!()
        .assets(AssetBundle::load().unwrap())
        .discover()
        .layer(RuntimeLayer::new())
        .build()
}
"#;

/// The document head the Installation page shows, for tooltips.
const HEAD: &str = r#"view! {
    <head>
        topcoat::runtime::script()
        <link rel="stylesheet" href=(tailwind::stylesheet!())>
    </head>
}
"#;

/// The imports the Installation page shows.
const IMPORTS: &str = r#"use crate::components::{
    bar_chart::bar_chart,
    chart::{ChartSeries, labels},
};
"#;

/// A keyed loop of charts, as the Installation page shows it.
const LOOP: &str = r#"view! {
    #[key(deck.id)]
    for deck in decks {
        bar_chart(
            label: deck.name.clone(),
            category_header: "Cost",
            categories: labels(["0", "1", "2", "3", "4", "5", "6+"]),
            series: vec![ChartSeries::new("Cards", deck.curve.clone())]
        )
    }
}
"#;

#[component]
pub async fn installation(cx: &Cx) -> Result<impl View> {
    let [topcoat, graph, init, add, add_all, update, reinstall] = INSTALL_COMMANDS else {
        unreachable!("the commands are listed above")
    };

    Ok(view! {
        page_header(
            title: "Installation",
            description: "Add the crate, set up a theme, install charts, and turn on tooltips."
        )
        prose(
            <h2>"1. Add the crates"</h2>
            <p>
                "Your app needs Topcoat with the "
                <code>"ui"</code>
                " and "
                <code>"tailwind"</code>
                " features. The "
                <code>"runtime"</code>
                " feature, which tooltips use, is on by default."
            </p>
            command(text: topcoat)
            <p>
                "Then add topcoat-graph. It isn't on crates.io yet, so add it from GitHub:"
            </p>
            command(text: graph)
            <p>
                <code>"topcoat ui"</code>
                " only offers registries that are direct dependencies of your app, which \
                 is why the crate goes in "
                <code>"[dependencies]"</code>
                ". Unlike a registry crate with no code, this one adds a little to your \
                 binary: the geometry the components call."
            </p>
            <h2>"2. Install a theme"</h2>
            <p>
                "The charts use the design tokens of a Topcoat theme. If your app doesn't \
                 have one yet, install the neutral theme. It writes "
                <code>"styles.css"</code>
                " and "
                <code>"components.toml"</code>
                "."
            </p>
            command(text: init)
            <p>
                "As with any "
                <code>"topcoat ui"</code>
                " app, "
                <code>"styles.css"</code>
                " is your Tailwind input. Render it in "
                <code>"build.rs"</code>
                ":"
            </p>
            code_panel(source: BUILD_RS)
            <p>
                "That needs "
                <code>"topcoat"</code>
                " in "
                <code>"[build-dependencies]"</code>
                " too, with the "
                <code>"tailwind"</code>
                " feature:"
            </p>
            code_panel(source: BUILD_DEPENDENCIES, language: Some("toml"))
            <h2>"3. Add charts"</h2>
            <p>
                "Install a chart by name. The parts it's built from, such as "
                <code>"chart"</code>
                " and "
                <code>"chart_axis"</code>
                ", come with it."
            </p>
            command(text: add)
            <p>"Or install everything at once:"</p>
            command(text: add_all)
            <p>
                "The sources land in "
                <code>"src/components/"</code>
                " and are declared in "
                <code>"src/components.rs"</code>
                ". Import them from there:"
            </p>
            code_panel(source: IMPORTS)
            <p>
                "Each chart page shows the props and an example of each option. "
                <a href="/docs/bar_chart">"Bar Chart"</a>
                " is a good place to start."
            </p>
            <h2>"4. Turn on tooltips"</h2>
            <p>
                "Tooltips need the runtime's browser script in the document head and a "
                <code>"RuntimeLayer"</code>
                " on the router, with the asset bundle that serves the script:"
            </p>
            code_panel(source: ROUTER)
            code_panel(source: HEAD)
            <p>
                "Without the runtime, charts still render in full: marks, axes, labels and \
                 the data table. Only the tooltips and the hover highlight are missing."
            </p>
            <h2>"Charts in a loop"</h2>
            <p>
                "Each chart keeps its hover state in a signal, and Topcoat needs to tell \
                 one chart's state from another's. When a "
                <code>"view!"</code>
                " loop renders a chart per item, key the loop, or the page fails with an \
                 \"ambiguous identity\" error:"
            </p>
            code_panel(source: LOOP)
            <h2>"Updating"</h2>
            <p>
                "To pick up a newer version, update the crate and reinstall the components \
                 together, so the geometry and the components match:"
            </p>
            command(text: update)
            command(text: reinstall)
            <p>
                <code>"--overwrite"</code>
                " replaces only the components you name, so name them all or use "
                <code>"--all"</code>
                ". It also replaces any changes you made to them. Commit first and \
                 reapply your changes from the diff."
            </p>
        )
    })
}

#[component]
pub async fn theming(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        page_header(
            title: "Theming",
            description: "Chart colors, per-series colors, sizes and dark mode."
        )
        prose(
            <h2>"Chart colors"</h2>
            <p>
                "Series take the colors "
                <code>"--chart-1"</code>
                " to "
                <code>"--chart-5"</code>
                " in turn. A sixth series starts over at "
                <code>"--chart-1"</code>
                ". These are the colors in the current theme:"
            </p>
            <div
                data-slot="swatches"
                class=(class!("my-6 grid grid-cols-5 gap-2 sm:gap-3", CHART_VARS))
            >
                for n in 1..=5 {
                    <div class="flex min-w-0 flex-col gap-1.5">
                        <span
                            class="h-12 rounded-md border border-border"
                            style=(format!(
                                "background:var(--chart-{n}, var(--chart-default-{n}))",
                            ))
                        ></span>
                        <code class="truncate text-xs">(format!("--chart-{n}"))</code>
                    </div>
                }
            </div>
            <p>
                "The neutral theme doesn't define these tokens, so the components carry \
                 defaults for light and dark mode. To set your own, add them to "
                <code>"styles.css"</code>
                " under "
                <code>":root"</code>
                " and "
                <code>".dark"</code>
                ". These are the defaults:"
            </p>
            code_panel(source: readme_block("```css"), language: Some("css"))
            <p>
                "Text, grid lines and the tooltip surface use tokens the theme already \
                 has, such as "
                <code>"text-muted-foreground"</code>
                ", "
                <code>"border"</code>
                " and "
                <code>"bg-popover"</code>
                "."
            </p>
            <h2>"A color per series"</h2>
            <p>
                "To give one series a color of its own, pass any CSS color to "
                <code>"ChartSeries::color"</code>
                ", or to "
                <code>"PieSlice::color"</code>
                " for a slice. The color goes into a "
                <code>"style"</code>
                " attribute as written. Pass colors your app defines, and check a color a \
                 user chose before passing it on."
            </p>
            example_section(example: &THEMING_EXAMPLES[0])
            <h2>"Sizes"</h2>
            <p>
                "A chart is as wide as its container and 16:9 by default. Classes in "
                <code>"attrs"</code>
                " go on the chart's root element and win over the default shape. For a \
                 fixed height, pass "
                <code>"aspect-auto"</code>
                " with a height. A pie is square, centered and at most 18rem wide; pass a \
                 different "
                <code>"max-w-*"</code>
                " to change that."
            </p>
            example_section(example: &THEMING_EXAMPLES[1])
            <h2>"Dark mode"</h2>
            <p>
                "The theme switches to dark mode when an ancestor of the chart, usually "
                <code>"<html>"</code>
                ", has the "
                <code>"dark"</code>
                " class. The default chart colors switch with it, and so do your own "
                <code>"--chart-*"</code>
                " values under "
                <code>".dark"</code>
                ". Use the toggle at the bottom of the sidebar to see the colors above \
                 change."
            </p>
        )
    })
}

/// The bar layout code the Geometry page shows.
const BAR_LAYOUT: &str = r#"use topcoat_graph::{BarOptions, bar_layout};

let counts = [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0];
let options = BarOptions { integer_ticks: true, ..Default::default() };
let layout = bar_layout(counts.len(), &[&counts], &options);

for bar in &layout.bars {
    // bar.series, bar.category, bar.value, and bar.rect.{x, y, width, height}
}
for tick in &layout.ticks {
    // tick.position and tick.label: "0" to "12"
}
"#;

/// The line layout code the Geometry page shows.
const XY_LAYOUT: &str = r#"use topcoat_graph::{Curve, XyOptions, xy_layout};

let wins = [3.0, 5.0, f64::NAN, 4.0, 6.0];
let options = XyOptions { curve: Curve::Smooth, include_zero: true, ..Default::default() };
let layout = xy_layout(wins.len(), &[&wins], &options);

let line = &layout.series[0].line; // SVG path data, in two pieces around the gap
let area = &layout.series[0].area; // the same, closed down to layout.zero
"#;

/// The pie layout code the Geometry page shows.
const PIE_LAYOUT: &str = r#"use topcoat_graph::{PieOptions, pie_layout};

let games = [18.0, 12.0, 2.0];
let options = PieOptions { inner_radius: 0.6, ..Default::default() };

for slice in pie_layout(&games, &options) {
    // slice.index, slice.path, slice.centroid and slice.fraction
}
"#;

/// The geometry crate's sources whose types the Geometry page lists.
const LAYOUT_SOURCE: &str = include_str!("../../crates/topcoat-graph/src/layout.rs");
const PIE_SOURCE: &str = include_str!("../../crates/topcoat-graph/src/pie.rs");
const PATH_SOURCE: &str = include_str!("../../crates/topcoat-graph/src/path.rs");

#[component]
pub async fn geometry(cx: &Cx) -> Result<impl View> {
    let items: Vec<api::Item> = api::items(LAYOUT_SOURCE)
        .into_iter()
        .chain(api::items(PIE_SOURCE))
        // From the paths, only `Curve` belongs here: the layouts take it as an option.
        .chain(
            api::items(PATH_SOURCE)
                .into_iter()
                .filter(|item| item.name == "Curve"),
        )
        .collect();

    Ok(view! {
        page_header(
            title: "Geometry crate",
            description: "Use the layouts directly, to draw charts your own way."
        )
        prose(
            <p>
                "The "
                <code>"topcoat-graph"</code>
                " crate is what the components call to place things: scales, ticks, \
                 stacking, line and area paths, and pie slices. It's plain Rust with no \
                 dependencies and knows nothing about HTML, so you can use it without the \
                 components."
            </p>
            <h2>"The 0 to 100 box"</h2>
            <p>
                "Every layout is computed in a "
                <code>"0 0 100 100"</code>
                " box, with x running right and y running down, so positions double as \
                 percentages. Draw the plot as an SVG with that "
                <code>"viewBox"</code>
                " and "
                <code>"preserveAspectRatio=\"none\""</code>
                ", and place labels over it in HTML with "
                <code>"left"</code>
                " and "
                <code>"top"</code>
                " as percentages. Give strokes "
                <code>"vector-effect=\"non-scaling-stroke\""</code>
                " so they keep their width when the SVG stretches."
            </p>
            <p>
                "A non-finite value, such as "
                <code>"f64::NAN"</code>
                ", means there's no value there. The layouts never panic and never write \
                 a non-number into their output, whatever the input, and an empty chart \
                 still gets an axis."
            </p>
            <h2>"Bars"</h2>
            <p>
                <code>"bar_layout"</code>
                " takes the number of categories, one slice of values per series, and "
                <code>"BarOptions"</code>
                ". It returns the bars, the value axis ticks, a band per category and \
                 where the zero line is."
            </p>
            code_panel(source: BAR_LAYOUT)
            example_section(example: &GEOMETRY_EXAMPLE)
            <h2>"Lines and areas"</h2>
            <p>
                <code>"xy_layout"</code>
                " does the same for line and area charts. Each series gets path data for \
                 its line and for the area under it, and its point in each category. A \
                 missing value breaks the line in two."
            </p>
            code_panel(source: XY_LAYOUT)
            <h2>"Pies"</h2>
            <p>
                <code>"pie_layout"</code>
                " lays values out as slices, clockwise from twelve o'clock. Values that \
                 are zero, negative or missing get no slice, so "
                <code>"slice.index"</code>
                " says which value a slice belongs to. Fill slices with "
                <code>"fill-rule=\"evenodd\""</code>
                "."
            </p>
            code_panel(source: PIE_LAYOUT)
            <h2>"Formatting numbers"</h2>
            <p>
                <code>"fmt::num"</code>
                " writes positions with at most two decimals, for path data and inline \
                 styles. "
                <code>"fmt::value"</code>
                " writes data values for people to read, keeping six significant digits, \
                 so "
                <code>"0.0012"</code>
                " stays "
                <code>"0.0012"</code>
                "."
            </p>
            <h2>"Reference"</h2>
            <p>
                "The layouts' settings and results, read from the crate's source. "
                <code>"Orientation"</code>
                " is "
                <code>"Vertical"</code>
                " or "
                <code>"Horizontal"</code>
                ", "
                <code>"BarMode"</code>
                " is "
                <code>"Grouped"</code>
                " or "
                <code>"Stacked"</code>
                ", and "
                <code>"Curve"</code>
                " is "
                <code>"Linear"</code>
                " or "
                <code>"Smooth"</code>
                ". The first of each is the default."
            </p>
        )
        <div class="mt-8 flex flex-col gap-8">
            for item in items {
                item_entry(item: item)
            }
        </div>
    })
}

#[cfg(test)]
mod tests {
    use super::{INSTALL_COMMANDS, README, readme_block};

    #[test]
    fn the_readme_shows_every_install_command() {
        for command in INSTALL_COMMANDS {
            assert!(README.contains(command), "README lacks {command:?}");
        }
    }

    #[test]
    fn the_readme_colors_are_the_components_defaults() {
        let chart = include_str!("../../crates/topcoat-graph/registry/components/chart.rs");
        let css = readme_block("```css");
        let values: Vec<&str> = css
            .lines()
            .filter_map(|line| line.trim().strip_prefix("--chart-"))
            .map(|rest| rest.split_once(": ").unwrap().1.trim_end_matches(';'))
            .collect();
        assert_eq!(values.len(), 10, "{css}");
        for value in values {
            assert!(
                chart.contains(&value.replace(' ', "_")),
                "{value} is not a default"
            );
        }
    }
}
