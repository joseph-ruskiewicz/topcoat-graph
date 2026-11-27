use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use topcoat_graph::{Band, Orientation, Tick};

use super::chart::percent;

/// The longest of `labels`, used to reserve room for a column of labels.
fn longest(labels: impl IntoIterator<Item = String>) -> String {
    labels
        .into_iter()
        .max_by_key(|label| label.chars().count())
        .unwrap_or_default()
}

/// The numbers along a chart's value axis.
///
/// For a vertical chart this is a column that goes to the left of the plot, as tall as
/// the plot. For a horizontal chart it is a row that goes under the plot, as wide as
/// the plot. Each label is centered on its tick's position.
#[component]
pub async fn chart_value_axis(
    cx: &Cx,
    /// The ticks to label, from the chart's layout.
    ticks: Vec<Tick>,
    /// Which way the chart's values run.
    #[default]
    orientation: Orientation,
) -> Result<impl View> {
    let vertical = orientation == Orientation::Vertical;
    let widest = longest(ticks.iter().map(|tick| tick.label.clone()));

    Ok(view! {
        <div
            data-chart="value-axis"
            aria-hidden="true"
            class=(if vertical {
                "relative tabular-nums"
            } else {
                "relative h-4 tabular-nums"
            })
        >
            if vertical {
                // Labels are positioned, so an unseen copy of the widest sets the width.
                <span class="invisible block leading-none">(widest)</span>
            }
            for tick in ticks {
                <span
                    data-chart="tick"
                    class=(if vertical {
                        "absolute right-0 -translate-y-1/2 leading-none"
                    } else {
                        "absolute top-0 -translate-x-1/2 leading-4"
                    })
                    style=(if vertical {
                        format!("top:{}", percent(tick.position))
                    } else {
                        format!("left:{}", percent(tick.position))
                    })
                >
                    (tick.label)
                </span>
            }
        </div>
    })
}

/// Grid lines across the plot, one per value tick.
///
/// Put this inside the plot's positioned box, before the marks, so the marks draw on
/// top. The lines are HTML borders, so they stay one pixel wide at any chart size.
#[component]
pub async fn chart_grid(
    cx: &Cx,
    /// The ticks to draw lines for, from the chart's layout.
    ticks: Vec<Tick>,
    /// Which way the chart's values run.
    #[default]
    orientation: Orientation,
) -> Result<impl View> {
    let vertical = orientation == Orientation::Vertical;

    Ok(view! {
        for tick in ticks {
            <div
                data-chart="grid-line"
                aria-hidden="true"
                class=(if vertical {
                    "absolute inset-x-0 border-t border-border"
                } else {
                    "absolute inset-y-0 border-l border-border"
                })
                style=(if vertical {
                    format!("top:{}", percent(tick.position))
                } else {
                    format!("left:{}", percent(tick.position))
                })
            ></div>
        }
    })
}

/// The labels along a chart's category axis.
///
/// For a vertical chart this is a row that goes under the plot. For a horizontal chart
/// it is a column that goes to the left of the plot. Each label is centered on its
/// band and cut off with an ellipsis when it does not fit.
#[component]
pub async fn chart_category_axis(
    cx: &Cx,
    /// The category labels, in order.
    labels: Vec<String>,
    /// Each category's band, from the chart's layout.
    bands: Vec<Band>,
    /// Which way the chart's values run.
    #[default]
    orientation: Orientation,
    /// Show one label in this many, starting with the first. Use it when the labels
    /// of a chart with many categories would run into each other.
    #[default(1)]
    label_every: usize,
) -> Result<impl View> {
    let vertical = orientation == Orientation::Vertical;
    let every = label_every.max(1);
    // A label may spread over the bands of the labels skipped after it.
    let room = 100.0 / bands.len().max(1) as f64 * every as f64;
    let shown: Vec<(String, Band)> = labels.into_iter().zip(bands).step_by(every).collect();
    // In a horizontal chart every label helps size the column: the label with the most
    // characters is not always the widest on screen.
    let sizers: Vec<String> = if vertical {
        Vec::new()
    } else {
        shown.iter().map(|(label, _)| label.clone()).collect()
    };

    Ok(view! {
        <div
            data-chart="category-axis"
            aria-hidden="true"
            class=(if vertical { "relative h-4" } else { "relative max-w-32" })
        >
            for sizer in sizers {
                <span
                    data-chart="label-size"
                    class="invisible block h-0 truncate leading-none"
                >
                    (sizer)
                </span>
            }
            for (label, band) in shown {
                <span
                    data-chart="category"
                    class=(if vertical {
                        "absolute top-0 -translate-x-1/2 truncate text-center leading-4"
                    } else {
                        "absolute right-0 max-w-full -translate-y-1/2 truncate leading-none"
                    })
                    style=(if vertical {
                        format!(
                            "left:{};max-width:{}",
                            percent(band.center),
                            percent(room),
                        )
                    } else {
                        format!("top:{}", percent(band.center))
                    })
                >
                    (label)
                </span>
            }
        </div>
    })
}
