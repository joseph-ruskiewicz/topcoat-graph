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
            label: "Daily players",
            category_header: "Day",
            categories: labels(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]),
            series: vec![
                ChartSeries::new(
                    "Players",
                    [120.0, 135.0, 128.0, f64::NAN, 160.0, 210.0],
                ),
            ],
            dots: true,
            include_zero: true
        )
    })
}
