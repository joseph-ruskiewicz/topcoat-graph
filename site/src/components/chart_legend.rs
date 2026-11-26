use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};

/// A row of color swatches and names, one per series or slice, that wraps onto more
/// lines when it runs out of room.
///
/// The legend is visible text that assistive technology reads too, so a chart with a
/// legend announces its series' names before its data table.
///
/// ```ignore
/// view! {
///     chart_legend(items: vec![
///         ("Wins".to_owned(), series_color(0, None)),
///         ("Losses".to_owned(), series_color(1, None)),
///     ])
/// }
/// ```
#[component]
pub async fn chart_legend(
    cx: &Cx,
    /// Each entry's name and CSS color, in order.
    items: Vec<(String, String)>,
) -> Result<impl View> {
    Ok(view! {
        <ul
            data-chart="legend"
            class="flex flex-wrap items-center justify-center gap-x-4 gap-y-1"
        >
            for (name, color) in items {
                <li data-chart="legend-item" class="flex items-center gap-1.5">
                    <span
                        aria-hidden="true"
                        class="size-2.5 shrink-0 rounded-[2px]"
                        style=(format!("background:{color}"))
                    ></span>
                    <span>(name)</span>
                </li>
            }
        </ul>
    })
}
