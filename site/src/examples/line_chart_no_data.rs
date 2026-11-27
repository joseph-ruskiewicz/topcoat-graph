use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::components::{chart::ChartSeries, line_chart::line_chart};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        line_chart(
            label: "No games yet",
            category_header: "Week",
            categories: Vec::new(),
            series: vec![ChartSeries::new("Games", [])],
            integer_ticks: true
        )
    })
}
