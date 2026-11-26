//! Pie and donut chart examples.

use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::{
    components::pie_chart::{PieSlice, pie_chart},
    shell::{example, snippet},
};

/// This page's source, which each example's code is cut from.
const SOURCE: &str = include_str!("pie.rs");

#[page]
async fn pie(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <h1 class="text-2xl font-semibold">"Pie chart"</h1>

        example(
            title: "Pie",
            note: "One slice per value, clockwise from twelve o'clock.",
            code: snippet(SOURCE, "pie"),
            // example: pie
            pie_chart(
                label: "Cards by type",
                category_header: "Type",
                value_header: "Cards",
                slices: vec![
                    PieSlice::new("Minions", 18.0),
                    PieSlice::new("Magic", 10.0),
                    PieSlice::new("Artifacts", 4.0),
                ]
            ) // end
        )
        example(
            title: "Donut",
            note: "A hole in the middle, with the total in it.",
            code: snippet(SOURCE, "donut"),
            // example: donut
            pie_chart(
                label: "Games by result",
                category_header: "Result",
                value_header: "Games",
                slices: vec![
                    PieSlice::new("Wins", 18.0),
                    PieSlice::new("Losses", 12.0),
                    PieSlice::new("Draws", 2.0),
                ],
                donut: true,
                legend: true,
                <span class="text-2xl font-semibold text-foreground tabular-nums">
                    "32"
                </span>
                <span>"games"</span>
            ) // end
        )
        example(
            title: "A zero among the values",
            note: "Earth has no cards, so it gets no slice. It stays in the legend, and Fire keeps its color.",
            code: snippet(SOURCE, "a-zero-among-the-values"),
            // example: a-zero-among-the-values
            pie_chart(
                label: "Cards by element",
                category_header: "Element",
                value_header: "Cards",
                slices: vec![
                    PieSlice::new("Air", 9.0),
                    PieSlice::new("Water", 5.0),
                    PieSlice::new("Earth", 0.0),
                    PieSlice::new("Fire", 7.0),
                ],
                legend: true
            ) // end
        )
        example(
            title: "A single slice",
            note: "One value with everything: a full circle.",
            code: snippet(SOURCE, "a-single-slice"),
            // example: a-single-slice
            pie_chart(
                label: "Cards by rarity",
                category_header: "Rarity",
                value_header: "Cards",
                slices: vec![PieSlice::new("Ordinary", 40.0)]
            ) // end
        )
        example(
            title: "No data",
            note: "When every value is zero there are no slices, only an empty ring.",
            code: snippet(SOURCE, "no-data"),
            // example: no-data
            pie_chart(
                label: "No games played",
                category_header: "Result",
                value_header: "Games",
                slices: vec![
                    PieSlice::new("Wins", 0.0),
                    PieSlice::new("Losses", 0.0),
                    PieSlice::new("Draws", 0.0),
                ],
                donut: true,
                <span class="text-2xl font-semibold text-foreground tabular-nums">
                    "0"
                </span>
                <span>"games"</span>
            ) // end
        )
    })
}
