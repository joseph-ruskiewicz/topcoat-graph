use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use topcoat_graph::Orientation;

use crate::components::{
    bar_chart::bar_chart,
    chart::{ChartSeries, labels},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        bar_chart(
            label: "Cards by element",
            category_header: "Element",
            categories: labels(["Air", "Earth", "Fire", "Water", "No element"]),
            series: vec![ChartSeries::new("Cards", [14.0, 9.0, 11.0, 6.0, 3.0])],
            orientation: Orientation::Horizontal,
            integer_ticks: true,
            attrs: attributes! { class="aspect-auto h-56" }
        )
    })
}
