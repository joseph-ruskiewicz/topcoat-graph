//! The docs shell: the sidebar, the content column, and the parts every docs page is
//! made of. These are site components, not registry ones.

use topcoat::{
    Result,
    context::Cx,
    cookie::{Cookie, Cookies, SameSite, cookies, time::Duration},
    icon::{icon, iconify::iconify_icon},
    runtime::{Event, Signal, link, procedure, signal},
    view::{Attributes, Child, StaticClass, Unescaped, View, attributes, class, component, view},
};

use crate::{
    GITHUB_URL, api,
    catalog::{self, ComponentDoc, Example, Group},
    code::{code_block, copy_button},
    components::{
        button::{ButtonSize, ButtonVariant, button},
        sheet::{SheetSide, sheet, sheet_content},
    },
};

/// The name of the cookie that remembers the theme.
const THEME_COOKIE: &str = "theme";

/// Whether the request's theme cookie asks for the dark theme. The layout reads it so
/// the first HTML already has the right class, with no flash of the other theme.
pub fn dark_from_cookie(cx: &Cx) -> bool {
    cookies(cx)
        .get(THEME_COOKIE)
        .is_some_and(|cookie| cookie.value() == "dark")
}

/// Remembers the reader's theme for a year. The procedure's response carries the
/// `Set-Cookie` header, so no navigation is needed.
#[procedure]
async fn save_theme(cx: &Cx, dark: bool) -> Result<()> {
    cookies(cx).add(
        Cookie::build((THEME_COOKIE, if dark { "dark" } else { "light" }))
            .path("/")
            .max_age(Duration::days(365))
            .same_site(SameSite::Lax)
            .http_only(true),
    );
    Ok(())
}

/// The sidebar and the content column around a page.
#[component]
pub async fn shell(
    cx: &Cx,
    path: &str,
    dark: Signal<bool>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let menu = signal(cx, || false);
    let wide = path == "/blocks";
    let sheet_dark = dark.clone();

    Ok(view! {
        // Narrow screens: a bar with the menu button; the sidebar is a sheet.
        <header
            class="sticky top-0 z-30 flex h-14 items-center gap-2 border-b border-dashed border-border bg-background px-3 lg:hidden"
        >
            button(
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Icon,
                attrs: attributes! {
                    type="button"
                    aria-label="Open menu"
                    aria-haspopup="dialog"
                    :aria-expanded=$(if menu.get() { "true" } else { "false" })
                    @click=$(|_e: Event| menu.set(true))
                },
                icon(data: iconify_icon!("lucide:menu"))
            )
            logo()
        </header>
        sheet(
            open: $(menu.get()),
            attrs: attributes! {
                aria-label="Menu"
                @keydown=$(|_e: Event| {
                    // Escape closes the menu, and so does a click on the backdrop: a click
                    // that lands on the dialog itself, not on the panel inside it.
                    if raw!("cx.hydrate(${_e}.inner.key === 'Escape')", false) {
                        menu.set(false);
                    }
                })
                @click=$(|_e: Event| {
                    if raw!(
                        "cx.hydrate(${_e}.inner.target === ${_e}.inner.currentTarget)",
                        false,
                    ) {
                        menu.set(false);
                    }
                })
            },
            sheet_content(
                side: SheetSide::Left,
                attrs: attributes! {
                    class="w-72 gap-0 p-0"
                    @click=$(|_e: Event| {
                        // Following a link closes the menu; the next page renders with it
                        // closed.
                        if raw!(
                            "cx.hydrate(${_e}.inner.target.closest('a') !== null)",
                            false,
                        ) {
                            menu.set(false);
                        }
                    })
                },
                <div class="flex items-center justify-between px-5 pt-5 pb-4">
                    logo()
                    button(
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        attrs: attributes! {
                            type="button"
                            aria-label="Close menu"
                            @click=$(|_e: Event| menu.set(false))
                        },
                        icon(data: iconify_icon!("lucide:x"))
                    )
                </div>
                sidebar_body(path: path, dark: sheet_dark)
            )
        )

        // Wide screens: a fixed sidebar.
        <aside
            class="fixed inset-y-0 left-0 z-20 hidden w-60 flex-col border-r border-dashed border-border bg-background lg:flex"
        >
            <div class="px-5 pt-8 pb-12">logo()</div>
            sidebar_body(path: path, dark: dark)
        </aside>

        <div class="lg:pl-60">
            <main
                class=(class!(
                    "mx-auto w-full min-w-0 px-4 pt-10 pb-24 sm:px-8 lg:pt-24 xl:mx-0 xl:ml-60 xl:px-0",
                    if wide { "max-w-5xl" } else { "max-w-3xl" },
                ))
            >
                (child)
            </main>
        </div>
    })
}

/// The mark and the wordmark, linking home.
#[component]
async fn logo(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        link(
            href: "/",
            attrs: attributes! {
                class="flex w-fit items-center gap-2 rounded-md text-base font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring"
            },
            <span
                aria-hidden="true"
                class="flex size-6 items-center justify-center rounded-md bg-foreground text-background"
            >
                icon(
                    data: iconify_icon!("lucide:chart-column"),
                    attrs: attributes! { class="size-4" }
                )
            </span>
            "topcoat-graph"
        )
    })
}

/// The link groups and the theme toggle, shared by the sidebar and the menu sheet.
#[component]
async fn sidebar_body(cx: &Cx, path: &str, dark: Signal<bool>) -> Result<impl View> {
    Ok(view! {
        <nav
            aria-label="Docs"
            class="flex min-h-0 flex-1 flex-col gap-8 overflow-y-auto px-3 pb-6"
        >
            nav_group(
                label: "Get Started",
                nav_link(href: "/", current: path == "/", "Home")
                for (href, title) in [
                    ("/docs/introduction", "Introduction"),
                    ("/docs/installation", "Installation"),
                    ("/docs/theming", "Theming"),
                ] {
                    nav_link(href: href, current: path == href, (title))
                }
            )
            nav_group(label: "Charts", component_links(path: path, group: Group::Chart))
            nav_group(label: "Parts", component_links(path: path, group: Group::Part))
            nav_group(
                label: "Blocks",
                nav_link(href: "/blocks", current: path == "/blocks", "Blocks")
            )
            nav_group(
                label: "Resources",
                nav_link(
                    href: "/docs/geometry",
                    current: path == "/docs/geometry",
                    "Geometry crate"
                )
                nav_link(href: "/llms.txt", plain: true, "llms.txt")
                nav_link(href: GITHUB_URL, plain: true, "GitHub")
            )
        </nav>
        <div class="border-t border-dashed border-border px-3 py-3">
            <button
                type="button"
                class=(NAV_LINK)
                @click=$(async |_e: Event| {
                    // The theme switches at once; the cookie is saved in the background.
                    dark.toggle();
                    save_theme(dark.get()).await;
                })
            >
                <span class="contents" :hidden=$(!dark.get())>
                    icon(data: iconify_icon!("lucide:sun"))
                    "Light mode"
                </span>
                <span class="contents" :hidden=$(dark.get())>
                    icon(data: iconify_icon!("lucide:moon"))
                    "Dark mode"
                </span>
            </button>
        </div>
    })
}

/// The sidebar links to the component pages of one group, in catalog order.
#[component]
async fn component_links(cx: &Cx, path: &str, group: Group) -> Result<impl View> {
    Ok(view! {
        for doc in catalog::COMPONENTS.iter().filter(|doc| doc.group == group) {
            let href = format!("/docs/{}", doc.name);
            nav_link(href: &href, current: path == href, (doc.title))
        }
    })
}

/// A labelled group of sidebar links.
#[component]
async fn nav_group(cx: &Cx, label: &str, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <div class="flex flex-col">
            <span class="px-2 pb-1.5 text-sm font-medium text-muted-foreground">
                (label)
            </span>
            <ul class="flex flex-col gap-0.5">(child)</ul>
        </div>
    })
}

/// Classes for a sidebar link or button.
const NAV_LINK: StaticClass = class!(
    "flex h-8 w-full items-center gap-2 rounded-lg px-2 text-left text-sm text-foreground/80",
    "outline-none transition-colors hover:bg-foreground/5 hover:text-foreground",
    "focus-visible:ring-2 focus-visible:ring-ring",
    "aria-[current=page]:bg-foreground/5 aria-[current=page]:font-medium",
    "aria-[current=page]:text-foreground",
);

/// A sidebar link. `plain` links are ordinary links, for files and other sites.
#[component]
async fn nav_link(
    cx: &Cx,
    href: &str,
    #[default] current: bool,
    #[default] plain: bool,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <li>
            if plain {
                <a href=(href) class=(NAV_LINK)>(child)</a>
            } else {
                link(
                    href: href,
                    attrs: attributes! {
                        class=(NAV_LINK)
                        aria-current=(current.then_some("page"))
                    },
                    (child)
                )
            }
        </li>
    })
}

/// A page's title and one-line description.
#[component]
pub async fn page_header(cx: &Cx, title: &str, #[default] description: &str) -> Result<impl View> {
    Ok(view! {
        <header class="flex flex-col gap-3">
            <h1 class="text-2xl leading-8 font-semibold tracking-tight">(title)</h1>
            if !description.is_empty() {
                <p class="text-base text-muted-foreground">(description)</p>
            }
        </header>
    })
}

/// A section heading on a docs page.
#[component]
pub async fn section_heading(
    cx: &Cx,
    #[default] id: &str,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <h2
            id=((!id.is_empty()).then_some(id))
            class="mt-12 mb-5 scroll-mt-20 text-xl leading-7 font-medium tracking-tight"
        >
            (child)
        </h2>
    })
}

/// A bordered panel with a live example centered in it. A `wide` panel is for blocks:
/// a tinted background, so a card stands out from it.
#[component]
pub async fn preview_panel(
    cx: &Cx,
    #[default] wide: bool,
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    Ok(view! {
        <div
            data-slot="preview"
            class=(class!(
                "flex w-full min-w-0 items-center justify-center overflow-hidden rounded-lg",
                "border border-border p-4 sm:p-10",
                if wide {
                    "min-h-[420px] bg-foreground/[0.025]"
                } else {
                    "min-h-[350px]"
                },
                attrs.remove("class"),
            ))
            (attrs)
        >
            <div class="flex w-full min-w-0 flex-col items-center">(child)</div>
        </div>
    })
}

/// Source in a code block, highlighted, with a copy button, scrolling past a height.
/// The language is Rust unless given.
#[component]
pub async fn code_panel(
    cx: &Cx,
    source: &str,
    #[default] language: Option<&str>,
) -> Result<impl View> {
    Ok(view! {
        <div class="code-panel min-w-0">
            code_block(
                language: language.unwrap_or("rust"),
                code: source,
                attrs: attributes! {
                    class="[&_pre]:max-h-[650px] [&_pre]:overflow-y-auto [&_pre]:text-[13px] [&_pre]:leading-[1.75]"
                }
            )
        </div>
    })
}

/// A short command, such as `topcoat ui add`, with a copy button.
#[component]
pub async fn command(cx: &Cx, text: &str) -> Result<impl View> {
    Ok(view! {
        <div
            data-slot="command"
            class="flex min-w-0 items-center gap-2 rounded-lg border border-border py-1 pr-1 pl-4"
        >
            <code
                class="min-w-0 flex-1 overflow-x-auto py-2 font-mono text-[13px] whitespace-pre"
            >
                (text)
            </code>
            copy_button(text: text, label: "Copy command")
        </div>
    })
}

/// Classes for a tab of a two-tab switcher.
const TAB: StaticClass = class!(
    "relative -mb-px inline-flex h-10 cursor-pointer items-center px-3 text-sm font-medium",
    "text-muted-foreground outline-none transition-colors hover:text-foreground",
    "focus-visible:ring-2 focus-visible:ring-ring",
    "aria-selected:text-foreground aria-selected:after:absolute aria-selected:after:inset-x-1",
    "aria-selected:after:bottom-0 aria-selected:after:h-0.5 aria-selected:after:rounded-full",
    "aria-selected:after:bg-foreground",
);

/// Two tabs, such as Preview and Code, over two panels. The first is selected until
/// the reader picks the second; without the runtime, only the first shows.
#[component]
pub async fn tab_pair(
    cx: &Cx,
    first: &str,
    second: &str,
    #[into] second_panel: Child<'_>,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    let showing_second = signal(cx, || false);
    let id = topcoat::view::element_id(cx);
    let (first_tab, second_tab) = (format!("{id}-tab-1"), format!("{id}-tab-2"));
    let (first_panel, second_panel_id) = (format!("{id}-panel-1"), format!("{id}-panel-2"));

    Ok(view! {
        <div class="flex min-w-0 flex-col gap-2">
            <div role="tablist" class="flex items-center gap-1 border-b border-border">
                <button
                    type="button"
                    role="tab"
                    id=(&first_tab)
                    aria-controls=(&first_panel)
                    class=(TAB)
                    :aria-selected=$(if showing_second.get() { "false" } else { "true" })
                    :tabindex=$(if showing_second.get() { "-1" } else { "0" })
                    @click=$(|_e: Event| showing_second.set(false))
                    @keydown=$(|_e: Event| {
                        // An arrow key moves to the other tab and shows its panel.
                        if raw!(
                            "cx.hydrate((k => (k === 'ArrowLeft' || k === 'ArrowRight') && (${_e}.inner.currentTarget.parentElement.querySelector('[aria-selected=\\\"false\\\"]').focus(), true))(${_e}.inner.key))",
                            false,
                        ) {
                            showing_second.toggle();
                        }
                    })
                >
                    (first)
                </button>
                <button
                    type="button"
                    role="tab"
                    id=(&second_tab)
                    aria-controls=(&second_panel_id)
                    class=(TAB)
                    :aria-selected=$(if showing_second.get() { "true" } else { "false" })
                    :tabindex=$(if showing_second.get() { "0" } else { "-1" })
                    @click=$(|_e: Event| showing_second.set(true))
                    @keydown=$(|_e: Event| {
                        // An arrow key moves to the other tab and shows its panel.
                        if raw!(
                            "cx.hydrate((k => (k === 'ArrowLeft' || k === 'ArrowRight') && (${_e}.inner.currentTarget.parentElement.querySelector('[aria-selected=\\\"false\\\"]').focus(), true))(${_e}.inner.key))",
                            false,
                        ) {
                            showing_second.toggle();
                        }
                    })
                >
                    (second)
                </button>
            </div>
            <div
                role="tabpanel"
                data-tab="first"
                id=(&first_panel)
                aria-labelledby=(&first_tab)
                class="min-w-0"
                :hidden=$(showing_second.get())
            >
                (child)
            </div>
            <div
                role="tabpanel"
                data-tab="second"
                id=(&second_panel_id)
                aria-labelledby=(&second_tab)
                class="min-w-0"
                :hidden=$(!showing_second.get())
            >
                (second_panel)
            </div>
        </div>
    })
}

/// An example: its heading, a sentence, and Preview and Code tabs.
#[component]
pub async fn example_section(
    cx: &Cx,
    example: &'static Example,
    #[default] wide: bool,
) -> Result<impl View> {
    Ok(view! {
        <section
            id=(example.id)
            data-example=(example.id)
            class="flex min-w-0 scroll-mt-20 flex-col gap-2.5 not-first:mt-10"
        >
            // A block sits right under the page title. An example sits under a section.
            if wide {
                <h2 class="text-base font-medium">(example.title)</h2>
            } else {
                <h3 class="text-base font-medium">(example.title)</h3>
            }
            if !example.description.is_empty() {
                <p class="text-sm leading-relaxed text-muted-foreground">
                    (inline_code(example.description))
                </p>
            }
            tab_pair(
                first: "Preview",
                second: "Code",
                second_panel: topcoat::view::Child::new(
                    view! { code_panel(source: example.source) },
                ),
                preview_panel(wide: wide, ((example.render)(cx)))
            )
        </section>
    })
}

/// Turns `` `code` `` spans in a sentence into `<code>` elements.
pub fn inline_code(text: &str) -> Unescaped<String> {
    let mut out = String::new();
    for (index, part) in text.split('`').enumerate() {
        let escaped = part
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        if index % 2 == 1 {
            out.push_str(
                "<code class=\"rounded bg-foreground/5 px-1 py-0.5 font-mono text-[0.85em]\">",
            );
            out.push_str(&escaped);
            out.push_str("</code>");
        } else {
            out.push_str(&escaped);
        }
    }
    Unescaped::new_unchecked(out)
}

/// Classes for inline code in a sentence the site writes itself.
const INLINE_CODE: StaticClass =
    class!("rounded bg-foreground/5 px-1 py-0.5 font-mono text-[0.85em]");

/// The Installation section of a component page: CLI and Manual tabs.
#[component]
pub async fn install_section(cx: &Cx, doc: &'static ComponentDoc) -> Result<impl View> {
    let add = format!("topcoat ui add {} --registry topcoat-graph", doc.name);
    let parts = catalog::dependencies(doc.name);
    let parts_list = parts.join(", ");

    Ok(view! {
        section_heading(id: "installation", "Installation")
        tab_pair(
            first: "CLI",
            second: "Manual",
            second_panel: topcoat::view::Child::new(
                view! {
                    <ol
                        class="list-decimal space-y-6 pl-5 text-sm leading-relaxed marker:text-muted-foreground"
                    >
                        <li class="space-y-3 pl-1">
                            <p>
                                "Add the "
                                <code class=(INLINE_CODE)>"topcoat-graph"</code>
                                " crate, which has the geometry the component calls:"
                            </p>
                            command(text: "cargo add topcoat-graph")
                        </li>
                        if !parts.is_empty() {
                            <li class="space-y-3 pl-1" data-slot="parts">
                                <p>
                                    "Copy in the parts it uses too, each from its own page:"
                                </p>
                                <ul class="flex flex-wrap gap-2">
                                    for name in &parts {
                                        let href = format!("/docs/{name}");
                                        <li>
                                            link(
                                                href: &href,
                                                attrs: attributes! {
                                                    class="rounded-md bg-foreground/5 px-2 py-1 font-mono text-xs underline-offset-4 hover:underline"
                                                },
                                                (name)
                                            )
                                        </li>
                                    }
                                </ul>
                            </li>
                        }
                        <li class="space-y-3 pl-1">
                            <p>
                                "Copy this source into "
                                <code class=(INLINE_CODE)>
                                    (format!("src/components/{}.rs", doc.name))
                                </code>
                                " and declare the module in "
                                <code class=(INLINE_CODE)>"src/components.rs"</code>
                                ":"
                            </p>
                            code_panel(source: doc.source)
                        </li>
                    </ol>
                },
            ),
            <div class="flex flex-col gap-3">
                command(text: &add)
                <p class="text-sm text-muted-foreground" data-slot="needs-crate">
                    "The topcoat-graph crate has to be one of your dependencies first. See "
                    link(
                        href: "/docs/installation",
                        attrs: attributes! { class="underline underline-offset-4" },
                        "Installation"
                    )
                    "."
                </p>
                if !parts_list.is_empty() {
                    <p class="text-sm text-muted-foreground">
                        "This also installs the parts it uses: "
                        (parts_list)
                        "."
                    </p>
                }
            </div>
        )
    })
}

/// Classes for a table cell holding code.
const CODE_CELL: StaticClass =
    class!("px-4 py-2.5 align-top font-mono text-[13px] whitespace-nowrap");

/// The API section: a table per public component function, then the public types,
/// helpers and constants, all read from the registry source.
#[component]
pub async fn api_section(cx: &Cx, doc: &'static ComponentDoc) -> Result<impl View> {
    let functions = api::functions(doc.source);
    let items = api::items(doc.source);

    Ok(view! {
        section_heading(id: "api", "Component API")
        <div class="flex flex-col gap-8">
            for function in functions {
                <div class="flex min-w-0 flex-col gap-2.5" data-slot="props">
                    <h3 class="font-mono text-sm font-medium">(function.name)</h3>
                    <div class="overflow-x-auto rounded-lg border border-border">
                        <table class="w-full text-left text-sm">
                            <thead class="bg-foreground/[0.03]">
                                <tr class="border-b border-border">
                                    <th scope="col" class="h-10 px-4 font-medium">"Prop"</th>
                                    <th scope="col" class="h-10 px-4 font-medium">"Type"</th>
                                    <th scope="col" class="h-10 px-4 font-medium">
                                        "Default"
                                    </th>
                                    <th scope="col" class="h-10 px-4 font-medium">
                                        "Description"
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-border">
                                for prop in function.props {
                                    <tr>
                                        <td class=(CODE_CELL)>(prop.name)</td>
                                        <td class=(class!(CODE_CELL, "text-muted-foreground"))>
                                            (prop.ty)
                                        </td>
                                        <td class=(class!(CODE_CELL, "text-muted-foreground"))>
                                            match prop.default {
                                                None => <span class="font-sans text-xs">"required"</span>,
                                                Some(default) => (default),
                                            }
                                        </td>
                                        <td
                                            class="min-w-56 px-4 py-2.5 align-top text-muted-foreground"
                                        >
                                            (inline_code(&prop.description))
                                        </td>
                                    </tr>
                                }
                            </tbody>
                        </table>
                    </div>
                </div>
            }
        </div>
        if !items.is_empty() {
            section_heading(id: "types", "Types and helpers")
            <div class="flex flex-col gap-8">
                for item in items {
                    item_entry(item: item)
                }
            </div>
        }
    })
}

/// A public struct, helper or constant: its declaration, what it is, and a struct's
/// fields and methods.
#[component]
pub async fn item_entry(cx: &Cx, item: api::Item) -> Result<impl View> {
    Ok(view! {
        <div
            class="flex min-w-0 flex-col gap-2.5"
            data-slot="item"
            id=(format!("item-{}", item.name))
        >
            <h3 class="font-mono text-sm font-medium">(&item.name)</h3>
            if !item.description.is_empty() {
                <p class="text-sm leading-relaxed text-muted-foreground">
                    (inline_code(&item.description))
                </p>
            }
            if item.fields.is_empty() {
                signature(source: &item.signature)
            } else {
                <div class="overflow-x-auto rounded-lg border border-border">
                    <table class="w-full text-left text-sm">
                        <thead class="bg-foreground/[0.03]">
                            <tr class="border-b border-border">
                                if item.signature.starts_with("pub enum ") {
                                    <th scope="col" class="h-10 px-4 font-medium">
                                        "Variant"
                                    </th>
                                    <th scope="col" class="h-10 px-4 font-medium">
                                        <span class="sr-only">"Default"</span>
                                    </th>
                                } else {
                                    <th scope="col" class="h-10 px-4 font-medium">"Field"</th>
                                    <th scope="col" class="h-10 px-4 font-medium">"Type"</th>
                                }
                                <th scope="col" class="h-10 px-4 font-medium">
                                    "Description"
                                </th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-border">
                            for field in &item.fields {
                                <tr>
                                    <td class=(CODE_CELL)>(&field.name)</td>
                                    <td class=(class!(CODE_CELL, "text-muted-foreground"))>
                                        (&field.ty)
                                    </td>
                                    <td
                                        class="min-w-56 px-4 py-2.5 align-top text-muted-foreground"
                                    >
                                        (inline_code(&field.description))
                                    </td>
                                </tr>
                            }
                        </tbody>
                    </table>
                </div>
            }
            for method in &item.methods {
                <div class="flex min-w-0 flex-col gap-1.5 pl-4">
                    signature(source: &method.signature)
                    if !method.description.is_empty() {
                        <p class="text-sm leading-relaxed text-muted-foreground">
                            (inline_code(&method.description))
                        </p>
                    }
                </div>
            }
        </div>
    })
}

/// A declaration, highlighted, in a small box that scrolls sideways.
#[component]
async fn signature(cx: &Cx, source: &str) -> Result<impl View> {
    let html = crate::code::highlight("rust", source).unwrap_or_default();

    Ok(view! {
        <pre
            class=(class!(
                "overflow-x-auto rounded-lg border border-border bg-card px-4 py-2.5 font-mono",
                "text-[13px] leading-relaxed",
                crate::code::TOKEN_COLORS,
            ))
        >
            <code>(Unescaped::new_unchecked(html))</code>
        </pre>
    })
}

/// Previous and next links, in sidebar order.
#[component]
pub async fn pager(cx: &Cx, path: &str) -> Result<impl View> {
    let pages = catalog::pages();
    let index = pages.iter().position(|page| page.href == path);
    let mut pages = pages.into_iter().map(Some).collect::<Vec<_>>();
    let mut take = |index: Option<usize>| index.and_then(|index| pages.get_mut(index)?.take());
    let previous = take(index.and_then(|index| index.checked_sub(1)));
    let next = take(index.map(|index| index + 1));

    Ok(view! {
        <nav
            aria-label="Pages"
            class="mt-16 flex items-center justify-between gap-4 text-sm"
        >
            match previous {
                Some(page) => {
                    link(
                        href: &page.href,
                        attrs: attributes! { class=(PAGER_LINK) rel="prev" },
                        icon(data: iconify_icon!("lucide:arrow-left"))
                        (page.title)
                    )
                }
                None => <span></span>,
            }
            match next {
                Some(page) => {
                    link(
                        href: &page.href,
                        attrs: attributes! { class=(PAGER_LINK) rel="next" },
                        (page.title)
                        icon(data: iconify_icon!("lucide:arrow-right"))
                    )
                }
                None => <span></span>,
            }
        </nav>
    })
}

const PAGER_LINK: StaticClass = class!(
    "inline-flex items-center gap-2 rounded-lg px-3 py-2 font-medium text-foreground/80",
    "outline-none transition-colors hover:bg-foreground/5 hover:text-foreground",
    "focus-visible:ring-2 focus-visible:ring-ring",
);

/// Prose for the guide pages.
#[component]
pub async fn prose(cx: &Cx, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <div
            class=(class!(
                "mt-8 min-w-0 text-[15px] leading-7 text-foreground/90",
                "[&_p]:my-4 [&_h2]:mt-12 [&_h2]:mb-4 [&_h2]:text-xl [&_h2]:font-medium",
                "[&_h2]:tracking-tight [&_h2]:text-foreground [&_h3]:mt-8 [&_h3]:mb-3",
                "[&_h3]:font-medium [&_h3]:text-foreground [&_ul]:my-4 [&_ul]:list-disc",
                "[&_ul]:pl-5 [&_li]:my-1.5 [&_a]:font-medium [&_a]:text-foreground",
                "[&_a]:underline [&_a]:underline-offset-4",
                "[&_:not(pre)>code]:rounded [&_:not(pre)>code]:bg-foreground/5",
                "[&_:not(pre)>code]:px-1 [&_:not(pre)>code]:py-0.5",
                "[&_:not(pre)>code]:font-mono [&_:not(pre)>code]:text-[0.85em]",
                "[&_.code-panel]:my-5 [&_[data-slot=command]]:my-4 [&_section]:my-8",
            ))
        >
            (child)
        </div>
    })
}
