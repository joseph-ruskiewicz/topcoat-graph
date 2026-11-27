//! Reads the API tables of the docs from the component sources themselves.
//!
//! Every public `#[component]` function becomes a table with a row per prop: its name,
//! type, default and the first sentence of its doc comment. Public structs, helper
//! functions and constants are listed too, with their fields and signatures. Reading the
//! sources keeps the tables from drifting: changing a prop changes its row. The sources
//! are rustfmt-formatted, so a line-based reader is enough.

/// The geometry crate's sources, where the enums the props use are defined, for their
/// `#[default]` variants.
const GEOMETRY: &[&str] = &[
    include_str!("../../crates/topcoat-graph/src/layout.rs"),
    include_str!("../../crates/topcoat-graph/src/path.rs"),
];

/// One public component function.
#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub props: Vec<Prop>,
}

/// One prop of a component function.
#[derive(Debug, PartialEq, Eq)]
pub struct Prop {
    /// The name as written in `view!`; the child content is `children`.
    pub name: String,
    pub ty: String,
    /// The default as Rust source, or `None` when the prop is required.
    pub default: Option<String>,
    pub description: String,
}

/// The component functions of `source`, in source order.
pub fn functions(source: &str) -> Vec<Function> {
    let lines: Vec<&str> = source.lines().collect();
    let mut functions = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].trim() != "#[component]" {
            i += 1;
            continue;
        }
        let Some(name) = lines
            .get(i + 1)
            .and_then(|line| line.trim().strip_prefix("pub async fn "))
            .and_then(|rest| rest.strip_suffix('('))
        else {
            i += 1;
            continue;
        };

        let mut props = Vec::new();
        let mut docs = Vec::new();
        let mut default = None;
        i += 2;
        while i < lines.len() && !lines[i].trim_start().starts_with(')') {
            let line = lines[i].trim();
            if let Some(doc) = line.strip_prefix("///") {
                docs.push(doc.trim());
            } else if line == "#[default]" {
                default = Some(String::new());
            } else if let Some(value) = line
                .strip_prefix("#[default(")
                .and_then(|rest| rest.strip_suffix(")]"))
            {
                default = Some(value.to_owned());
            } else if line.starts_with("#[") {
                // `#[into]` and other attributes don't change the table.
            } else if let Some((param, ty)) = line.trim_end_matches(',').split_once(": ") {
                let param = param.trim_start_matches("mut ");
                if param != "cx" {
                    let default = default.take().map(|value| {
                        if value.is_empty() {
                            implied_default(ty, source)
                        } else {
                            simplify(&value)
                        }
                    });
                    props.push(Prop {
                        name: if param == "child" { "children" } else { param }.to_owned(),
                        ty: ty.to_owned(),
                        default,
                        description: first_sentence(&docs.join(" ")),
                    });
                }
                docs.clear();
                default = None;
            }
            i += 1;
        }
        functions.push(Function {
            name: name.to_owned(),
            props,
        });
    }

    functions
}

/// What `#[default]` gives a prop of type `ty`, as Rust source.
fn implied_default(ty: &str, source: &str) -> String {
    match ty {
        "bool" => "false".to_owned(),
        "u32" | "usize" | "u64" | "i32" => "0".to_owned(),
        "f64" => "0.0".to_owned(),
        "&str" | "String" => "\"\"".to_owned(),
        _ if ty.starts_with("Option<") => "None".to_owned(),
        _ if ty.starts_with("Vec<") => "[]".to_owned(),
        _ if ty == "Attributes" || ty.starts_with("Child") => String::new(),
        _ => std::iter::once(source)
            .chain(GEOMETRY.iter().copied())
            .find_map(|source| enum_default(ty, source))
            .unwrap_or_else(|| format!("{ty}::default()")),
    }
}

/// The variant of the enum `ty` in `source` marked `#[default]`.
fn enum_default(ty: &str, source: &str) -> Option<String> {
    let start = source.find(&format!("pub enum {ty} {{"))?;
    let body = &source[start..];
    let body = &body[..body.find("\n}")?];
    let mut lines = body.lines().map(str::trim);
    lines.find(|line| *line == "#[default]")?;
    let variant = lines.find(|line| !line.starts_with("///") && !line.starts_with("#["))?;
    Some(format!("{ty}::{}", variant.trim_end_matches(',')))
}

/// Shortens default expressions to the value a reader cares about.
fn simplify(value: &str) -> String {
    if let Some(inner) = value
        .strip_prefix("String::from(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return inner.to_owned();
    }
    value.to_owned()
}

/// The first sentence of a doc comment, without its Markdown link targets.
fn first_sentence(docs: &str) -> String {
    let end = docs
        .match_indices(". ")
        .map(|(index, _)| index + 1)
        .next()
        .unwrap_or(docs.len());
    unlink(&docs[..end])
}

/// The first paragraph of a doc comment, without its Markdown link targets.
fn first_paragraph(docs: &[&str]) -> String {
    let paragraph: Vec<&str> = docs
        .iter()
        .copied()
        .take_while(|line| !line.is_empty())
        .collect();
    unlink(&paragraph.join(" "))
}

/// `` [`name`] `` links as plain `` `name` ``.
fn unlink(text: &str) -> String {
    text.replace("[`", "`").replace("`]", "`")
}

/// A public struct, function or constant that isn't a component.
#[derive(Debug, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    /// The declaration as Rust source: a function's signature, a struct's name line or
    /// a constant's type.
    pub signature: String,
    pub description: String,
    /// A struct's public fields.
    pub fields: Vec<Field>,
    /// A struct's public methods.
    pub methods: Vec<Item>,
}

/// One public field of a struct.
#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub description: String,
}

/// The public structs, functions and constants of `source` that aren't components, in
/// source order, with each struct's methods under it.
pub fn items(source: &str) -> Vec<Item> {
    let lines: Vec<&str> = source.lines().collect();
    let mut items: Vec<Item> = Vec::new();
    let mut docs: Vec<&str> = Vec::new();
    // The struct whose `impl` block the reader is in.
    let mut in_impl: Option<String> = None;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        if let Some(doc) = trimmed.strip_prefix("///") {
            docs.push(doc.strip_prefix(' ').unwrap_or(doc));
            i += 1;
            continue;
        }
        if trimmed.starts_with("#[") {
            if trimmed == "#[component]" {
                docs.clear();
            }
            i += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("impl ") {
            in_impl = rest.strip_suffix(" {").map(str::to_owned);
        } else if line == "}" {
            in_impl = None;
        } else if let Some(rest) = line.strip_prefix("pub struct ") {
            let name = rest.trim_end_matches(" {").trim_end_matches(';').to_owned();
            let mut fields = Vec::new();
            let mut field_docs = Vec::new();
            i += 1;
            while i < lines.len() && lines[i] != "}" {
                let field = lines[i].trim();
                if let Some(doc) = field.strip_prefix("///") {
                    field_docs.push(doc.strip_prefix(' ').unwrap_or(doc));
                } else if let Some((name, ty)) = field
                    .strip_prefix("pub ")
                    .and_then(|field| field.trim_end_matches(',').split_once(": "))
                {
                    fields.push(Field {
                        name: name.to_owned(),
                        ty: ty.to_owned(),
                        description: first_paragraph(&field_docs),
                    });
                    field_docs.clear();
                } else {
                    field_docs.clear();
                }
                i += 1;
            }
            items.push(Item {
                signature: format!("pub struct {name}"),
                name,
                description: first_paragraph(&docs),
                fields,
                methods: Vec::new(),
            });
        } else if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            let name = rest[..rest.find(['(', '<']).unwrap_or(rest.len())].to_owned();
            let indent = line.len() - trimmed.len();
            let mut signature = Vec::new();
            loop {
                let part = &lines[i][indent.min(lines[i].len())..];
                if let Some(head) = part.strip_suffix(" {") {
                    signature.push(head);
                    break;
                }
                signature.push(part);
                i += 1;
                if i >= lines.len() {
                    break;
                }
            }
            let item = Item {
                name,
                signature: signature.join("\n"),
                description: first_paragraph(&docs),
                fields: Vec::new(),
                methods: Vec::new(),
            };
            match &in_impl {
                Some(owner) if indent > 0 => {
                    if let Some(owner) = items.iter_mut().find(|item| item.name == *owner) {
                        owner.methods.push(item);
                    }
                }
                _ if indent == 0 => items.push(item),
                _ => {}
            }
        } else if let Some((name, ty)) = line
            .strip_prefix("pub const ")
            .and_then(|rest| rest.split_once(": "))
        {
            let ty = ty.split_once(" =").map_or(ty, |(ty, _)| ty);
            items.push(Item {
                name: name.to_owned(),
                signature: format!("pub const {name}: {ty}"),
                description: first_paragraph(&docs),
                fields: Vec::new(),
                methods: Vec::new(),
            });
        }
        docs.clear();
        i += 1;
    }

    items
}

#[cfg(test)]
mod tests {
    use super::{Field, Function, Item, Prop, functions, items};

    const SOURCE: &str = r#"
/// The animation.
#[derive(Default)]
pub enum Variant {
    /// Dots.
    #[default]
    Dots,
    Pulse,
}

/// A point.
///
/// More about points.
#[derive(Clone)]
pub struct Point {
    /// Across, from 0 to 100.
    pub x: f64,
    /// A private detail.
    hidden: bool,
}

impl Point {
    /// A point at `x`.
    ///
    /// ```ignore
    /// Point::new(1.0)
    /// ```
    pub fn new(x: f64) -> Self {
        Self { x, hidden: false }
    }

    fn private(&self) {}
}

/// Labels from strings: [`labels`].
pub fn labels<T: Into<String>>(items: impl IntoIterator<Item = T>) -> Vec<String> {
    todo!()
}

/// Many arguments.
pub fn long(
    a: &[String],
    b: Orientation,
) -> Vec<u8> {
    todo!()
}

/// The colors.
pub const COLORS: StaticClass = class!(
    "a",
);

/// A thing.
#[component]
pub async fn thing(
    cx: &Cx,
    /// The animation to show. More text.
    #[default]
    variant: Variant,
    /// Which way, for [`thing`].
    #[default]
    orientation: Orientation,
    /// The label.
    #[into]
    #[default(String::from("Thinking…"))]
    label: String,
    /// One in this many.
    #[default(1)]
    every: usize,
    /// The URL.
    href: &str,
    /// Extra attributes.
    #[default]
    mut attrs: Attributes,
    /// The content.
    #[default]
    child: Child<'_>,
) -> Result<impl View> {
    todo!()
}

async fn private_helper(cx: &Cx) {}
"#;

    #[test]
    fn reads_props_types_defaults_and_docs() {
        let prop = |name: &str, ty: &str, default: Option<&str>, description: &str| Prop {
            name: name.to_owned(),
            ty: ty.to_owned(),
            default: default.map(str::to_owned),
            description: description.to_owned(),
        };
        assert_eq!(
            functions(SOURCE),
            [Function {
                name: "thing".to_owned(),
                props: vec![
                    prop(
                        "variant",
                        "Variant",
                        Some("Variant::Dots"),
                        "The animation to show."
                    ),
                    // Defined in the geometry crate, not in the source.
                    prop(
                        "orientation",
                        "Orientation",
                        Some("Orientation::Vertical"),
                        "Which way, for `thing`."
                    ),
                    prop("label", "String", Some("\"Thinking…\""), "The label."),
                    prop("every", "usize", Some("1"), "One in this many."),
                    prop("href", "&str", None, "The URL."),
                    prop("attrs", "Attributes", Some(""), "Extra attributes."),
                    prop("children", "Child<'_>", Some(""), "The content."),
                ],
            }]
        );
    }

    #[test]
    fn reads_structs_helpers_and_constants() {
        let items = items(SOURCE);
        let names: Vec<&str> = items.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, ["Point", "labels", "long", "COLORS"]);
        assert_eq!(
            items[0],
            Item {
                name: "Point".to_owned(),
                signature: "pub struct Point".to_owned(),
                description: "A point.".to_owned(),
                fields: vec![Field {
                    name: "x".to_owned(),
                    ty: "f64".to_owned(),
                    description: "Across, from 0 to 100.".to_owned(),
                }],
                methods: vec![Item {
                    name: "new".to_owned(),
                    signature: "pub fn new(x: f64) -> Self".to_owned(),
                    description: "A point at `x`.".to_owned(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                }],
            }
        );
        assert_eq!(items[1].description, "Labels from strings: `labels`.");
        assert_eq!(
            items[2].signature,
            "pub fn long(\n    a: &[String],\n    b: Orientation,\n) -> Vec<u8>"
        );
        assert_eq!(items[3].signature, "pub const COLORS: StaticClass");
    }

    #[test]
    fn every_registry_component_documents_every_prop() {
        for doc in crate::catalog::COMPONENTS {
            let functions = functions(doc.source);
            assert!(
                !functions.is_empty(),
                "{} has no component functions",
                doc.name
            );
            for function in functions {
                for prop in function.props {
                    assert!(
                        !prop.description.is_empty(),
                        "{}::{} prop `{}` has no doc comment",
                        doc.name,
                        function.name,
                        prop.name,
                    );
                    assert!(
                        !prop
                            .default
                            .as_deref()
                            .unwrap_or("")
                            .ends_with("::default()"),
                        "{}::{} prop `{}` has no readable default",
                        doc.name,
                        function.name,
                        prop.name,
                    );
                }
            }
        }
    }

    #[test]
    fn the_registry_helpers_are_listed() {
        let chart = crate::catalog::component("chart").unwrap();
        let names: Vec<String> = items(chart.source)
            .into_iter()
            .map(|item| item.name)
            .collect();
        for name in [
            "ChartSeries",
            "labels",
            "series_color",
            "CHART_VARS",
            "percent",
        ] {
            assert!(names.iter().any(|item| item == name), "{name} in {names:?}");
        }
        let series = items(chart.source)
            .into_iter()
            .find(|item| item.name == "ChartSeries")
            .unwrap();
        let fields: Vec<&str> = series
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        assert_eq!(fields, ["name", "values", "color"]);
        let methods: Vec<&str> = series
            .methods
            .iter()
            .map(|method| method.name.as_str())
            .collect();
        assert_eq!(methods, ["new", "color"]);

        let pie = crate::catalog::component("pie_chart").unwrap();
        let names: Vec<String> = items(pie.source)
            .into_iter()
            .map(|item| item.name)
            .collect();
        assert_eq!(names, ["PieSlice"], "private structs stay out");

        let tooltip = crate::catalog::component("chart_tooltip").unwrap();
        let names: Vec<String> = items(tooltip.source)
            .into_iter()
            .map(|item| item.name)
            .collect();
        assert_eq!(
            names,
            ["TooltipRow", "CategoryTooltip", "category_tooltips"]
        );
    }
}
