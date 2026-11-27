use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use topcoat_graph::Orientation;

use crate::components::{
    bar_chart::bar_chart,
    card::{card, card_content, card_description, card_header, card_title},
    chart::ChartSeries,
};

/// How many winning decks ran each card this season, in no particular order.
const CARDS: [(&str, f64); 7] = [
    ("Lantern of the Drowned Choir", 64.0),
    ("Ashen Pact", 112.0),
    ("Warden of the Seventh Gate", 87.0),
    ("Quickfoot Scout", 41.0),
    ("Archivist of Forgotten Spells", 95.0),
    ("Stormcaller's Last Rite", 73.0),
    ("Thornback Boar", 58.0),
];

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    // Most played first.
    let mut ranked = CARDS.to_vec();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));

    Ok(view! {
        card(
            attrs: attributes! { class="w-full max-w-xl" },
            card_header(
                card_title("Most played cards")
                card_description("Winning decks that ran each card this season")
            )
            card_content(
                bar_chart(
                    label: "Most played cards in winning decks",
                    category_header: "Card",
                    categories: ranked
                        .iter()
                        .map(|(name, _)| name.to_string())
                        .collect(),
                    series: vec![
                        ChartSeries::new(
                            "Decks",
                            ranked.iter().map(|(_, decks)| *decks).collect::<Vec<_>>(),
                        ),
                    ],
                    orientation: Orientation::Horizontal,
                    integer_ticks: true,
                    attrs: attributes! { class="aspect-auto h-80" }
                )
            )
        )
    })
}
