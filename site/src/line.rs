//! Line chart examples.

use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};
use topcoat_graph::Curve;

use crate::{
    components::{
        chart::{ChartSeries, labels},
        line_chart::line_chart,
    },
    shell::{example, snippet},
};

/// This page's source, which each example's code is cut from.
const SOURCE: &str = include_str!("line.rs");

const WEEKS: [&str; 6] = ["Week 1", "Week 2", "Week 3", "Week 4", "Week 5", "Week 6"];

#[page]
async fn line(cx: &Cx) -> Result<impl View> {
    let months = labels(["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"]);

    Ok(view! {
        <h1 class="text-2xl font-semibold">"Line chart"</h1>

        example(
            title: "Single series",
            note: "One value per week, joined by straight segments.",
            code: snippet(SOURCE, "single-series"),
            // example: single-series
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
            ) // end
        )
        example(
            title: "Several series",
            note: "One line per series, named in a legend.",
            code: snippet(SOURCE, "several-series"),
            // example: several-series
            line_chart(
                label: "Games won and lost",
                category_header: "Week",
                categories: labels(WEEKS),
                series: vec![
                    ChartSeries::new("Wins", [3.0, 5.0, 4.0, 7.0, 6.0, 8.0]),
                    ChartSeries::new("Losses", [4.0, 3.0, 4.0, 2.0, 3.0, 2.0]),
                ],
                integer_ticks: true,
                include_zero: true,
                legend: true
            ) // end
        )
        example(
            title: "Smooth",
            note: "A curve through every point that never overshoots the data.",
            code: snippet(SOURCE, "smooth"),
            // example: smooth
            line_chart(
                label: "Rating over a season",
                category_header: "Month",
                categories: months,
                series: vec![
                    ChartSeries::new(
                        "Rating",
                        [1210.0, 1245.0, 1230.0, 1302.0, 1350.0, 1338.0, 1395.0, 1420.0],
                    ),
                ],
                curve: Curve::Smooth
            ) // end
        )
        example(
            title: "Dots",
            note: "A dot on every value. The day without a value has neither dot nor line.",
            code: snippet(SOURCE, "dots"),
            // example: dots
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
            ) // end
        )
        example(
            title: "Small values",
            note: "Values keep their precision in the tooltip and the data table.",
            code: snippet(SOURCE, "small-values"),
            // example: small-values
            line_chart(
                label: "Error rate by release",
                category_header: "Release",
                categories: labels(["1.0", "1.1", "1.2", "1.3"]),
                series: vec![
                    ChartSeries::new("Error rate", [0.0034, 0.0021, 0.0012, 0.0015]),
                ]
            ) // end
        )
        example(
            title: "Missing values",
            note: "A value of f64::NAN breaks the line and is left out of the tooltip.",
            code: snippet(SOURCE, "missing-values"),
            // example: missing-values
            line_chart(
                label: "Wins per week, with a gap",
                category_header: "Week",
                categories: labels(["Week 1", "Week 2", "Week 3", "Week 4", "Week 5"]),
                series: vec![ChartSeries::new("Wins", [3.0, 5.0, f64::NAN, 4.0, 6.0])],
                integer_ticks: true
            ) // end
        )
        example(
            title: "No data",
            note: "An empty chart keeps its frame and axis.",
            code: snippet(SOURCE, "no-data"),
            // example: no-data
            line_chart(
                label: "No games yet",
                category_header: "Week",
                categories: Vec::new(),
                series: vec![ChartSeries::new("Games", [])],
                integer_ticks: true
            ) // end
        )
    })
}
