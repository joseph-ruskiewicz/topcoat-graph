//! What the docs say about each component, in one place.
//!
//! `COMPONENTS` drives the sidebar, the component pages, their previous and next links,
//! and `llms.txt`. Each example is a real component in `src/examples/`: the page renders
//! it as the preview and shows the same file, through `include_str!`, as its code. The
//! manual-install source, the parts a component needs and the API tables come from the
//! registry's files. So the only prose here is a title, a one-line description and the
//! example headings.

use topcoat::{
    context::Cx,
    view::{BoxView, ViewExt, view},
};

/// Which sidebar group a component is listed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// A finished chart, such as `bar_chart`.
    Chart,
    /// A part charts are built from, such as `chart_axis`.
    Part,
}

/// One component's docs page.
pub struct ComponentDoc {
    /// The registry name, used in `topcoat ui add` and in the page URL.
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub group: Group,
    /// The registry source of the component.
    pub source: &'static str,
    /// Live controls for the chart, shown above the examples under "Playground".
    pub playground: Option<fn(&Cx) -> BoxView<'_>>,
    pub examples: &'static [Example],
}

/// An example: a component in `src/examples/` and its source.
pub struct Example {
    /// The file name without `.rs`, unique across the site.
    pub id: &'static str,
    pub title: &'static str,
    /// A sentence under the title, or `""`.
    pub description: &'static str,
    /// The example's source file.
    pub source: &'static str,
    /// Renders the example.
    pub render: fn(&Cx) -> BoxView<'_>,
}

/// Declares an example from its file in `src/examples/`.
macro_rules! example {
    ($file:ident, $title:literal $(, $description:literal)? $(,)?) => {{
        fn render(cx: &Cx) -> BoxView<'_> {
            view! { crate::examples::$file::example() }.boxed()
        }
        Example {
            id: stringify!($file),
            title: $title,
            description: example!(@description $($description)?),
            source: include_str!(concat!("examples/", stringify!($file), ".rs")),
            render,
        }
    }};
    (@description $description:literal) => { $description };
    (@description) => { "" };
}

/// The registry source of the component `name`.
macro_rules! registry_source {
    ($name:literal) => {
        include_str!(concat!(
            "../../crates/topcoat-graph/registry/components/",
            $name,
            ".rs"
        ))
    };
}

/// Declares the function that renders a chart's playground.
macro_rules! playground {
    ($name:ident, $module:ident) => {
        fn $name(cx: &Cx) -> BoxView<'_> {
            view! { crate::playground::$module::playground() }.boxed()
        }
    };
}

playground!(area_playground, area);
playground!(bar_playground, bar);
playground!(line_playground, line);
playground!(pie_playground, pie);

/// Every component, in sidebar order: the charts, then their parts, each alphabetical
/// by title.
pub static COMPONENTS: &[ComponentDoc] = &[
    ComponentDoc {
        name: "area_chart",
        title: "Area Chart",
        description: "Filled areas under one or more lines, overlapping or stacked.",
        group: Group::Chart,
        source: registry_source!("area_chart"),
        playground: Some(area_playground),
        examples: &[
            example!(
                area_chart_single_series,
                "Single series",
                "The space under the line is filled down to zero. The value axis always starts at zero."
            ),
            example!(
                area_chart_several_series,
                "Several series",
                "Areas overlap, each filled down to zero. `legend: true` names them under the plot."
            ),
            example!(
                area_chart_smooth,
                "Smooth",
                "`curve: Curve::Smooth` draws a curve through every point that never overshoots the data."
            ),
            example!(
                area_chart_stacked,
                "Stacked",
                "`stacked: true` puts each series on top of the one before, so the top line is the total."
            ),
            example!(
                area_chart_missing_values,
                "Missing values",
                "A value of `f64::NAN` breaks the area in two and is left out of the tooltip."
            ),
            example!(
                area_chart_no_data,
                "No data",
                "An empty chart keeps its frame and axis."
            ),
        ],
    },
    ComponentDoc {
        name: "bar_chart",
        title: "Bar Chart",
        description: "Bars, vertical or horizontal, with several series grouped or stacked.",
        group: Group::Chart,
        source: registry_source!("bar_chart"),
        playground: Some(bar_playground),
        examples: &[
            example!(
                bar_chart_single_series,
                "Single series",
                "A deck's cards by mana cost. `integer_ticks: true` keeps the axis on whole numbers."
            ),
            example!(
                bar_chart_grouped,
                "Grouped",
                "Several series sit side by side in each category. `legend: true` names them."
            ),
            example!(
                bar_chart_stacked,
                "Stacked",
                "`mode: BarMode::Stacked` puts the series on top of each other, so each bar's height is the total."
            ),
            example!(
                bar_chart_horizontal,
                "Horizontal",
                "`orientation: Orientation::Horizontal` runs the categories down the side, which suits long labels. A horizontal chart usually wants a fixed height."
            ),
            example!(
                bar_chart_negative_values,
                "Negative values",
                "Bars grow from the zero line in both directions."
            ),
            example!(
                bar_chart_missing_values,
                "Missing values",
                "A value of `f64::NAN` leaves a gap and is left out of the tooltip."
            ),
            example!(
                bar_chart_many_categories,
                "Many categories",
                "With `label_every: 5`, one category label in five is shown. Every category still has its bar, its tooltip and its row in the data table."
            ),
            example!(
                bar_chart_no_data,
                "No data",
                "An empty chart keeps its frame and axis."
            ),
        ],
    },
    ComponentDoc {
        name: "line_chart",
        title: "Line Chart",
        description: "One or more lines through a point per category, straight or smooth.",
        group: Group::Chart,
        source: registry_source!("line_chart"),
        playground: Some(line_playground),
        examples: &[
            example!(
                line_chart_single_series,
                "Single series",
                "One value per week, joined by straight segments."
            ),
            example!(
                line_chart_several_series,
                "Several series",
                "One line per series, named in a legend. `include_zero: true` starts the value axis at zero."
            ),
            example!(
                line_chart_smooth,
                "Smooth",
                "`curve: Curve::Smooth` draws a curve through every point that never overshoots the data."
            ),
            example!(
                line_chart_dots,
                "Dots",
                "`dots: true` marks every value. The day without a value has neither dot nor line."
            ),
            example!(
                line_chart_small_values,
                "Small values",
                "Values keep their precision in the tooltip and the data table."
            ),
            example!(
                line_chart_missing_values,
                "Missing values",
                "A value of `f64::NAN` breaks the line and is left out of the tooltip."
            ),
            example!(
                line_chart_no_data,
                "No data",
                "An empty chart keeps its frame and axis."
            ),
        ],
    },
    ComponentDoc {
        name: "pie_chart",
        title: "Pie Chart",
        description: "A pie or a donut, one slice per value, with a tooltip on each slice.",
        group: Group::Chart,
        source: registry_source!("pie_chart"),
        playground: Some(pie_playground),
        examples: &[
            example!(
                pie_chart_pie,
                "Pie",
                "One slice per value, clockwise from twelve o'clock. `value_header` names the values in the tooltip and the data table."
            ),
            example!(
                pie_chart_small_slice,
                "A very small slice",
                "A slice too thin to see still has its share in the legend, the tooltip and the data table."
            ),
            example!(
                pie_chart_donut,
                "Donut",
                "`donut: true` cuts a hole in the middle. Children go in the hole, such as a total."
            ),
            example!(
                pie_chart_zero_value,
                "A zero among the values",
                "Earth has no cards, so it gets no slice. It stays in the legend, and Fire keeps its color."
            ),
            example!(
                pie_chart_single_slice,
                "A single slice",
                "One value with everything: a full circle."
            ),
            example!(
                pie_chart_no_data,
                "No data",
                "When every value is zero there are no slices, only an empty ring."
            ),
        ],
    },
    ComponentDoc {
        name: "chart",
        title: "Chart",
        description: "The frame every chart sits in, with the series type, colors and the hidden data table.",
        group: Group::Part,
        source: registry_source!("chart"),
        playground: None,
        examples: &[
            example!(
                chart_sparkline,
                "A sparkline",
                "`chart` names the figure and `chart_table` carries the numbers for screen readers. The line comes from `xy_layout` and `series_color` gives it the first chart color."
            ),
            example!(
                chart_shares,
                "Shares of a total",
                "Marks can be plain HTML too. `percent` turns a position in the 0 to 100 box into a width."
            ),
        ],
    },
    ComponentDoc {
        name: "chart_axis",
        title: "Chart Axis",
        description: "Value axis labels, category axis labels and grid lines.",
        group: Group::Part,
        source: registry_source!("chart_axis"),
        playground: None,
        examples: &[
            example!(
                chart_axis_lollipop,
                "A lollipop chart",
                "`bar_layout` gives the ticks, the bands and where each bar would end. The axes and the grid take them as they are, and the marks are a stem and a dot. `label_every: 2` shows every other month."
            ),
            example!(
                chart_axis_horizontal,
                "Categories down the side",
                "With `Orientation::Horizontal`, the category axis is a column on the left and the value axis a row underneath."
            ),
        ],
    },
    ComponentDoc {
        name: "chart_legend",
        title: "Chart Legend",
        description: "A color swatch and a name per series or slice.",
        group: Group::Part,
        source: registry_source!("chart_legend"),
        playground: None,
        examples: &[
            example!(
                chart_legend_basic,
                "A legend for your own marks",
                "Each item is a name and the CSS color its marks use. `series_color` gives the same colors the charts use."
            ),
            example!(
                chart_legend_shared,
                "One legend for two charts",
                "Two charts with the same series can share one legend. Each chart gives its first series `series_color(0, None)`, so the colors line up."
            ),
        ],
    },
    ComponentDoc {
        name: "chart_tooltip",
        title: "Chart Tooltip",
        description: "The panel shown for a hovered or focused category or slice.",
        group: Group::Part,
        source: registry_source!("chart_tooltip"),
        playground: None,
        examples: &[
            example!(
                chart_tooltip_sparkline,
                "Tooltips on your own chart",
                "A signal holds the hovered day. `chart_hit_bands` sets it, `category_tooltips` builds one panel per day, and each `chart_tooltip` shows while its index matches."
            ),
            example!(
                chart_tooltip_custom_rows,
                "Rows of your own",
                "Build the `TooltipRow`s yourself to show more than the plotted value. Here each week also shows the change from the week before."
            ),
        ],
    },
];

/// The example on the home page.
pub static HOME_EXAMPLE: Example = example!(line_chart_several_series, "Line chart");

/// The examples on the Theming page.
pub static THEMING_EXAMPLES: &[Example] = &[
    example!(
        theming_series_color,
        "A color per series",
        "`.color(...)` takes any CSS color. Theme tokens such as `var(--primary)` follow light and dark mode."
    ),
    example!(
        theming_sizes,
        "Sizes",
        "Classes in `attrs` change a chart's shape. `aspect-auto h-40` gives a fixed height; `max-w-32` makes a pie smaller."
    ),
];

/// The example on the Geometry page.
pub static GEOMETRY_EXAMPLE: Example = example!(
    geometry_svg,
    "A plain SVG from a layout",
    "No components and no runtime: `bar_layout` gives each bar's rectangle in the 0 to 100 box, and the SVG stretches it to any size."
);

/// The blocks on `/blocks`, in page order. To add one, put its file in `src/examples/`
/// as `block_<name>.rs`, declare it in `examples.rs`, and append an entry here.
pub static BLOCKS: &[Example] = &[
    example!(
        block_mana_curve,
        "Mana curve",
        "A card with a heading and a single-series bar chart at a fixed height."
    ),
    example!(
        block_stat_cards,
        "Stat cards over a chart",
        "Four headline numbers, each with its change from last month, above the weekly trend of the first. The cards stack on narrow screens."
    ),
    example!(
        block_interactive_range,
        "Range and metric switcher",
        "The buttons set signals that a `#[shard]` reads, so the server draws the chart again for the chosen range and metric."
    ),
    example!(
        block_multi_line,
        "Win rate by deck",
        "Three smooth lines with dots and a legend, and a footer worked out from the data."
    ),
    example!(
        block_donut_total,
        "Donut with a total",
        "The total sits in the hole, and a table beside the chart gives each slice's share."
    ),
    example!(
        block_ranked_bars,
        "Ranked list",
        "A horizontal bar chart sorted from most to least. Long names are cut short on the axis and shown in full in the tooltip."
    ),
];

/// The registry manifest, read for the parts each component needs.
const REGISTRY: &str = include_str!("../../crates/topcoat-graph/registry/registry.toml");

/// The topcoat-graph components that `registry.toml` lists as dependencies of `name`.
/// `topcoat ui add` installs them along with it.
pub fn dependencies(name: &str) -> Vec<&'static str> {
    let header = format!("[components.{name}]");
    let Some(start) = REGISTRY.find(&header) else {
        return Vec::new();
    };
    let section = &REGISTRY[start + header.len()..];
    let section = &section[..section.find("\n[").unwrap_or(section.len())];
    let Some((_, list)) = section.split_once("dependencies = [") else {
        return Vec::new();
    };
    let list = &list[..list.find(']').unwrap_or(list.len())];
    list.split(',')
        .map(|name| name.trim().trim_matches('"'))
        .filter(|name| !name.is_empty())
        .collect()
}

/// A page in sidebar order, for previous and next links.
pub struct Page {
    pub title: &'static str,
    pub href: String,
}

/// Every docs page in sidebar order.
pub fn pages() -> Vec<Page> {
    let page = |title, href: &str| Page {
        title,
        href: href.to_owned(),
    };
    let mut pages = vec![
        page("Home", "/"),
        page("Introduction", "/docs/introduction"),
        page("Installation", "/docs/installation"),
        page("Theming", "/docs/theming"),
    ];
    pages.extend(
        COMPONENTS
            .iter()
            .map(|doc| page(doc.title, &format!("/docs/{}", doc.name))),
    );
    pages.push(page("Blocks", "/blocks"));
    pages.push(page("Geometry crate", "/docs/geometry"));
    pages
}

/// `/llms.txt`: an index of the docs for language models.
pub fn llms_txt() -> String {
    let mut out = String::from(
        "# topcoat-graph\n\n\
         > Bar, line, area and pie charts for Topcoat, the Rust web framework. Charts render \
         on the server as SVG with HTML labels and work without script; with the Topcoat \
         runtime they show tooltips on hover and focus. Components are copied into an app as \
         source with `topcoat ui add <name> --registry topcoat-graph`, and call the chart \
         geometry in the `topcoat-graph` crate.\n\n\
         ## Docs\n\n\
         - [Introduction](/docs/introduction): what topcoat-graph is and how it works\n\
         - [Installation](/docs/installation): adding the crate, installing components, \
         runtime setup for tooltips, and updating\n\
         - [Theming](/docs/theming): the chart color tokens, per-series colors, sizes and \
         dark mode\n\
         - [Blocks](/blocks): charts composed into larger pieces\n\
         - [Geometry crate](/docs/geometry): using the layouts without the components\n\n\
         ## Charts\n\n",
    );
    list(&mut out, Group::Chart);
    out.push_str("\n## Parts\n\n");
    list(&mut out, Group::Part);
    out
}

/// Appends a line per component of `group` to `out`.
fn list(out: &mut String, group: Group) {
    for doc in COMPONENTS.iter().filter(|doc| doc.group == group) {
        out.push_str(&format!(
            "- [{}](/docs/{}): {} Install: `topcoat ui add {} --registry topcoat-graph`\n",
            doc.title, doc.name, doc.description, doc.name
        ));
    }
}

/// The docs of the component `name`.
pub fn component(name: &str) -> Option<&'static ComponentDoc> {
    let find = |docs: &'static [ComponentDoc]| docs.iter().find(|doc| doc.name == name);
    #[cfg(test)]
    if let Some(doc) = find(tests::TEST_COMPONENTS) {
        return Some(doc);
    }
    find(COMPONENTS)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{collections::BTreeSet, path::Path};

    use topcoat::{
        context::Cx,
        view::{BoxView, ViewExt, view},
    };
    use topcoat_ui::Registry;

    use super::{
        BLOCKS, COMPONENTS, ComponentDoc, Example, GEOMETRY_EXAMPLE, Group, THEMING_EXAMPLES,
    };

    fn test_playground(cx: &Cx) -> BoxView<'_> {
        view! { <p data-test="playground">"Test playground"</p> }.boxed()
    }

    /// Pages that only tests can reach, at `/docs/<name>`: a component with a playground,
    /// so the page's playground slot is covered before any real chart has one.
    pub static TEST_COMPONENTS: &[ComponentDoc] = &[ComponentDoc {
        name: "test_playground",
        title: "Test Playground",
        description: "A test page.",
        group: Group::Part,
        source: registry_source!("chart_legend"),
        playground: Some(test_playground),
        examples: &[example!(chart_legend_basic, "An example")],
    }];

    #[test]
    fn every_registry_component_has_a_page_with_an_example() {
        let registry = Registry::load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../crates/topcoat-graph/registry"),
        )
        .expect("the workspace's topcoat-graph registry loads");
        let registry: BTreeSet<&str> = registry.names().collect();
        let documented: BTreeSet<&str> = COMPONENTS.iter().map(|doc| doc.name).collect();

        assert_eq!(
            registry, documented,
            "every registry component needs a docs entry"
        );
        for doc in COMPONENTS {
            assert!(!doc.examples.is_empty(), "{} has no example", doc.name);
        }
    }

    #[test]
    fn reads_dependencies_from_the_registry() {
        use super::dependencies;

        assert_eq!(
            dependencies("bar_chart"),
            ["chart", "chart_axis", "chart_legend", "chart_tooltip"]
        );
        assert_eq!(
            dependencies("pie_chart"),
            ["chart", "chart_legend", "chart_tooltip"]
        );
        assert_eq!(dependencies("chart_axis"), ["chart"]);
        assert!(dependencies("chart").is_empty());
        assert!(dependencies("chart_legend").is_empty());
    }

    #[test]
    fn components_are_in_sidebar_order() {
        for group in [Group::Chart, Group::Part] {
            let titles: Vec<&str> = COMPONENTS
                .iter()
                .filter(|doc| doc.group == group)
                .map(|doc| doc.title)
                .collect();
            let mut sorted = titles.clone();
            sorted.sort_unstable();
            assert_eq!(titles, sorted);
        }
        let groups: Vec<Group> = COMPONENTS.iter().map(|doc| doc.group).collect();
        assert!(
            groups.is_sorted_by_key(|group| *group == Group::Part),
            "charts come first"
        );
    }

    #[test]
    fn example_ids_are_unique_and_match_their_files() {
        let all: Vec<&Example> = COMPONENTS
            .iter()
            .flat_map(|doc| doc.examples)
            .chain(BLOCKS)
            .chain(THEMING_EXAMPLES)
            .chain([&GEOMETRY_EXAMPLE])
            .collect();
        let ids: Vec<&str> = all.iter().map(|example| example.id).collect();
        let unique: BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len());
        for doc in COMPONENTS {
            for example in doc.examples {
                assert!(
                    example.id.starts_with(&format!("{}_", doc.name)),
                    "{} is on the {} page",
                    example.id,
                    doc.name
                );
            }
        }
        for block in BLOCKS {
            assert!(block.id.starts_with("block_"), "{}", block.id);
        }
    }

    #[test]
    fn every_example_file_is_in_the_catalog() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/examples");
        let files: BTreeSet<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| {
                let name = entry.unwrap().file_name().into_string().unwrap();
                name.trim_end_matches(".rs").to_owned()
            })
            .collect();
        let listed: BTreeSet<String> = COMPONENTS
            .iter()
            .flat_map(|doc| doc.examples)
            .chain(BLOCKS)
            .chain(THEMING_EXAMPLES)
            .chain([&GEOMETRY_EXAMPLE])
            .map(|example| example.id.to_owned())
            .collect();
        assert_eq!(files, listed);
    }
}
