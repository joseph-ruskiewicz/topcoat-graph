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
            label: "Revenue and costs by quarter",
            category_header: "Quarter",
            categories: labels(["Q1", "Q2", "Q3", "Q4"]),
            series: vec![
                ChartSeries::new("Revenue", [42.0, 58.0, 51.0, 73.0]),
                ChartSeries::new("Costs", [30.0, 35.0, 41.0, 44.0]),
            ],
            legend: true
        )
    })
}
