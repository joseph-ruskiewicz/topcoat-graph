use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::components::{bar_chart::bar_chart, chart::ChartSeries};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    // Thirty days, with a made-up number of games on each.
    let days: Vec<String> = (1..=30).map(|day| format!("Day {day}")).collect();
    let games: Vec<f64> = (0..30).map(|i| f64::from((i * 7 + 3) % 11 + 1)).collect();

    Ok(view! {
        bar_chart(
            label: "Games played per day",
            category_header: "Day",
            categories: days,
            series: vec![ChartSeries::new("Games", games)],
            integer_ticks: true,
            label_every: 5
        )
    })
}
