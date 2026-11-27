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
            label: "Mana cost distribution",
            category_header: "Cost",
            categories: labels(["0", "1", "2", "3", "4", "5", "6+"]),
            series: vec![ChartSeries::new("Cards", [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0])],
            integer_ticks: true
        )
    })
}
