use topcoat::{
    Result,
    context::Cx,
    router::{error::RouterErrorExt, module_param, page},
    view::{View, view},
};

use crate::{
    catalog::{self, ComponentDoc},
    guides,
    shell::{api_section, example_section, install_section, page_header, pager, section_heading},
};

module_param!(pub slug: String else not_found);

/// The guide pages, by slug.
enum Guide {
    Introduction,
    Installation,
    Theming,
    Geometry,
}

/// `/docs/<slug>`: a guide or a component's page.
#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let slug = slug(cx)?;
    let path = format!("/docs/{slug}");
    let page = match slug.as_str() {
        "introduction" => Ok(Guide::Introduction),
        "installation" => Ok(Guide::Installation),
        "theming" => Ok(Guide::Theming),
        "geometry" => Ok(Guide::Geometry),
        name => Err(catalog::component(name).ok_or_not_found()?),
    };

    Ok(view! {
        match page {
            Ok(Guide::Introduction) => guides::introduction(),
            Ok(Guide::Installation) => guides::installation(),
            Ok(Guide::Theming) => guides::theming(),
            Ok(Guide::Geometry) => guides::geometry(),
            Err(doc) => component_page(doc: doc),
        }
        pager(path: &path)
    })
}

/// A component's page: the playground, when it has one, then examples, installation
/// and API.
#[topcoat::view::component]
async fn component_page(cx: &Cx, doc: &'static ComponentDoc) -> Result<impl View> {
    Ok(view! {
        page_header(title: doc.title, description: doc.description)
        if let Some(playground) = doc.playground {
            section_heading(id: "playground", "Playground")
            <div data-slot="playground" class="min-w-0">(playground(cx))</div>
        }
        section_heading(id: "examples", "Examples")
        <div class="flex flex-col">
            #[key(example.id)]
            for example in doc.examples {
                example_section(example: example)
            }
        </div>
        install_section(doc: doc)
        api_section(doc: doc)
    })
}
