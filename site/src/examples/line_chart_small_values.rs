use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::components::{
    chart::{ChartSeries, labels},
    line_chart::line_chart,
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        line_chart(
            label: "Error rate by release",
            category_header: "Release",
            categories: labels(["1.0", "1.1", "1.2", "1.3"]),
            series: vec![
                ChartSeries::new("Error rate", [0.0034, 0.0021, 0.0012, 0.0015]),
            ]
        )
    })
}
