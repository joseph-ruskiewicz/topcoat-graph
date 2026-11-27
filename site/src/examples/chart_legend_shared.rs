use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::components::{
    bar_chart::bar_chart,
    chart::{ChartSeries, labels, series_color},
    chart_legend::chart_legend,
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let costs = labels(["0", "1", "2", "3", "4", "5", "6+"]);

    Ok(view! {
        <div class="flex w-full flex-col gap-4">
            // One legend for both charts. The colors are the ones each chart gives its
            // first and second series.
            chart_legend(
                items: vec![
                    ("Minions".to_owned(), series_color(0, None)),
                    ("Magic".to_owned(), series_color(1, None)),
                ]
            )
            <div class="grid gap-6 sm:grid-cols-2">
                bar_chart(
                    label: "Aggro deck by cost and type",
                    category_header: "Cost",
                    categories: costs.clone(),
                    series: vec![
                        ChartSeries::new("Minions", [2.0, 8.0, 9.0, 5.0, 2.0, 0.0, 0.0]),
                        ChartSeries::new("Magic", [0.0, 3.0, 4.0, 2.0, 1.0, 0.0, 0.0]),
                    ],
                    integer_ticks: true,
                    attrs: attributes! { class="aspect-auto h-40" }
                )
                bar_chart(
                    label: "Control deck by cost and type",
                    category_header: "Cost",
                    categories: costs,
                    series: vec![
                        ChartSeries::new("Minions", [0.0, 2.0, 3.0, 5.0, 4.0, 3.0, 3.0]),
                        ChartSeries::new("Magic", [1.0, 2.0, 5.0, 4.0, 2.0, 1.0, 1.0]),
                    ],
                    integer_ticks: true,
                    attrs: attributes! { class="aspect-auto h-40" }
                )
            </div>
        </div>
    })
}
