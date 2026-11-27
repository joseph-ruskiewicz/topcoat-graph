use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use topcoat_graph::Curve;

use crate::components::{
    area_chart::area_chart,
    chart::{ChartSeries, labels},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        area_chart(
            label: "Deck size over a draft",
            category_header: "Pick",
            categories: labels(
                ["Pick 5", "Pick 10", "Pick 15", "Pick 20", "Pick 25", "Pick 30"],
            ),
            series: vec![ChartSeries::new("Cards", [4.0, 9.0, 13.0, 16.0, 22.0, 27.0])],
            curve: Curve::Smooth,
            integer_ticks: true
        )
    })
}
