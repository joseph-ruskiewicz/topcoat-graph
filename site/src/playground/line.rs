//! The line chart playground.

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
    components::{chart::labels, line_chart::line_chart},
    shell::{code_panel, preview_panel},
};

/// The chart's accessible name.
pub const LABEL: &str = "Games by week as lines";

/// What the controls are set to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub series: usize,
    pub smooth: bool,
    pub dots: bool,
    pub legend: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            series: 2,
            smooth: false,
            dots: false,
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
    if options.dots {
        props.push(String::from("dots: true"));
    }
    props.push(String::from("integer_ticks: true"));
    if options.legend {
        props.push(String::from("legend: true"));
    }
    let uses = series_uses("line_chart");
    source(&graph, "line_chart", &[&uses[0], &uses[1]], &props, &[])
}

/// The playground: a line chart, its controls and its code.
#[shard]
pub async fn playground(cx: &Cx) -> Result<impl View> {
    let defaults = Options::default();
    let series = signal(cx, || defaults.series);
    let smooth = signal(cx, || defaults.smooth);
    let dots = signal(cx, || defaults.dots);
    let legend = signal(cx, || defaults.legend);
    let options = Options {
        series: series_count(series.get()),
        smooth: smooth.get(),
        dots: dots.get(),
        legend: legend.get(),
    };
    let source = code(options);

    Ok(view! {
        <div class="flex w-full min-w-0 flex-col gap-3">
            preview_panel(
                line_chart(
                    label: LABEL,
                    category_header: "Week",
                    categories: labels(WEEKS),
                    series: game_series(options.series),
                    curve: if options.smooth { Curve::Smooth } else { Curve::Linear },
                    dots: options.dots,
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
                    label: "Dots",
                    choice(
                        label: "Off",
                        pressed: !options.dots,
                        attrs: attributes! { @click=$(|_e: Event| dots.set(false)) }
                    )
                    choice(
                        label: "On",
                        pressed: options.dots,
                        attrs: attributes! { @click=$(|_e: Event| dots.set(true)) }
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
        assert!(code.contains("    line_chart(\n        label: \"Games by week as lines\",\n"));
        assert_eq!(code.matches("ChartSeries::new").count(), 2);
        assert!(
            code.ends_with("        integer_ticks: true,\n        legend: true\n    )\n}\n"),
            "{code}"
        );
        assert!(!code.contains("curve") && !code.contains("dots:"));
    }

    #[test]
    fn options_add_their_props_and_imports() {
        let code = code(Options {
            series: 1,
            smooth: true,
            dots: true,
            legend: false,
        });
        assert!(code.starts_with("use topcoat_graph::Curve;\n\n"), "{code}");
        assert!(code.contains(
            "        curve: Curve::Smooth,\n        dots: true,\n        integer_ticks: true\n    )"
        ));
        assert_eq!(code.matches("ChartSeries::new").count(), 1);
    }
}
