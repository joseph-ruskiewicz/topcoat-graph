use topcoat::{
    Result,
    context::Cx,
    view::{View, component, view},
};
use topcoat_graph::{BarOptions, bar_layout, fmt::num};

#[component]
pub async fn example(cx: &Cx) -> Result<impl View> {
    let counts = [2.0, 5.0, 12.0, 9.0, 4.0, 1.0, 0.0];
    let options = BarOptions {
        integer_ticks: true,
        ..Default::default()
    };
    let layout = bar_layout(counts.len(), &[&counts], &options);

    // No components: a plain SVG, stretched to 160 by 48 pixels, with a bar per count.
    Ok(view! {
        <svg
            role="img"
            aria-label="Cards by mana cost, from 0 to 6 or more: 2, 5, 12, 9, 4, 1 and 0"
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
            class="h-12 w-40 overflow-visible"
        >
            for bar in &layout.bars {
                <rect
                    x=(num(bar.rect.x))
                    y=(num(bar.rect.y))
                    width=(num(bar.rect.width))
                    height=(num(bar.rect.height))
                    rx="1"
                    class="fill-foreground"
                ></rect>
            }
            <line
                x1="0"
                x2="100"
                y1=(num(layout.zero))
                y2=(num(layout.zero))
                stroke-width="1"
                vector-effect="non-scaling-stroke"
                class="stroke-border"
            ></line>
        </svg>
    })
}
