use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use topcoat_graph::{BarOptions, bar_layout, fmt::num};

use crate::components::{
    chart::{ChartSeries, chart, chart_table, labels, series_color},
    chart_axis::{chart_category_axis, chart_grid, chart_value_axis},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let months = labels([
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]);
    let games = [
        12.0, 9.0, 14.0, 20.0, 18.0, 25.0, 31.0, 28.0, 22.0, 17.0, 15.0, 19.0,
    ];
    let options = BarOptions {
        integer_ticks: true,
        ..Default::default()
    };
    // The bar layout gives the ticks, the bands and where each bar would end.
    let layout = bar_layout(months.len(), &[&games], &options);
    let color = series_color(0, None);

    Ok(view! {
        chart(
            label: "Games per month",
            <div
                class="grid min-h-0 flex-1 grid-cols-[auto_minmax(0,1fr)] grid-rows-[minmax(0,1fr)_auto] gap-x-2 gap-y-1.5 py-1.5"
            >
                chart_value_axis(ticks: layout.ticks.clone())
                <div class="relative min-h-0">
                    chart_grid(ticks: layout.ticks)
                    <svg
                        viewBox="0 0 100 100"
                        preserveAspectRatio="none"
                        aria-hidden="true"
                        class="absolute inset-0 size-full overflow-visible"
                    >
                        for bar in &layout.bars {
                            let x = num(bar.rect.x + bar.rect.width / 2.0);
                            let top = num(bar.rect.y);
                            // A stem from the zero line, then a round head.
                            <path
                                d=(format!("M{x} {}L{x} {top}", num(layout.zero)))
                                stroke-width="2"
                                vector-effect="non-scaling-stroke"
                                style=(format!("stroke:{color}"))
                            ></path>
                            <path
                                d=(format!("M{x} {top}L{x} {top}"))
                                stroke-width="10"
                                stroke-linecap="round"
                                vector-effect="non-scaling-stroke"
                                style=(format!("stroke:{color}"))
                            ></path>
                        }
                    </svg>
                </div>
                <div></div>
                chart_category_axis(
                    labels: months.clone(),
                    bands: layout.bands,
                    label_every: 2
                )
            </div>
            chart_table(
                label: "Games per month",
                category_header: "Month",
                categories: months,
                series: vec![ChartSeries::new("Games", games)]
            )
        )
    })
}
