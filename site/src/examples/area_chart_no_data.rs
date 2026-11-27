use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::components::{area_chart::area_chart, chart::ChartSeries};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        area_chart(
            label: "No turns yet",
            category_header: "Turn",
            categories: Vec::new(),
            series: vec![ChartSeries::new("Cards", [])],
            integer_ticks: true
        )
    })
}
