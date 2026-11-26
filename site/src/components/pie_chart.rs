use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{Attributes, Child, StaticClass, View, class, component, element_id, view},
};
use topcoat_graph::{PieOptions, fmt::value, pie_layout};

use super::{
    chart::{ChartSeries, chart, chart_table, series_color},
    chart_legend::chart_legend,
    chart_tooltip::{TooltipRow, chart_tooltip},
};

/// One slice of a pie: a name and a value.
#[derive(Clone, Debug)]
pub struct PieSlice {
    /// The slice's name, shown in the legend, the tooltip and the data table.
    pub name: String,
    /// The slice's value. Zero, negative and non-finite values get no slice, but stay
    /// in the legend and the data table.
    pub value: f64,
    /// A CSS color for the slice. Without one it takes the next chart color.
    pub color: Option<String>,
}

impl PieSlice {
    /// A slice in the next chart color.
    ///
    /// ```ignore
    /// PieSlice::new("Minions", 18.0)
    /// ```
    pub fn new(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
            color: None,
        }
    }

    /// Sets the slice's color to any CSS color, such as `"var(--primary)"`.
    pub fn color(mut self, css: impl Into<String>) -> Self {
        self.color = Some(css.into());
        self
    }
}

/// A slice that has something to draw, with what its path and tooltip need.
struct DrawnSlice {
    /// SVG path data in the `0 0 100 100` box.
    path: String,
    /// The slice's CSS color.
    color: String,
    /// The slice's name, which labels its path and heads its tooltip.
    name: String,
    /// The id of the slice's tooltip.
    tooltip_id: String,
    /// The tooltip's row: the value and its share of the total.
    row: TooltipRow,
    /// Where the tooltip is anchored, in the `0 0 100 100` box.
    centroid: (f64, f64),
}

/// The root's shape: centered and at most 18rem wide, with its height left to the
/// square plot. The width limit has no specificity, so a class in `attrs` beats it.
const ROOT: StaticClass = class!("mx-auto aspect-auto [:where(&)]:max-w-72");

/// A slice: dimmed while another slice is hovered, outlined while it has keyboard focus.
const SLICE: StaticClass = class!(
    "outline-none transition-opacity data-[dimmed=true]:opacity-40",
    "focus-visible:stroke-foreground",
);

/// A pie chart, or a donut chart with `donut: true`: one slice per value, clockwise
/// from twelve o'clock.
///
/// The chart is drawn on the server and reads fine with no script. With the Topcoat
/// runtime on the page, pointing at a slice or tabbing to it shows its value and share
/// in a tooltip and dims the other slices. A donut shows its children in the hole, for
/// example a total.
///
/// ```ignore
/// view! {
///     pie_chart(
///         label: "Games by result",
///         category_header: "Result",
///         value_header: "Games",
///         slices: vec![
///             PieSlice::new("Wins", 18.0),
///             PieSlice::new("Losses", 12.0),
///             PieSlice::new("Draws", 2.0),
///         ],
///         donut: true,
///         <span class="text-2xl font-semibold text-foreground">"32"</span>
///     )
/// }
/// ```
///
/// In a `view!` loop that renders one chart per item, put `#[key(...)]` on the loop:
/// each chart keeps its hover state in a signal and needs its own identity.
///
/// The chart is square, as wide as its container up to 18rem, and centered. `attrs`
/// are forwarded to the chart's root `<div>`. Extra classes are added to its classes;
/// to change the width, pass a different limit, such as `max-w-96`.
#[component]
pub async fn pie_chart(
    cx: &Cx,
    /// The chart's accessible name: what it shows.
    #[into]
    label: String,
    /// The heading for the slice names in the chart's data table, such as "Type".
    #[into]
    category_header: String,
    /// The heading for the values in the chart's data table and their tooltip row,
    /// such as "Cards".
    #[into]
    value_header: String,
    /// The slices, in order.
    slices: Vec<PieSlice>,
    /// Cut a hole in the middle, for a donut chart.
    #[default]
    donut: bool,
    /// Show each slice's name and color under the chart.
    #[default]
    legend: bool,
    /// Extra attributes for the chart's root `<div>` element.
    #[default]
    mut attrs: Attributes,
    /// What a donut shows in its hole, such as a total. A pie shows nothing.
    #[default]
    child: Child<'_>,
) -> Result<impl View> {
    let options = PieOptions {
        inner_radius: if donut { 0.6 } else { 0.0 },
        ..Default::default()
    };
    let values: Vec<f64> = slices.iter().map(|slice| slice.value).collect();
    let layout = pie_layout(&values, &options);
    // Colors follow the input, so a slice keeps its color when a neighbor is skipped.
    let colors: Vec<String> = slices
        .iter()
        .enumerate()
        .map(|(index, slice)| series_color(index, slice.color.as_deref()))
        .collect();
    let legend_items: Vec<(String, String)> = slices
        .iter()
        .zip(&colors)
        .map(|(slice, color)| (slice.name.clone(), color.clone()))
        .collect();
    // With nothing to draw, a muted disc or ring keeps the chart's shape.
    let empty = if layout.is_empty() {
        pie_layout(&[1.0], &options).pop().map(|whole| whole.path)
    } else {
        None
    };

    // The hovered slice, by its position among the drawn slices. The browser owns this
    // value, so it is only compared.
    let hovered = signal(cx, || None::<usize>);
    let tooltip_base = element_id(cx);
    let drawn: Vec<DrawnSlice> = layout
        .into_iter()
        .enumerate()
        .map(|(position, shape)| {
            let slice = &slices[shape.index];
            let color = colors[shape.index].clone();
            DrawnSlice {
                path: shape.path,
                name: slice.name.clone(),
                tooltip_id: format!("{tooltip_base}-{position}"),
                row: TooltipRow {
                    name: value_header.clone(),
                    value: format!(
                        "{} ({}%)",
                        value(slice.value),
                        value(shape.fraction * 100.0)
                    ),
                    color: color.clone(),
                },
                color,
                centroid: shape.centroid,
            }
        })
        .collect();
    let names: Vec<String> = slices.iter().map(|slice| slice.name.clone()).collect();

    let extra = attrs.remove("class");
    attrs.insert(cx, "class", class!(ROOT, extra));

    Ok(view! {
        chart(
            label: label.clone(),
            attrs: attrs,
            <div
                data-chart="plot"
                class="relative aspect-square w-full"
                @mouseleave=$(|_e| hovered.set(None))
            >
                // Not aria-hidden like the other charts' plots: the slices inside are
                // focusable, and focusable content must not be hidden from assistive
                // technology. Each slice is named, like the other charts' hover targets.
                <svg
                    viewBox="0 0 100 100"
                    class="absolute inset-0 size-full overflow-visible"
                >
                    if let Some(path) = empty {
                        <path
                            data-chart="empty"
                            aria-hidden="true"
                            d=(path)
                            fill-rule="evenodd"
                            class="fill-border"
                        ></path>
                    }
                    for (index, slice) in drawn.iter().enumerate() {
                        let hovered = hovered.clone();
                        <path
                            data-chart="slice"
                            fill-rule="evenodd"
                            d=(&slice.path)
                            tabindex="0"
                            role="img"
                            aria-label=(&slice.name)
                            aria-describedby=(&slice.tooltip_id)
                            class=(SLICE)
                            stroke-width="2"
                            stroke-linejoin="round"
                            vector-effect="non-scaling-stroke"
                            style=(format!("fill:{}", slice.color))
                            :data-dimmed=$(if hovered.get().is_some() {
                                if hovered.get().unwrap() == index {
                                    "false"
                                } else {
                                    "true"
                                }
                            } else {
                                "false"
                            })
                            @mouseenter=$(|_e| hovered.set(Some(index)))
                            @focus=$(|_e| hovered.set(Some(index)))
                            @blur=$(|_e| hovered.set(None))
                        ></path>
                    }
                </svg>
                if donut {
                    <div
                        data-chart="center"
                        class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center text-center"
                    >
                        (child)
                    </div>
                }
                for (index, slice) in drawn.into_iter().enumerate() {
                    chart_tooltip(
                        hovered: hovered.clone(),
                        index: index,
                        id: slice.tooltip_id,
                        title: slice.name,
                        rows: vec![slice.row],
                        x: slice.centroid.0,
                        y: slice.centroid.1,
                        opens_left: slice.centroid.0 > 50.0
                    )
                }
            </div>
            if legend {
                chart_legend(items: legend_items)
            }
            chart_table(
                label: label,
                category_header: category_header,
                categories: names,
                series: vec![ChartSeries::new(value_header, values)]
            )
        )
    })
}
