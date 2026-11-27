//! The area chart playground.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, shard, signal},
    view::{View, attributes, view},
};
use topcoat_graph::Curve;

use super::{
    WEEKS, choice, control, game_series, series_count, series_prop, series_uses, source, weeks_prop,
};
use crate::{
    components::{area_chart::area_chart, chart::labels},
    shell::{code_panel, preview_panel},
};

/// The chart's accessible name.
pub const LABEL: &str = "Games by week as areas";

/// What the controls are set to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub series: usize,
    pub smooth: bool,
    pub stacked: bool,
    pub legend: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            series: 2,
            smooth: false,
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
    if options.smooth {
        graph.push("Curve");
        props.push(String::from("curve: Curve::Smooth"));
    }
    if options.stacked {
        props.push(String::from("stacked: true"));
    }
    props.push(String::from("integer_ticks: true"));
    if options.legend {
        props.push(String::from("legend: true"));
    }
    let uses = series_uses("area_chart");
    source(&graph, "area_chart", &[&uses[0], &uses[1]], &props, &[])
}

/// The playground: an area chart, its controls and its code.
#[shard]
pub async fn playground(cx: &Cx) -> Result<impl View> {
    let defaults = Options::default();
    let series = signal(cx, || defaults.series);
    let smooth = signal(cx, || defaults.smooth);
    let stacked = signal(cx, || defaults.stacked);
    let legend = signal(cx, || defaults.legend);
    let options = Options {
        series: series_count(series.get()),
        smooth: smooth.get(),
        stacked: stacked.get(),
        legend: legend.get(),
    };
    let source = code(options);

    Ok(view! {
        <div class="flex w-full min-w-0 flex-col gap-3">
            preview_panel(
                area_chart(
                    label: LABEL,
                    category_header: "Week",
                    categories: labels(WEEKS),
                    series: game_series(options.series),
                    curve: if options.smooth { Curve::Smooth } else { Curve::Linear },
                    stacked: options.stacked,
                    integer_ticks: true,
                    legend: options.legend
                )
            )
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
                    label: "Curve",
                    choice(
                        label: "Straight",
                        pressed: !options.smooth,
                        attrs: attributes! { @click=$(|_e: Event| smooth.set(false)) }
                    )
                    choice(
                        label: "Smooth",
                        pressed: options.smooth,
                        attrs: attributes! { @click=$(|_e: Event| smooth.set(true)) }
                    )
                )
                control(
                    label: "Stacked",
                    choice(
                        label: "Off",
                        pressed: !options.stacked,
                        attrs: attributes! { @click=$(|_e: Event| stacked.set(false)) }
                    )
                    choice(
                        label: "On",
                        pressed: options.stacked,
                        attrs: attributes! { @click=$(|_e: Event| stacked.set(true)) }
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
    fn the_default_code_is_two_straight_series_with_a_legend() {
        let code = code(Options::default());
        assert!(code.starts_with("use crate::components::{\n"), "{code}");
        assert!(code.contains("    area_chart(\n        label: \"Games by week as areas\",\n"));
        assert_eq!(code.matches("ChartSeries::new").count(), 2);
        assert!(
            code.ends_with("        integer_ticks: true,\n        legend: true\n    )\n}\n"),
            "{code}"
        );
        assert!(!code.contains("curve") && !code.contains("stacked:"));
    }

    #[test]
    fn options_add_their_props_and_imports() {
        let code = code(Options {
            series: 1,
            smooth: true,
            stacked: true,
            legend: false,
        });
        assert!(code.starts_with("use topcoat_graph::Curve;\n\n"), "{code}");
        assert!(code.contains("        curve: Curve::Smooth,\n        stacked: true,\n        integer_ticks: true\n    )"));
        assert_eq!(code.matches("ChartSeries::new").count(), 1);
    }
}
