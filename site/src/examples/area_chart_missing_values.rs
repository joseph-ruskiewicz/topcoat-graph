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
            label: "Cards drawn, with a gap",
            category_header: "Turn",
            categories: labels(["Turn 1", "Turn 2", "Turn 3", "Turn 4", "Turn 5"]),
            series: vec![ChartSeries::new("Cards drawn", [7.0, 1.0, f64::NAN, 2.0, 3.0])],
            integer_ticks: true
        )
    })
}
