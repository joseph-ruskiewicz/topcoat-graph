use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{View, attributes, component, element_id, view},
};
use topcoat_graph::{Orientation, XyOptions, xy_layout};

use crate::components::{
    chart::{ChartSeries, chart, chart_hit_bands, chart_table, labels, series_color},
    chart_tooltip::{category_tooltips, chart_tooltip},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let days = labels(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]);
    let series = vec![ChartSeries::new(
        "Players",
        [120.0, 135.0, 128.0, 150.0, 160.0, 210.0, 190.0],
    )];
    let colors = vec![series_color(0, None)];
    let layout = xy_layout(days.len(), &[&series[0].values], &XyOptions::default());
    let line = layout.series[0].line.clone();

    // The hovered day, set by the hit bands and read by the tooltips.
    let hovered = signal(cx, || None::<usize>);
    let base = element_id(cx);
    let ids: Vec<String> = (0..days.len()).map(|day| format!("{base}-{day}")).collect();
    let tooltips = category_tooltips(
        &days,
        &series,
        &colors,
        &layout.bands,
        Orientation::Vertical,
    );

    Ok(view! {
        chart(
            label: "Players this week, with tooltips",
            attrs: attributes! { class="aspect-auto h-24 max-w-sm" },
            <div class="relative min-h-0 flex-1" @mouseleave=$(|_e| hovered.set(None))>
                <svg
                    viewBox="0 0 100 100"
                    preserveAspectRatio="none"
                    aria-hidden="true"
                    class="absolute inset-0 size-full overflow-visible"
                >
                    <path
                        d=(line)
                        fill="none"
                        stroke-width="2"
                        stroke-linejoin="round"
                        stroke-linecap="round"
                        vector-effect="non-scaling-stroke"
                        style=(format!("stroke:{}", colors[0]))
                    ></path>
                </svg>
                chart_hit_bands(
                    hovered: hovered.clone(),
                    labels: days.clone(),
                    tooltip_ids: ids.clone()
                )
                for (index, tip) in tooltips.into_iter().enumerate() {
                    chart_tooltip(
                        hovered: hovered.clone(),
                        index: index,
                        id: ids[index].clone(),
                        title: tip.title,
                        rows: tip.rows,
                        x: tip.x,
                        y: tip.y,
                        opens_left: tip.opens_left
                    )
                }
            </div>
            chart_table(
                label: "Players this week, with tooltips",
                category_header: "Day",
                categories: days,
                series: series
            )
        )
    })
}
