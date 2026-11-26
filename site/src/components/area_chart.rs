use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{Attributes, StaticClass, View, class, component, element_id, view},
};
use topcoat_graph::{Curve, Orientation, XyOptions, xy_layout};

use super::{
    chart::{ChartSeries, chart, chart_hit_bands, chart_table, series_color},
    chart_axis::{chart_category_axis, chart_grid, chart_value_axis},
    chart_legend::chart_legend,
    chart_tooltip::{category_tooltips, chart_tooltip},
};

/// The plot and its two axes: labels in a gutter on the left and a row underneath,
/// each sized by its content, with the plot taking the rest.
const FRAME: StaticClass = class!(
    "grid min-h-0 flex-1 grid-cols-[auto_minmax(0,1fr)] grid-rows-[minmax(0,1fr)_auto]",
    "gap-x-2 gap-y-1.5 py-1.5",
);

/// An area chart: one line per series with the space under it filled, down to zero or,
/// when stacked, down to the series below.
///
/// The chart is drawn on the server and reads fine with no script. With the Topcoat
/// runtime on the page, pointing at a category or tabbing to it shows its values in a
/// tooltip. A missing value breaks its area in two. The value axis always starts at
/// zero. It fills its container's width; see [`chart`] for changing its shape.
///
/// ```ignore
/// view! {
///     area_chart(
///         label: "Cards in hand by type",
///         category_header: "Turn",
///         categories: labels(["Turn 1", "Turn 2", "Turn 3"]),
///         series: vec![
///             ChartSeries::new("Minions", [3.0, 4.0, 2.0]),
///             ChartSeries::new("Magic", [2.0, 1.0, 3.0]),
///         ],
///         stacked: true,
///         legend: true,
///     )
/// }
/// ```
///
/// `attrs` are forwarded to the chart's root `<div>`. Extra classes are added to its
/// classes.
#[component]
pub async fn area_chart(
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
    /// Stack the series on top of each other, so the top line is the total.
    #[default]
    stacked: bool,
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
    let options = XyOptions {
        curve,
        stacked,
        include_zero: true,
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
    // Each series' area and the line along its top, skipping series with nothing to
    // draw.
    let shapes: Vec<(String, String, String)> = layout
        .series
        .iter()
        .zip(&colors)
        .filter(|(shape, _)| !shape.line.is_empty())
        .map(|(shape, color)| (shape.area.clone(), shape.line.clone(), color.clone()))
        .collect();

    // The hovered category. The browser owns this value, so it is only compared.
    let hovered = signal(cx, || None::<usize>);
    let tooltip_base = element_id(cx);
    let tooltip_ids: Vec<String> = (0..categories.len())
        .map(|index| format!("{tooltip_base}-{index}"))
        .collect();
    let tooltips = category_tooltips(
        &categories,
        &series,
        &colors,
        &layout.bands,
        Orientation::Vertical,
    );

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
                        for (area, _, color) in &shapes {
                            <path
                                data-chart="area"
                                d=(area)
                                fill-opacity="0.25"
                                style=(format!("fill:{color}"))
                            ></path>
                        }
                        for (_, line, color) in &shapes {
                            <path
                                data-chart="line"
                                d=(line)
                                fill="none"
                                stroke-width="2"
                                stroke-linejoin="round"
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
