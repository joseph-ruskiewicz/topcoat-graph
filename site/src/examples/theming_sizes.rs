use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::components::{
    bar_chart::bar_chart,
    chart::{ChartSeries, labels},
    pie_chart::{PieSlice, pie_chart},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <div class="grid w-full items-center gap-8 sm:grid-cols-[2fr_1fr]">
            // A fixed height instead of the 16:9 shape.
            bar_chart(
                label: "Cards by cost, 10rem tall",
                category_header: "Cost",
                categories: labels(["0", "1", "2", "3", "4", "5", "6+"]),
                series: vec![
                    ChartSeries::new("Cards", [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0]),
                ],
                integer_ticks: true,
                attrs: attributes! { class="aspect-auto h-40" }
            )
            // A pie at most 8rem wide instead of 18rem.
            pie_chart(
                label: "Cards by type, 8rem wide",
                category_header: "Type",
                value_header: "Cards",
                slices: vec![
                    PieSlice::new("Minions", 18.0),
                    PieSlice::new("Magic", 10.0),
                    PieSlice::new("Artifacts", 4.0),
                ],
                attrs: attributes! { class="max-w-32" }
            )
        </div>
    })
}
