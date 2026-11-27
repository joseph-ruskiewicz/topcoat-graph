use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

use crate::components::pie_chart::{PieSlice, pie_chart};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
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
            <span class="text-2xl font-semibold text-foreground tabular-nums">"0"</span>
            <span>"games"</span>
        )
    })
}
