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
            label: "Win rate by week",
            category_header: "Week",
            categories: labels(
                [
                    "Week 1",
                    "Week 2",
                    "Week 3",
                    "Week 4",
                    "Week 5",
                    "Week 6",
                    "Week 7",
                    "Week 8",
                ],
            ),
            series: vec![
                ChartSeries::new(
                    "Win rate",
                    [48.0, 52.0, 51.0, 57.0, 55.0, 61.0, 59.0, 64.0],
                ),
            ]
        )
    })
}
