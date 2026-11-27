use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::components::{bar_chart::bar_chart, chart::ChartSeries};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        bar_chart(
            label: "An empty deck",
            category_header: "Cost",
            categories: Vec::new(),
            series: vec![ChartSeries::new("Cards", [])],
            integer_ticks: true
        )
    })
}
