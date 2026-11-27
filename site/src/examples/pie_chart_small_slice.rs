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
            label: "Games by platform",
            category_header: "Platform",
            value_header: "Games",
            slices: vec![PieSlice::new("Online", 9997.0), PieSlice::new("Tabletop", 3.0)],
            legend: true
        )
    })
}
