use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use topcoat_graph::Curve;

use crate::components::{
    chart::{ChartSeries, labels},
    line_chart::line_chart,
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        line_chart(
            label: "Rating over a season",
            category_header: "Month",
            categories: labels(["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"]),
            series: vec![
                ChartSeries::new(
                    "Rating",
                    [1210.0, 1245.0, 1230.0, 1302.0, 1350.0, 1338.0, 1395.0, 1420.0],
                ),
            ],
            curve: Curve::Smooth
        )
    })
}
