use topcoat::{
    Result,
    context::Cx,
    runtime::signal,
    view::{View, attributes, component, element_id, view},
};
use topcoat_graph::{BarOptions, bar_layout, fmt::num};

use crate::components::{
    chart::{ChartSeries, chart, chart_hit_bands, chart_table, labels, series_color},
    chart_tooltip::{TooltipRow, chart_tooltip},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let weeks = labels(["Week 1", "Week 2", "Week 3", "Week 4", "Week 5"]);
    let rating = [1210.0, 1245.0, 1230.0, 1302.0, 1350.0];
    let layout = bar_layout(weeks.len(), &[&rating], &BarOptions::default());
    let color = series_color(2, None);

    let hovered = signal(cx, || None::<usize>);
    let base = element_id(cx);
    let ids: Vec<String> = (0..weeks.len())
        .map(|week| format!("{base}-{week}"))
        .collect();
    // Two rows of your own per week: the rating and the change since the week before.
    let rows: Vec<Vec<TooltipRow>> = rating
        .iter()
        .enumerate()
        .map(|(week, value)| {
            let change = match week {
                0 => "–".to_owned(),
                _ => format!("{:+}", value - rating[week - 1]),
            };
            vec![
                TooltipRow {
                    name: "Rating".to_owned(),
                    value: value.to_string(),
                    color: color.clone(),
                },
                TooltipRow {
                    name: "Change".to_owned(),
                    value: change,
                    color: "transparent".to_owned(),
                },
            ]
        })
        .collect();

    Ok(view! {
        chart(
            label: "Rating by week",
            attrs: attributes! { class="aspect-auto h-40" },
            <div class="relative min-h-0 flex-1" @mouseleave=$(|_e| hovered.set(None))>
                <svg
                    viewBox="0 0 100 100"
                    preserveAspectRatio="none"
                    aria-hidden="true"
                    class="absolute inset-0 size-full overflow-visible"
                >
                    for bar in &layout.bars {
                        <rect
                            x=(num(bar.rect.x))
                            y=(num(bar.rect.y))
                            width=(num(bar.rect.width))
                            height=(num(bar.rect.height))
                            style=(format!("fill:{color}"))
                        ></rect>
                    }
                </svg>
                chart_hit_bands(
                    hovered: hovered.clone(),
                    labels: weeks.clone(),
                    tooltip_ids: ids.clone()
                )
                for (index, (band, rows)) in layout.bands.iter().zip(rows).enumerate() {
                    // Anchor each panel beside its band, on the side with more room.
                    let opens_left = band.center > 50.0;
                    chart_tooltip(
                        hovered: hovered.clone(),
                        index: index,
                        id: ids[index].clone(),
                        title: weeks[index].clone(),
                        rows: rows,
                        x: if opens_left { band.start } else { band.start + band.size },
                        y: 50.0,
                        opens_left: opens_left
                    )
                }
            </div>
            chart_table(
                label: "Rating by week",
                category_header: "Week",
                categories: weeks.clone(),
                series: vec![ChartSeries::new("Rating", rating)]
            )
        )
    })
}
