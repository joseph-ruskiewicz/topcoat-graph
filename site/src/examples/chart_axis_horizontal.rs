use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use topcoat_graph::{BarOptions, Orientation, bar_layout, fmt::num};

use crate::components::{
    chart::{ChartSeries, chart, chart_table, labels, series_color},
    chart_axis::{chart_category_axis, chart_grid, chart_value_axis},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let formats = labels(["Standard", "Draft", "Sealed", "Two-headed giant"]);
    let minutes = [24.0, 31.0, 38.0, 52.0];
    let options = BarOptions {
        orientation: Orientation::Horizontal,
        ..Default::default()
    };
    let layout = bar_layout(formats.len(), &[&minutes], &options);
    let color = series_color(1, None);

    Ok(view! {
        chart(
            label: "Average game length by format",
            attrs: attributes! { class="aspect-auto h-48" },
            <div
                class="grid min-h-0 flex-1 grid-cols-[auto_minmax(0,1fr)] grid-rows-[minmax(0,1fr)_auto] gap-x-2 gap-y-1.5 py-1.5"
            >
                // Categories down the side, values along the bottom.
                chart_category_axis(
                    labels: formats.clone(),
                    bands: layout.bands.clone(),
                    orientation: Orientation::Horizontal
                )
                <div class="relative min-h-0">
                    chart_grid(
                        ticks: layout.ticks.clone(),
                        orientation: Orientation::Horizontal
                    )
                    <svg
                        viewBox="0 0 100 100"
                        preserveAspectRatio="none"
                        aria-hidden="true"
                        class="absolute inset-0 size-full overflow-visible"
                    >
                        for bar in &layout.bars {
                            // A dot where the bar would end.
                            let x = num(bar.rect.x + bar.rect.width);
                            let y = num(bar.rect.y + bar.rect.height / 2.0);
                            <path
                                d=(format!("M{x} {y}L{x} {y}"))
                                stroke-width="12"
                                stroke-linecap="round"
                                vector-effect="non-scaling-stroke"
                                style=(format!("stroke:{color}"))
                            ></path>
                        }
                    </svg>
                </div>
                <div></div>
                chart_value_axis(
                    ticks: layout.ticks,
                    orientation: Orientation::Horizontal
                )
            </div>
            chart_table(
                label: "Average game length by format",
                category_header: "Format",
                categories: formats,
                series: vec![ChartSeries::new("Minutes", minutes)]
            )
        )
    })
}
