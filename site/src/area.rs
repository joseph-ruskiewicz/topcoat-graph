//! Area chart examples.

use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};
use topcoat_graph::Curve;

use crate::{
    components::{
        area_chart::area_chart,
        chart::{ChartSeries, labels},
    },
    shell::example,
};

const TURNS: [&str; 8] = [
    "Turn 1", "Turn 2", "Turn 3", "Turn 4", "Turn 5", "Turn 6", "Turn 7", "Turn 8",
];

#[page]
async fn area(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <h1 class="text-2xl font-semibold">"Area chart"</h1>

        example(
            title: "Single series",
            note: "The space under the line is filled down to zero.",
            area_chart(
                label: "Cards drawn per turn",
                category_header: "Turn",
                categories: labels(TURNS),
                series: vec![
                    ChartSeries::new(
                        "Cards drawn",
                        [7.0, 1.0, 2.0, 1.0, 3.0, 1.0, 2.0, 4.0],
                    ),
                ],
                integer_ticks: true
            )
        )
        example(
            title: "Several series",
            note: "Areas overlap, each down to zero, named in a legend.",
            area_chart(
                label: "Mana spent and left over",
                category_header: "Turn",
                categories: labels(TURNS),
                series: vec![
                    ChartSeries::new("Spent", [1.0, 2.0, 3.0, 3.0, 5.0, 4.0, 6.0, 7.0]),
                    ChartSeries::new(
                        "Left over",
                        [0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 1.0, 1.0],
                    ),
                ],
                integer_ticks: true,
                legend: true
            )
        )
        example(
            title: "Smooth",
            note: "A curve through every point that never overshoots the data.",
            area_chart(
                label: "Deck size over a draft",
                category_header: "Pick",
                categories: labels(
                    ["Pick 5", "Pick 10", "Pick 15", "Pick 20", "Pick 25", "Pick 30"],
                ),
                series: vec![
                    ChartSeries::new("Cards", [4.0, 9.0, 13.0, 16.0, 22.0, 27.0]),
                ],
                curve: Curve::Smooth,
                integer_ticks: true
            )
        )
        example(
            title: "Stacked",
            note: "Series on top of each other, so the top line is the total.",
            area_chart(
                label: "Cards in hand by type",
                category_header: "Turn",
                categories: labels(TURNS),
                series: vec![
                    ChartSeries::new("Minions", [3.0, 3.0, 4.0, 3.0, 2.0, 2.0, 1.0, 1.0]),
                    ChartSeries::new("Magic", [2.0, 2.0, 1.0, 2.0, 2.0, 1.0, 1.0, 0.0]),
                    ChartSeries::new(
                        "Artifacts",
                        [2.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0],
                    ),
                ],
                stacked: true,
                integer_ticks: true,
                legend: true
            )
        )
        example(
            title: "Missing values",
            note: "A value of f64::NAN breaks the area and is left out of the tooltip.",
            area_chart(
                label: "Cards drawn, with a gap",
                category_header: "Turn",
                categories: labels(["Turn 1", "Turn 2", "Turn 3", "Turn 4", "Turn 5"]),
                series: vec![
                    ChartSeries::new("Cards drawn", [7.0, 1.0, f64::NAN, 2.0, 3.0]),
                ],
                integer_ticks: true
            )
        )
        example(
            title: "No data",
            note: "An empty chart keeps its frame and axis.",
            area_chart(
                label: "No turns yet",
                category_header: "Turn",
                categories: Vec::new(),
                series: vec![ChartSeries::new("Cards", [])],
                integer_ticks: true
            )
        )
    })
}
