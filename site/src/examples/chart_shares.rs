use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::components::chart::{ChartSeries, chart, chart_table, percent, series_color};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let types = ["Minions", "Magic", "Artifacts"];
    let cards = [18.0, 10.0, 4.0];
    let total: f64 = cards.iter().sum();

    Ok(view! {
        chart(
            label: "Deck by card type",
            attrs: attributes! { class="aspect-auto max-w-md gap-3" },
            // One bar split into shares, drawn with HTML.
            <div
                aria-hidden="true"
                class="flex h-3 w-full overflow-hidden rounded-full"
            >
                for (index, count) in cards.into_iter().enumerate() {
                    <div
                        style=(format!(
                            "width:{};background:{}",
                            percent(count / total * 100.0),
                            series_color(index, None),
                        ))
                    ></div>
                }
            </div>
            <div aria-hidden="true" class="flex justify-between gap-4">
                for (name, count) in types.into_iter().zip(cards) {
                    <span>
                        (name)
                        " "
                        <span class="font-medium text-foreground tabular-nums">
                            (count.to_string())
                        </span>
                    </span>
                }
            </div>
            chart_table(
                label: "Deck by card type",
                category_header: "Type",
                categories: types.map(String::from).to_vec(),
                series: vec![ChartSeries::new("Cards", cards)]
            )
        )
    })
}
