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
        )
    })
}
