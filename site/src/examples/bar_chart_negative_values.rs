use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::components::{
    bar_chart::bar_chart,
    chart::{ChartSeries, labels},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        bar_chart(
            label: "Net change by quarter",
            category_header: "Quarter",
            categories: labels(["Q1", "Q2", "Q3", "Q4"]),
            series: vec![ChartSeries::new("Net change", [12.0, -8.0, 4.0, -15.0])]
        )
    })
}
