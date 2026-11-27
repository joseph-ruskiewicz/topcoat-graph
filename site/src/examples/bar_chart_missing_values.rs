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
            label: "Wins per week",
            category_header: "Week",
            categories: labels(["Week 1", "Week 2", "Week 3", "Week 4"]),
            series: vec![ChartSeries::new("Wins", [3.0, f64::NAN, 5.0, 2.0])],
            integer_ticks: true
        )
    })
}
