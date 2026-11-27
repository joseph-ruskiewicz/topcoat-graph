use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::components::{
    area_chart::area_chart,
    chart::{ChartSeries, labels},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        area_chart(
            label: "Cards in hand by type",
            category_header: "Turn",
            categories: labels(
                [
                    "Turn 1",
                    "Turn 2",
                    "Turn 3",
                    "Turn 4",
                    "Turn 5",
                    "Turn 6",
                    "Turn 7",
                    "Turn 8",
                ],
            ),
            series: vec![
                ChartSeries::new("Minions", [3.0, 3.0, 4.0, 3.0, 2.0, 2.0, 1.0, 1.0]),
                ChartSeries::new("Magic", [2.0, 2.0, 1.0, 2.0, 2.0, 1.0, 1.0, 0.0]),
                ChartSeries::new("Artifacts", [2.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0]),
            ],
            stacked: true,
            integer_ticks: true,
            legend: true
        )
    })
}
