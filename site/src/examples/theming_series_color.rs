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
            label: "This season against the last",
            category_header: "Month",
            categories: labels(["Jan", "Feb", "Mar", "Apr", "May", "Jun"]),
            series: vec![
                ChartSeries::new(
                    "This season",
                    [1210.0, 1245.0, 1230.0, 1302.0, 1350.0, 1338.0],
                ).color("var(--primary)"),
                ChartSeries::new(
                    "Last season",
                    [1180.0, 1190.0, 1220.0, 1215.0, 1240.0, 1262.0],
                ).color("var(--muted-foreground)"),
            ],
            legend: true
        )
    })
}
