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
        )
    })
}
