use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use topcoat_graph::BarMode;

use crate::components::{
    bar_chart::bar_chart,
    chart::{ChartSeries, labels},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        bar_chart(
            label: "Cards by cost and type",
            category_header: "Cost",
            categories: labels(["0", "1", "2", "3", "4", "5", "6+"]),
            series: vec![
                ChartSeries::new("Minions", [1.0, 3.0, 7.0, 5.0, 3.0, 1.0, 0.0]),
                ChartSeries::new("Magic", [1.0, 2.0, 4.0, 2.0, 1.0, 0.0, 0.0]),
                ChartSeries::new("Artifacts", [0.0, 0.0, 1.0, 2.0, 0.0, 0.0, 0.0]),
            ],
            mode: BarMode::Stacked,
            integer_ticks: true
        )
    })
}
