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
            label: "Mana spent and left over",
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
                ChartSeries::new("Spent", [1.0, 2.0, 3.0, 3.0, 5.0, 4.0, 6.0, 7.0]),
                ChartSeries::new("Left over", [0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 1.0, 1.0]),
            ],
            integer_ticks: true,
            legend: true
        )
    })
}
