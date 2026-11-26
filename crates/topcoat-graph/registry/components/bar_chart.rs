use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{Attributes, StaticClass, View, class, component, element_id, view},
};
use topcoat_graph::{BarMode, BarOptions, Orientation, bar_layout, fmt::num};

use super::{
    chart::{ChartSeries, chart, chart_hit_bands, chart_table, series_color},
    chart_axis::{chart_category_axis, chart_grid, chart_value_axis},
    chart_legend::chart_legend,
    chart_tooltip::{TooltipRow, chart_tooltip},
};

/// The plot and its two axes: labels in a gutter on the left and a row underneath,
/// each sized by its content, with the plot taking the rest.
const FRAME: StaticClass = class!(
    "grid min-h-0 flex-1 grid-cols-[auto_minmax(0,1fr)] grid-rows-[minmax(0,1fr)_auto]",
    "gap-x-2 gap-y-1.5 py-1.5",
);

/// A bar chart: one bar per category, or several for several series.
///
/// The chart is drawn on the server and reads fine with no script. With the Topcoat
/// runtime on the page, pointing at a category or tabbing to it shows its values in a
/// tooltip. It fills its container's width; see [`chart`] for changing its shape.
///
/// ```ignore
/// view! {
///     bar_chart(
///         label: "Mana cost distribution",
///         category_header: "Cost",
///         categories: labels(["0", "1", "2", "3", "4", "5", "6+"]),
///         series: vec![ChartSeries::new("Cards", [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0])],
///         integer_ticks: true,
///     )
/// }
/// ```
///
/// `attrs` are forwarded to the chart's root `<div>`. Extra classes are added to its
/// classes.
#[component]
pub async fn bar_chart(
    cx: &Cx,
    /// The chart's accessible name: what it shows.
    #[into]
    label: String,
    /// The heading for the categories in the chart's data table, such as "Cost".
    #[into]
    category_header: String,
    /// The category labels, in order.
    categories: Vec<String>,
    /// The series to draw. Each has one value per category.
    series: Vec<ChartSeries>,
    /// Bars growing up from the bottom, or rightward from the left.
    #[default]
    orientation: Orientation,
    /// Several series side by side, or stacked on each other.
    #[default]
    mode: BarMode,
    /// Keep the value axis on whole numbers, for charts that count things.
    #[default]
    integer_ticks: bool,
    /// Show one category label in this many. See [`chart_category_axis`].
    #[default(1)]
    label_every: usize,
    /// Show each series' name and color under the plot.
    #[default]
    legend: bool,
    /// Extra attributes for the chart's root `<div>` element.
    #[default]
    attrs: Attributes,
) -> Result<impl View> {
    let values: Vec<&[f64]> = series.iter().map(|one| one.values.as_slice()).collect();
    let options = BarOptions {
        orientation,
        mode,
        integer_ticks,
        ..Default::default()
    };
    let layout = bar_layout(categories.len(), &values, &options);
    let vertical = orientation == Orientation::Vertical;
    let colors: Vec<String> = series
        .iter()
        .enumerate()
        .map(|(index, one)| series_color(index, one.color.as_deref()))
        .collect();
    let legend_items: Vec<(String, String)> = series
        .iter()
        .zip(&colors)
        .map(|(one, color)| (one.name.clone(), color.clone()))
        .collect();

    // The hovered category. The browser owns this value, so it is only compared.
    let hovered = signal(cx, || None::<usize>);
    let tooltip_base = element_id(cx);
    let tooltip_ids: Vec<String> = (0..categories.len())
        .map(|index| format!("{tooltip_base}-{index}"))
        .collect();
    // Each category's tooltip: every series with a value there, anchored beside the
    // category's bars so it does not cover them.
    let tooltips: Vec<(String, Vec<TooltipRow>, f64, f64)> = categories
        .iter()
        .zip(&layout.bands)
        .enumerate()
        .map(|(category, (title, band))| {
            let rows = series
                .iter()
                .zip(&colors)
                .filter_map(|(one, color)| {
                    let value = one.values.get(category).filter(|v| v.is_finite())?;
                    Some(TooltipRow {
                        name: one.name.clone(),
                        value: num(*value),
                        color: color.clone(),
                    })
                })
                .collect();
            let left_half = band.center <= 50.0;
            let (x, y) = if vertical {
                (
                    if left_half {
                        band.start + band.size
                    } else {
                        band.start
                    },
                    50.0,
                )
            } else {
                (50.0, band.center)
            };
            (title.clone(), rows, x, y)
        })
        .collect();

    Ok(view! {
        chart(
            label: label.clone(),
            attrs: attrs,
            <div class=(FRAME)>
                if vertical {
                    chart_value_axis(
                        ticks: layout.ticks.clone(),
                        orientation: orientation
                    )
                } else {
                    chart_category_axis(
                        labels: categories.clone(),
                        bands: layout.bands.clone(),
                        orientation: orientation,
                        label_every: label_every
                    )
                }
                <div
                    data-chart="plot"
                    class="relative min-h-0"
                    @mouseleave=$(|_e| hovered.set(None))
                >
                    chart_grid(ticks: layout.ticks.clone(), orientation: orientation)
                    <svg
                        viewBox="0 0 100 100"
                        preserveAspectRatio="none"
                        aria-hidden="true"
                        class="absolute inset-0 size-full overflow-visible"
                    >
                        for bar in layout.bars {
                            <rect
                                x=(num(bar.rect.x))
                                y=(num(bar.rect.y))
                                width=(num(bar.rect.width))
                                height=(num(bar.rect.height))
                                style=(format!("fill:{}", colors[bar.series]))
                            ></rect>
                        }
                    </svg>
                    chart_hit_bands(
                        hovered: hovered.clone(),
                        labels: categories.clone(),
                        tooltip_ids: tooltip_ids.clone(),
                        orientation: orientation
                    )
                    for (index, (title, rows, x, y)) in tooltips.into_iter().enumerate() {
                        chart_tooltip(
                            hovered: hovered.clone(),
                            index: index,
                            id: tooltip_ids[index].clone(),
                            title: title,
                            rows: rows,
                            x: x,
                            y: y
                        )
                    }
                </div>
                // The corner under the gutter.
                <div></div>
                if vertical {
                    chart_category_axis(
                        labels: categories.clone(),
                        bands: layout.bands,
                        orientation: orientation,
                        label_every: label_every
                    )
                } else {
                    chart_value_axis(ticks: layout.ticks, orientation: orientation)
                }
            </div>
            if legend {
                chart_legend(items: legend_items)
            }
            chart_table(
                label: label,
                category_header: category_header,
                categories: categories,
                series: series
            )
        )
    })
}
