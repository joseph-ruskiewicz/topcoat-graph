//! The pie chart playground.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, shard, signal},
    view::{View, attributes, component, view},
};

use super::{choice, control, source};
use crate::{
    components::pie_chart::{PieSlice, pie_chart},
    shell::{code_panel, preview_panel},
};

/// The chart's accessible name.
pub const LABEL: &str = "Deck cards by type";

/// The slices, in the order they are added.
pub const TYPES: [(&str, f64); 5] = [
    ("Minions", 18.0),
    ("Magic", 10.0),
    ("Artifacts", 4.0),
    ("Auras", 3.0),
    ("Relics", 2.0),
];

/// What the controls are set to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub slices: usize,
    pub donut: bool,
    pub legend: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            slices: 3,
            donut: false,
            legend: true,
        }
    }
}

/// How many slices to draw: what the browser sent, held to 2 to 5.
pub fn slice_count(sent: usize) -> usize {
    sent.clamp(2, TYPES.len())
}

/// The sum of the first `count` slices.
pub fn total(count: usize) -> f64 {
    TYPES
        .iter()
        .take(slice_count(count))
        .map(|(_, value)| value)
        .sum()
}

/// The code that draws the chart for `options`.
pub fn code(options: Options) -> String {
    let mut slices = String::from("slices: vec![\n");
    for (name, value) in TYPES.iter().take(slice_count(options.slices)) {
        slices += &format!("    PieSlice::new(\"{name}\", {value:.1}),\n");
    }
    slices += "]";
    let mut props = vec![
        format!("label: \"{LABEL}\""),
        String::from("category_header: \"Type\""),
        String::from("value_header: \"Cards\""),
        slices,
    ];
    let mut children = Vec::new();
    if options.donut {
        props.push(String::from("donut: true"));
        children.push(format!(
            "<span class=\"text-2xl font-semibold text-foreground tabular-nums\">\"{}\"</span>",
            total(options.slices)
        ));
        children.push(String::from("<span>\"cards\"</span>"));
    }
    if options.legend {
        props.push(String::from("legend: true"));
    }
    source(
        &[],
        "pie_chart",
        &["pie_chart::{PieSlice, pie_chart}"],
        &props,
        &children,
    )
}

/// The chart for `options`: what the playground draws and what [`code`] describes.
#[component]
pub async fn chart(cx: &Cx, options: Options) -> Result<impl View> {
    let center = total(options.slices).to_string();
    Ok(view! {
        pie_chart(
            label: LABEL,
            category_header: "Type",
            value_header: "Cards",
            slices: TYPES
                .iter()
                .take(options.slices)
                .map(|(name, value)| PieSlice::new(*name, *value))
                .collect(),
            donut: options.donut,
            legend: options.legend,
            if options.donut {
                <span class="text-2xl font-semibold text-foreground tabular-nums">
                    (center)
                </span>
                <span>"cards"</span>
            }
        )
    })
}

/// The playground: a pie chart, its controls and its code.
#[shard]
pub async fn playground(cx: &Cx) -> Result<impl View> {
    let defaults = Options::default();
    let slices = signal(cx, || defaults.slices);
    let donut = signal(cx, || defaults.donut);
    let legend = signal(cx, || defaults.legend);
    let options = Options {
        slices: slice_count(slices.get()),
        donut: donut.get(),
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
                    label: "Slices",
                    choice(
                        label: "2",
                        pressed: options.slices == 2,
                        attrs: attributes! { @click=$(|_e: Event| slices.set(2)) }
                    )
                    choice(
                        label: "3",
                        pressed: options.slices == 3,
                        attrs: attributes! { @click=$(|_e: Event| slices.set(3)) }
                    )
                    choice(
                        label: "4",
                        pressed: options.slices == 4,
                        attrs: attributes! { @click=$(|_e: Event| slices.set(4)) }
                    )
                    choice(
                        label: "5",
                        pressed: options.slices == 5,
                        attrs: attributes! { @click=$(|_e: Event| slices.set(5)) }
                    )
                )
                control(
                    label: "Shape",
                    choice(
                        label: "Pie",
                        pressed: !options.donut,
                        attrs: attributes! { @click=$(|_e: Event| donut.set(false)) }
                    )
                    choice(
                        label: "Donut",
                        pressed: options.donut,
                        attrs: attributes! { @click=$(|_e: Event| donut.set(true)) }
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
    fn the_default_code_is_a_three_slice_pie_with_a_legend() {
        assert_eq!(
            code(Options::default()),
            r#"use crate::components::pie_chart::{PieSlice, pie_chart};

view! {
    pie_chart(
        label: "Deck cards by type",
        category_header: "Type",
        value_header: "Cards",
        slices: vec![
            PieSlice::new("Minions", 18.0),
            PieSlice::new("Magic", 10.0),
            PieSlice::new("Artifacts", 4.0),
        ],
        legend: true
    )
}
"#
        );
    }

    #[test]
    fn a_donut_shows_the_total_of_its_slices_in_the_middle() {
        let code = code(Options {
            slices: 5,
            donut: true,
            legend: false,
        });
        assert!(code.contains("        donut: true,\n        <span class=\"text-2xl font-semibold text-foreground tabular-nums\">\"37\"</span>\n        <span>\"cards\"</span>\n    )"), "{code}");
        assert_eq!(total(2), 28.0);
    }

    #[test]
    fn a_slice_count_from_the_browser_is_held_to_the_data() {
        assert_eq!(slice_count(0), 2);
        assert_eq!(slice_count(usize::MAX), 5);
        assert_eq!(
            code(Options {
                slices: 99,
                ..Options::default()
            })
            .matches("PieSlice::new")
            .count(),
            5
        );
    }
}
