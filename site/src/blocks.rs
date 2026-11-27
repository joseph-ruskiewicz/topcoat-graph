//! `/blocks`: charts composed into larger pieces.

use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::{
    catalog::BLOCKS,
    shell::{example_section, page_header, pager},
};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        page_header(
            title: "Blocks",
            description: "Charts composed with cards and text into pieces of a dashboard. \
                Copy one into your app as a starting point."
        )
        <div class="mt-12 flex flex-col gap-14">
            #[key(block.id)]
            for block in BLOCKS {
                example_section(example: block, wide: true)
            }
        </div>
        pager(path: "/blocks")
    })
}
