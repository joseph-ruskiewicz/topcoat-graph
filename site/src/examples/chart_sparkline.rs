use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use topcoat_graph::{XyOptions, xy_layout};

use crate::components::chart::{ChartSeries, chart, chart_table, labels, series_color};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let days = labels(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]);
    let players = [120.0, 135.0, 128.0, 150.0, 160.0, 210.0, 190.0];
    // The geometry crate lays the line out in a 0 0 100 100 box.
    let layout = xy_layout(days.len(), &[&players], &XyOptions::default());
    let line = layout.series[0].line.clone();

    Ok(view! {
        chart(
            label: "Players this week",
            attrs: attributes! { class="aspect-auto h-16 max-w-64" },
            <svg
                viewBox="0 0 100 100"
                preserveAspectRatio="none"
                aria-hidden="true"
                class="size-full overflow-visible"
            >
                <path
                    d=(line)
                    fill="none"
                    stroke-width="2"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                    vector-effect="non-scaling-stroke"
                    style=(format!("stroke:{}", series_color(0, None)))
                ></path>
            </svg>
            chart_table(
                label: "Players this week",
                category_header: "Day",
                categories: days,
                series: vec![ChartSeries::new("Players", players)]
            )
        )
    })
}
