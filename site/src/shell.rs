//! Pieces the gallery pages share.

use topcoat::{
    Result,
    context::Cx,
    view::{Child, View, component, view},
};

/// One example on a gallery page: a heading, a line about it, and the chart in a card.
#[component]
pub async fn example(
    cx: &Cx,
    /// The example's heading.
    title: &str,
    /// What the example shows.
    #[default]
    note: &str,
    /// The chart.
    #[default]
    child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <section class="flex flex-col gap-3">
            <div>
                <h2 class="text-base font-medium">(title)</h2>
                if !note.is_empty() {
                    <p class="text-sm text-muted-foreground">(note)</p>
                }
            </div>
            <div class="rounded-lg border bg-card p-4">(child)</div>
        </section>
    })
}
