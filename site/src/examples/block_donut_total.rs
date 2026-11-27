use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::components::{
    card::{card, card_content, card_description, card_header, card_title},
    chart::series_color,
    pie_chart::{PieSlice, pie_chart},
};

/// Games played with each archetype this season.
const ARCHETYPES: [(&str, f64); 4] = [
    ("Aggro", 47.0),
    ("Midrange", 38.0),
    ("Control", 29.0),
    ("Combo", 13.0),
];

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let total: f64 = ARCHETYPES.iter().map(|(_, games)| games).sum();

    Ok(view! {
        card(
            attrs: attributes! { class="@container w-full max-w-2xl" },
            card_header(
                card_title("Games by archetype")
                card_description("This season, all formats")
            )
            card_content(
                attrs: attributes! {
                    class="flex flex-col items-center gap-6 @lg:flex-row @lg:items-center"
                },
                pie_chart(
                    label: "Games by archetype this season",
                    category_header: "Archetype",
                    value_header: "Games",
                    slices: ARCHETYPES
                        .iter()
                        .map(|(name, games)| PieSlice::new(*name, *games))
                        .collect(),
                    donut: true,
                    legend: true,
                    attrs: attributes! { class="max-w-56 shrink-0" },
                    <span class="text-2xl font-semibold text-foreground tabular-nums">
                        (total)
                    </span>
                    <span>"games"</span>
                )
                <div data-slot="shares" class="w-full min-w-0 flex-1">
                    <table class="w-full text-sm">
                        <thead>
                            <tr
                                class="border-b border-border text-left text-muted-foreground"
                            >
                                <th scope="col" class="pb-2 font-medium">"Archetype"</th>
                                <th scope="col" class="pb-2 text-right font-medium">
                                    "Games"
                                </th>
                                <th scope="col" class="pb-2 text-right font-medium">
                                    "Share"
                                </th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            for (index, (name, games)) in ARCHETYPES.iter().enumerate() {
                                <tr>
                                    <th scope="row" class="py-2 text-left font-normal">
                                        <span class="flex items-center gap-2">
                                            <span
                                                aria-hidden="true"
                                                class="size-2.5 shrink-0 rounded-sm"
                                                style=(format!("background:{}", series_color(index, None)))
                                            ></span>
                                            <span>(name)</span>
                                        </span>
                                    </th>
                                    <td class="py-2 text-right tabular-nums">(games)</td>
                                    <td
                                        class="py-2 text-right tabular-nums text-muted-foreground"
                                    >
                                        (format!("{:.1}%", games / total * 100.0))
                                    </td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>
            )
        )
    })
}
