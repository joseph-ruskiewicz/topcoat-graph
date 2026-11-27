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
            label: "Wins per week, with a gap",
            category_header: "Week",
            categories: labels(["Week 1", "Week 2", "Week 3", "Week 4", "Week 5"]),
            series: vec![ChartSeries::new("Wins", [3.0, 5.0, f64::NAN, 4.0, 6.0])],
            integer_ticks: true
        )
    })
}
