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
            label: "Games won and lost",
            category_header: "Week",
            categories: labels(
                ["Week 1", "Week 2", "Week 3", "Week 4", "Week 5", "Week 6"],
            ),
            series: vec![
                ChartSeries::new("Wins", [3.0, 5.0, 4.0, 7.0, 6.0, 8.0]),
                ChartSeries::new("Losses", [4.0, 3.0, 4.0, 2.0, 3.0, 2.0]),
            ],
            integer_ticks: true,
            include_zero: true,
            legend: true
        )
    })
}
