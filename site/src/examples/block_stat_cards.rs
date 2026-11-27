use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};
use topcoat_graph::Curve;

use crate::components::{
    area_chart::area_chart,
    card::{card, card_content, card_description, card_header, card_title},
    chart::ChartSeries,
};

/// Games played each week, oldest first. The last four weeks are this month.
const WEEKLY_GAMES: [f64; 12] = [
    41.0, 38.0, 45.0, 52.0, 49.0, 47.0, 55.0, 58.0, 54.0, 61.0, 66.0, 63.0,
];

/// The other numbers: a label, this month, last month, and a unit after the value.
const STATS: [(&str, f64, f64, &str); 3] = [
    ("Win rate", 54.2, 51.8, "%"),
    ("Average game length", 18.5, 19.7, " min"),
    ("Decks built", 7.0, 4.0, ""),
];

/// The change from `before` to `now`: in points for a value that is already a
/// percentage, otherwise as a signed percentage of `before`.
fn change(now: f64, before: f64, unit: &str) -> String {
    if unit == "%" {
        format!("{:+.1} points from last month", now - before)
    } else {
        let percent = (now - before) / before * 100.0;
        format!("{percent:+.1}% from last month")
    }
}

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let this_month: f64 = WEEKLY_GAMES[8..].iter().sum();
    let last_month: f64 = WEEKLY_GAMES[4..8].iter().sum();
    let mut stats = vec![(
        "Games played".to_owned(),
        this_month.to_string(),
        change(this_month, last_month, ""),
    )];
    for (label, now, before, unit) in STATS {
        stats.push((
            label.to_owned(),
            format!("{now}{unit}"),
            change(now, before, unit),
        ));
    }

    Ok(view! {
        <div class="@container flex w-full flex-col gap-4">
            <div class="grid grid-cols-1 gap-4 @md:grid-cols-2 @3xl:grid-cols-4">
                for (label, value, delta) in &stats {
                    card(
                        attrs: attributes! { class="gap-2 py-5" data-slot="stat" },
                        card_header(
                            attrs: attributes! { class="px-5" },
                            card_description((label))
                        )
                        card_content(
                            attrs: attributes! { class="flex flex-col gap-1 px-5" },
                            <p class="text-2xl font-semibold tabular-nums">(value)</p>
                            <p class="text-xs text-muted-foreground">(delta)</p>
                        )
                    )
                }
            </div>
            card(
                card_header(
                    card_title("Games played")
                    card_description("Per week, over the last 12 weeks")
                )
                card_content(
                    area_chart(
                        label: "Games played per week, last 12 weeks",
                        category_header: "Week",
                        categories: (1..=12)
                            .map(|week| format!("Week {week}"))
                            .collect(),
                        series: vec![ChartSeries::new("Games", WEEKLY_GAMES)],
                        curve: Curve::Smooth,
                        integer_ticks: true,
                        label_every: 2,
                        attrs: attributes! { class="aspect-auto h-56" }
                    )
                )
            )
        </div>
    })
}
