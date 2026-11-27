//! The bar chart playground.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, shard, signal},
    view::{View, attributes, component, view},
};
use topcoat_graph::{BarMode, Orientation};

use super::{
    WEEKS, choice, control, game_series, series_count, series_prop, series_uses, source, weeks_prop,
};
use crate::{
    components::{bar_chart::bar_chart, chart::labels},
    shell::{code_panel, preview_panel},
};

/// The chart's accessible name.
pub const LABEL: &str = "Games by week as bars";

/// What the controls are set to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub series: usize,
    pub horizontal: bool,
    pub stacked: bool,
    pub legend: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            series: 2,
            horizontal: false,
            stacked: false,
            legend: true,
        }
    }
}

/// The code that draws the chart for `options`.
pub fn code(options: Options) -> String {
    let mut graph = Vec::new();
    let mut props = vec![
        format!("label: \"{LABEL}\""),
        String::from("category_header: \"Week\""),
        weeks_prop(),
        series_prop(options.series),
    ];
    if options.horizontal {
        graph.push("Orientation");
        props.push(String::from("orientation: Orientation::Horizontal"));
    }
    if options.stacked {
        graph.insert(0, "BarMode");
        props.push(String::from("mode: BarMode::Stacked"));
    }
    props.push(String::from("integer_ticks: true"));
    if options.legend {
        props.push(String::from("legend: true"));
    }
    let uses = series_uses("bar_chart");
    source(&graph, "bar_chart", &[&uses[0], &uses[1]], &props, &[])
}

/// The chart for `options`: what the playground draws and what [`code`] describes.
#[component]
pub async fn chart(cx: &Cx, options: Options) -> Result<impl View> {
    Ok(view! {
        bar_chart(
            label: LABEL,
            category_header: "Week",
            categories: labels(WEEKS),
            series: game_series(options.series),
            orientation: if options.horizontal {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            },
            mode: if options.stacked { BarMode::Stacked } else { BarMode::Grouped },
            integer_ticks: true,
            legend: options.legend
        )
    })
}

/// The playground: a bar chart, its controls and its code.
#[shard]
pub async fn playground(cx: &Cx) -> Result<impl View> {
    let defaults = Options::default();
    let series = signal(cx, || defaults.series);
    let horizontal = signal(cx, || defaults.horizontal);
    let stacked = signal(cx, || defaults.stacked);
    let legend = signal(cx, || defaults.legend);
    let options = Options {
        series: series_count(series.get()),
        horizontal: horizontal.get(),
        stacked: stacked.get(),
        legend: legend.get(),
    };
    let source = code(options);

    Ok(view! {
        <div class="flex w-full min-w-0 flex-col gap-3">
            preview_panel(chart(options: options))
            <div
                data-slot="controls"
                class="flex flex-wrap items-center gap-x-5 gap-y-2"
            >
                control(
                    label: "Series",
                    choice(
                        label: "1",
                        pressed: options.series == 1,
                        attrs: attributes! { @click=$(|_e: Event| series.set(1)) }
                    )
                    choice(
                        label: "2",
                        pressed: options.series == 2,
                        attrs: attributes! { @click=$(|_e: Event| series.set(2)) }
                    )
                    choice(
                        label: "3",
                        pressed: options.series == 3,
                        attrs: attributes! { @click=$(|_e: Event| series.set(3)) }
                    )
                )
                control(
                    label: "Layout",
                    choice(
                        label: "Grouped",
                        pressed: !options.stacked,
                        attrs: attributes! { @click=$(|_e: Event| stacked.set(false)) }
                    )
                    choice(
                        label: "Stacked",
                        pressed: options.stacked,
                        attrs: attributes! { @click=$(|_e: Event| stacked.set(true)) }
                    )
                )
                control(
                    label: "Direction",
                    choice(
                        label: "Vertical",
                        pressed: !options.horizontal,
                        attrs: attributes! { @click=$(|_e: Event| horizontal.set(false)) }
                    )
                    choice(
                        label: "Horizontal",
                        pressed: options.horizontal,
                        attrs: attributes! { @click=$(|_e: Event| horizontal.set(true)) }
                    )
                )
                control(
                    label: "Legend",
                    choice(
                        label: "Off",
                        pressed: !options.legend,
                        attrs: attributes! { @click=$(|_e: Event| legend.set(false)) }
                    )
                    choice(
                        label: "On",
                        pressed: options.legend,
                        attrs: attributes! { @click=$(|_e: Event| legend.set(true)) }
                    )
                )
            </div>
            code_panel(source: &source)
        </div>
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_code_is_a_grouped_vertical_chart_with_a_legend() {
        assert_eq!(
            code(Options::default()),
            r#"use crate::components::{
    bar_chart::bar_chart,
    chart::{ChartSeries, labels},
};

view! {
    bar_chart(
        label: "Games by week as bars",
        category_header: "Week",
        categories: labels(["Week 1", "Week 2", "Week 3", "Week 4", "Week 5", "Week 6"]),
        series: vec![
            ChartSeries::new("Wins", [3.0, 5.0, 4.0, 7.0, 6.0, 8.0]),
            ChartSeries::new("Losses", [4.0, 3.0, 4.0, 2.0, 3.0, 2.0]),
        ],
        integer_ticks: true,
        legend: true
    )
}
"#
        );
    }

    #[test]
    fn options_add_their_props_and_imports() {
        let all = code(Options {
            series: 3,
            horizontal: true,
            stacked: true,
            legend: false,
        });
        assert!(
            all.starts_with("use topcoat_graph::{BarMode, Orientation};\n\n"),
            "{all}"
        );
        assert!(all.contains("        orientation: Orientation::Horizontal,\n        mode: BarMode::Stacked,\n        integer_ticks: true\n    )"));
        assert!(all.contains("ChartSeries::new(\"Draws\""));
        assert!(!all.contains("legend"));
        let stacked = code(Options {
            stacked: true,
            ..Options::default()
        });
        assert!(
            stacked.starts_with("use topcoat_graph::BarMode;\n\n"),
            "{stacked}"
        );
    }

    #[test]
    fn a_series_count_out_of_range_still_gives_code() {
        let code = code(Options {
            series: 99,
            ..Options::default()
        });
        assert_eq!(code.matches("ChartSeries::new").count(), 3);
    }
}
