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
            label: "Cards by rarity",
            category_header: "Rarity",
            value_header: "Cards",
            slices: vec![PieSlice::new("Ordinary", 40.0)]
        )
    })
}
