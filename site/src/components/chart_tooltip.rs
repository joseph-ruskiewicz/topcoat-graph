use topcoat::{
    Result,
    context::Cx,
    runtime::Signal,
    view::{View, class, component, view},
};

use topcoat_graph::{Band, Orientation, fmt::value};

use super::chart::{ChartSeries, percent};

/// One line of a tooltip: a series' color, name and value.
#[derive(Clone, Debug)]
pub struct TooltipRow {
    /// The series' name.
    pub name: String,
    /// The value, already formatted for display.
    pub value: String,
    /// The series' CSS color, for the swatch.
    pub color: String,
}

/// One category's tooltip: what it says and where it goes.
#[derive(Clone, Debug)]
pub struct CategoryTooltip {
    /// The heading: the category's label.
    pub title: String,
    /// One row per series that has a value in the category.
    pub rows: Vec<TooltipRow>,
    /// The anchor's horizontal position in the plot, from 0 to 100.
    pub x: f64,
    /// The anchor's vertical position in the plot, from 0 to 100.
    pub y: f64,
    /// Whether the panel opens to the left of its anchor.
    pub opens_left: bool,
}

/// Builds the tooltip for each category of a bar, line or area chart.
///
/// A tooltip lists every series that has a value in its category, under the category's
/// label. In a vertical chart it is anchored to the edge of the category's band and
/// opens away from the band, toward whichever side of the plot has more room, so it
/// never covers the marks it describes.
pub fn category_tooltips(
    categories: &[String],
    series: &[ChartSeries],
    colors: &[String],
    bands: &[Band],
    orientation: Orientation,
) -> Vec<CategoryTooltip> {
    categories
        .iter()
        .zip(bands)
        .enumerate()
        .map(|(category, (title, band))| {
            let rows = series
                .iter()
                .zip(colors)
                .filter_map(|(one, color)| {
                    let number = one.values.get(category).filter(|v| v.is_finite())?;
                    Some(TooltipRow {
                        name: one.name.clone(),
                        value: value(*number),
                        color: color.clone(),
                    })
                })
                .collect();
            let opens_left = orientation == Orientation::Vertical && band.center > 50.0;
            let (x, y) = match orientation {
                Orientation::Vertical if opens_left => (band.start, 50.0),
                Orientation::Vertical => (band.start + band.size, 50.0),
                Orientation::Horizontal => (50.0, band.center),
            };
            CategoryTooltip {
                title: title.clone(),
                rows,
                x,
                y,
                opens_left,
            }
        })
        .collect()
}

/// The panel that shows one category's values while it is hovered or focused.
///
/// A chart renders one of these per category, all hidden. The browser shows the one
/// whose `index` matches `hovered`. Place it inside the plot's positioned box: `x` and
/// `y` are percentages of that box. The panel opens to the right of its anchor, or to
/// the left with `opens_left`; pick the side with more room so it stays inside.
#[component]
pub async fn chart_tooltip(
    cx: &Cx,
    /// The chart's hover state: the index of the hovered category, if any.
    hovered: Signal<Option<usize>>,
    /// The index of the category this tooltip describes.
    index: usize,
    /// The element id, which the category's hover target points to with
    /// `aria-describedby`.
    #[into]
    id: String,
    /// The tooltip's heading: the category's label.
    #[into]
    title: String,
    /// One row per series that has a value in this category.
    rows: Vec<TooltipRow>,
    /// The anchor's horizontal position in the plot, from 0 to 100.
    x: f64,
    /// The anchor's vertical position in the plot, from 0 to 100.
    y: f64,
    /// Open to the left of the anchor instead of the right.
    #[default]
    opens_left: bool,
) -> Result<impl View> {
    Ok(view! {
        <div
            role="tooltip"
            data-chart="tooltip"
            id=(id)
            class=(class!(
                "pointer-events-none absolute z-10 flex -translate-y-1/2 flex-col gap-1",
                "rounded-md border border-border bg-popover px-2.5 py-1.5 whitespace-nowrap",
                "text-popover-foreground shadow-sm",
                if opens_left { "mr-2 -translate-x-full -ml-2" } else { "ml-2" },
            ))
            style=(format!("left:{};top:{}", percent(x), percent(y)))
            :hidden=$(if hovered.get().is_some() {
                hovered.get().unwrap() != index
            } else {
                true
            })
        >
            <div class="font-medium">(title)</div>
            for row in rows {
                <div data-chart="tooltip-row" class="flex items-center gap-2">
                    <span
                        class="size-2 shrink-0 rounded-[2px]"
                        style=(format!("background:{}", row.color))
                    ></span>
                    <span class="text-muted-foreground">(row.name)</span>
                    <span class="ml-auto pl-3 font-medium tabular-nums">
                        (row.value)
                    </span>
                </div>
            }
        </div>
    })
}
