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
            label: "Cards by type",
            category_header: "Type",
            value_header: "Cards",
            slices: vec![
                PieSlice::new("Minions", 18.0),
                PieSlice::new("Magic", 10.0),
                PieSlice::new("Artifacts", 4.0),
            ]
        )
    })
}
