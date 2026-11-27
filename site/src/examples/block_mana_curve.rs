use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::components::{
    bar_chart::bar_chart,
    card::{card, card_content, card_description, card_header, card_title},
    chart::{ChartSeries, labels},
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let curve = [2.0, 8.0, 11.0, 9.0, 6.0, 3.0, 1.0];
    let total: f64 = curve.iter().sum();

    Ok(view! {
        card(
            attrs: attributes! { class="w-full max-w-md" },
            card_header(
                card_title("Mana curve")
                card_description((format!("Aggro deck, {total} cards")))
            )
            card_content(
                bar_chart(
                    label: "Mana curve of an aggro deck",
                    category_header: "Cost",
                    categories: labels(["0", "1", "2", "3", "4", "5", "6+"]),
                    series: vec![ChartSeries::new("Cards", curve)],
                    integer_ticks: true,
                    attrs: attributes! { class="aspect-auto h-48" }
                )
            )
        )
    })
}
