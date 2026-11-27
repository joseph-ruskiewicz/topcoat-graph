use topcoat::{
    Result,
    context::Cx,
    view::{View, attributes, component, view},
};

use crate::components::{
    chart::{ChartSeries, chart, chart_table, percent, series_color},
    chart_legend::chart_legend,
};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let results = ["Wins", "Losses", "Draws"];
    let games = [18.0, 12.0, 2.0];
    let total: f64 = games.iter().sum();
    // Each entry is a name and the CSS color its marks use.
    let items: Vec<(String, String)> = results
        .iter()
        .enumerate()
        .map(|(index, name)| (name.to_string(), series_color(index, None)))
        .collect();

    Ok(view! {
        chart(
            label: "Games by result",
            attrs: attributes! { class="aspect-auto max-w-md gap-3" },
            <div aria-hidden="true" class="flex h-3 w-full gap-0.5">
                for (index, count) in games.into_iter().enumerate() {
                    <div
                        class="rounded-sm"
                        style=(format!(
                            "width:{};background:{}",
                            percent(count / total * 100.0),
                            series_color(index, None),
                        ))
                    ></div>
                }
            </div>
            chart_legend(items: items)
            chart_table(
                label: "Games by result",
                category_header: "Result",
                categories: results.map(String::from).to_vec(),
                series: vec![ChartSeries::new("Games", games)]
            )
        )
    })
}
