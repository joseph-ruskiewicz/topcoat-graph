use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{Attributes, StaticClass, View, class, component, element_id, view},
};
use topcoat_graph::{Curve, Orientation, XyOptions, fmt::num, xy_layout};

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

/// A line chart: one line per series, through a point per category.
///
/// The chart is drawn on the server and reads fine with no script. With the Topcoat
/// runtime on the page, pointing at a category or tabbing to it shows its values in a
/// tooltip. A missing value breaks its line in two. It fills its container's width;
/// see [`chart`] for changing its shape.
///
/// ```ignore
/// view! {
///     line_chart(
///         label: "Games won and lost",
///         category_header: "Week",
///         categories: labels(["Week 1", "Week 2", "Week 3"]),
///         series: vec![
///             ChartSeries::new("Wins", [3.0, 5.0, 4.0]),
///             ChartSeries::new("Losses", [2.0, 1.0, 3.0]),
///         ],
///         legend: true,
///     )
/// }
/// ```
///
/// `attrs` are forwarded to the chart's root `<div>`. Extra classes are added to its
/// classes.
#[component]
pub async fn line_chart(
    cx: &Cx,
    /// The chart's accessible name: what it shows.
    #[into]
    label: String,
    /// The heading for the categories in the chart's data table, such as "Week".
    #[into]
    category_header: String,
    /// The category labels, in order.
    categories: Vec<String>,
    /// The series to draw. Each has one value per category.
    series: Vec<ChartSeries>,
    /// Straight segments between points, or a smooth curve through them.
    #[default]
    curve: Curve,
    /// Mark each value with a dot.
    #[default]
    dots: bool,
    /// Keep the value axis on whole numbers, for charts that count things.
    #[default]
    integer_ticks: bool,
    /// Start the value axis at zero even when the values sit far from it.
    #[default]
    include_zero: bool,
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
    let options = XyOptions {
        curve,
        include_zero,
        integer_ticks,
        ..Default::default()
    };
    let layout = xy_layout(categories.len(), &values, &options);
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
    // Each series' line, skipping series with nothing to draw.
    let lines: Vec<(String, String)> = layout
        .series
        .iter()
        .zip(&colors)
        .filter(|(shape, _)| !shape.line.is_empty())
        .map(|(shape, color)| (shape.line.clone(), color.clone()))
        .collect();
    // A zero-length stroke at every present point. Its round cap draws a dot that stays
    // round however the plot stretches.
    let dot_marks: Vec<(String, String)> = if dots {
        layout
            .series
            .iter()
            .zip(&colors)
            .flat_map(|(shape, color)| {
                shape.points.iter().flatten().map(|&(x, y)| {
                    let at = format!("{},{}", num(x), num(y));
                    (format!("M{at}L{at}"), color.clone())
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    // The hovered category. The browser owns this value, so it is only compared.
    let hovered = signal(cx, || None::<usize>);
    let tooltip_base = element_id(cx);
    let tooltip_ids: Vec<String> = (0..categories.len())
        .map(|index| format!("{tooltip_base}-{index}"))
        .collect();
    // Each category's tooltip: every series with a value there, anchored beside the
    // category's points so it does not cover them.
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
            let x = if band.center <= 50.0 {
                band.start + band.size
            } else {
                band.start
            };
            (title.clone(), rows, x, 50.0)
        })
        .collect();

    Ok(view! {
        chart(
            label: label.clone(),
            attrs: attrs,
            <div class=(FRAME)>
                chart_value_axis(
                    ticks: layout.ticks.clone(),
                    orientation: Orientation::Vertical
                )
                <div
                    data-chart="plot"
                    class="relative min-h-0"
                    @mouseleave=$(|_e| hovered.set(None))
                >
                    chart_grid(
                        ticks: layout.ticks.clone(),
                        orientation: Orientation::Vertical
                    )
                    <svg
                        viewBox="0 0 100 100"
                        preserveAspectRatio="none"
                        aria-hidden="true"
                        class="absolute inset-0 size-full overflow-visible"
                    >
                        for (d, color) in lines {
                            <path
                                data-chart="line"
                                d=(d)
                                fill="none"
                                stroke-width="2"
                                stroke-linejoin="round"
                                stroke-linecap="round"
                                vector-effect="non-scaling-stroke"
                                style=(format!("stroke:{color}"))
                            ></path>
                        }
                        for (d, color) in dot_marks {
                            <path
                                data-chart="dot"
                                d=(d)
                                fill="none"
                                stroke-width="6"
                                stroke-linecap="round"
                                vector-effect="non-scaling-stroke"
                                style=(format!("stroke:{color}"))
                            ></path>
                        }
                    </svg>
                    chart_hit_bands(
                        hovered: hovered.clone(),
                        labels: categories.clone(),
                        tooltip_ids: tooltip_ids.clone(),
                        orientation: Orientation::Vertical
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
                chart_category_axis(
                    labels: categories.clone(),
                    bands: layout.bands,
                    orientation: Orientation::Vertical,
                    label_every: label_every
                )
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
