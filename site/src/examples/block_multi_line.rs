use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use topcoat_graph::Curve;

use crate::components::{
    card::{card, card_content, card_description, card_footer, card_header, card_title},
    chart::{ChartSeries, labels},
    line_chart::line_chart,
};

const WEEKS: [&str; 8] = [
    "Week 1", "Week 2", "Week 3", "Week 4", "Week 5", "Week 6", "Week 7", "Week 8",
];

/// Each deck's win rate in percent, one value per week.
const DECKS: [(&str, [f64; 8]); 3] = [
    (
        "Ember Rush",
        [48.0, 51.0, 50.0, 55.0, 57.0, 56.0, 60.0, 62.0],
    ),
    (
        "Tidal Control",
        [55.0, 54.0, 56.0, 53.0, 52.0, 54.0, 51.0, 50.0],
    ),
    (
        "Grove Ramp",
        [44.0, 46.0, 45.0, 47.0, 49.0, 48.0, 50.0, 51.0],
    ),
];

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    // The deck whose win rate changed most from the first week to the last.
    let (name, rates) = DECKS
        .iter()
        .max_by(|a, b| {
            let change = |rates: &[f64; 8]| rates[7] - rates[0];
            change(&a.1).total_cmp(&change(&b.1))
        })
        .expect("there are decks");
    let (first, last) = (rates[0], rates[7]);
    let trend = if last >= first {
        format!("{name} rose the most, from {first}% to {last}% in eight weeks.")
    } else {
        format!("No deck rose. {name} fell least, from {first}% to {last}%.")
    };

    Ok(view! {
        card(
            attrs: attributes! { class="w-full max-w-2xl" },
            card_header(
                card_title("Win rate by deck")
                card_description("Share of games won each week")
            )
            card_content(
                line_chart(
                    label: "Win rate by deck over eight weeks",
                    category_header: "Week",
                    categories: labels(WEEKS),
                    series: DECKS
                        .iter()
                        .map(|(name, rates)| ChartSeries::new(*name, *rates))
                        .collect(),
                    curve: Curve::Smooth,
                    dots: true,
                    legend: true,
                    attrs: attributes! { class="aspect-auto h-64" }
                )
            )
            card_footer(
                attrs: attributes! { class="text-sm text-muted-foreground" data-slot="trend" },
                (trend)
            )
        )
    })
}
