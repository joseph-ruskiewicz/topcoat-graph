//! Bar chart examples.

use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, attributes, view},
};
use topcoat_graph::{BarMode, Orientation};

use crate::{
    components::{
        bar_chart::bar_chart,
        chart::{ChartSeries, labels},
    },
    shell::example,
};

/// A deck's cards by mana cost.
pub const MANA_CURVE: [f64; 7] = [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0];
pub const COSTS: [&str; 7] = ["0", "1", "2", "3", "4", "5", "6+"];

const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];

#[page]
async fn bar(cx: &Cx) -> Result<impl View> {
    let days: Vec<String> = (1..=30).map(|day| format!("Day {day}")).collect();
    let many_values: Vec<f64> = (0..30).map(|i| f64::from((i * 7 + 3) % 11 + 1)).collect();

    Ok(view! {
        <h1 class="text-2xl font-semibold">"Bar chart"</h1>

        example(
            title: "Single series",
            note: "A deck's cards by mana cost, with whole-number ticks.",
            bar_chart(
                label: "Mana cost distribution",
                category_header: "Cost",
                categories: labels(COSTS),
                series: vec![ChartSeries::new("Cards", MANA_CURVE)],
                integer_ticks: true
            )
        )
        example(
            title: "Grouped",
            note: "Several series side by side in each category, with a legend.",
            bar_chart(
                label: "Revenue and costs by quarter",
                category_header: "Quarter",
                categories: labels(QUARTERS),
                series: vec![
                    ChartSeries::new("Revenue", [42.0, 58.0, 51.0, 73.0]),
                    ChartSeries::new("Costs", [30.0, 35.0, 41.0, 44.0]),
                ],
                legend: true
            )
        )
        example(
            title: "Stacked",
            note: "Series on top of each other, so each bar's height is the total.",
            bar_chart(
                label: "Cards by cost and type",
                category_header: "Cost",
                categories: labels(COSTS),
                series: vec![
                    ChartSeries::new("Minions", [1.0, 3.0, 7.0, 5.0, 3.0, 1.0, 0.0]),
                    ChartSeries::new("Magic", [1.0, 2.0, 4.0, 2.0, 1.0, 0.0, 0.0]),
                    ChartSeries::new("Artifacts", [0.0, 0.0, 1.0, 2.0, 0.0, 0.0, 0.0]),
                ],
                mode: BarMode::Stacked,
                integer_ticks: true
            )
        )
        example(
            title: "Horizontal",
            note: "Categories down the side. Good for long labels.",
            bar_chart(
                label: "Cards by element",
                category_header: "Element",
                categories: labels(["Air", "Earth", "Fire", "Water", "No element"]),
                series: vec![ChartSeries::new("Cards", [14.0, 9.0, 11.0, 6.0, 3.0])],
                orientation: Orientation::Horizontal,
                integer_ticks: true,
                attrs: attributes! { class="aspect-auto h-56" }
            )
        )
        example(
            title: "Negative values",
            note: "Bars grow from the zero line in both directions.",
            bar_chart(
                label: "Net change by quarter",
                category_header: "Quarter",
                categories: labels(QUARTERS),
                series: vec![ChartSeries::new("Net change", [12.0, -8.0, 4.0, -15.0])]
            )
        )
        example(
            title: "Missing values",
            note: "A value of f64::NAN leaves a gap and is left out of the tooltip.",
            bar_chart(
                label: "Wins per week",
                category_header: "Week",
                categories: labels(["Week 1", "Week 2", "Week 3", "Week 4"]),
                series: vec![ChartSeries::new("Wins", [3.0, f64::NAN, 5.0, 2.0])],
                integer_ticks: true
            )
        )
        example(
            title: "Many categories",
            note: "With label_every: 5, one label in five is shown.",
            bar_chart(
                label: "Games played per day",
                category_header: "Day",
                categories: days,
                series: vec![ChartSeries::new("Games", many_values)],
                integer_ticks: true,
                label_every: 5
            )
        )
        example(
            title: "No data",
            note: "An empty chart keeps its frame and axis.",
            bar_chart(
                label: "An empty deck",
                category_header: "Cost",
                categories: Vec::new(),
                series: vec![ChartSeries::new("Cards", [])],
                integer_ticks: true
            )
        )
    })
}
