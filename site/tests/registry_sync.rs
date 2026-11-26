//! Checks that the site installs every topcoat-graph component and that each installed
//! file is byte-for-byte the registry source. After changing a component in
//! `crates/topcoat-graph`, run `topcoat ui add <name> --registry topcoat-graph --overwrite`
//! in `site`.

use std::path::Path;

use topcoat_ui::{
    Registry,
    manage::{InstallStatus, Package, list},
};

/// The registry's name: the name of the crate that ships it.
const REGISTRY: &str = "topcoat-graph";

/// The site package's root directory.
fn package_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The topcoat-graph registry, loaded from its crate in this workspace.
fn registry() -> Registry {
    Registry::load(package_root().join("../crates/topcoat-graph/registry"))
        .expect("the workspace's topcoat-graph registry loads")
}

/// What to run when the component `name` has drifted from the registry.
fn hint(name: &str) -> String {
    format!("run `topcoat ui add {name} --registry {REGISTRY} --overwrite` in site")
}

#[test]
fn components_match_registry() {
    let registry = registry();
    let mut count = 0;

    for name in registry.names() {
        let component = registry.get(name).expect("name came from the registry");
        let installed = package_root()
            .join("src/components")
            .join(component.file_name());
        let hint = hint(name);
        let source = std::fs::read_to_string(&installed)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}; {hint}", installed.display()));
        assert!(
            source == component.read_source().unwrap(),
            "{} no longer matches the registry's `{name}` component; {hint}",
            installed.display(),
        );
        count += 1;
    }

    assert_eq!(count, 7, "the registry should offer all 7 components");
}

#[test]
fn components_are_up_to_date() {
    let package = Package::locate(Some(String::from("site"))).expect("the site is a package");
    let listings =
        list(&package, Some(REGISTRY)).expect("the topcoat-graph registry can be listed");
    let statuses = listings
        .into_iter()
        .next()
        .expect("listing the topcoat-graph registry yields it")
        .outcome
        .expect("the topcoat-graph registry loads");

    for status in statuses {
        let name = &status.name;
        match status.status {
            InstallStatus::UpToDate { .. } => {}
            InstallStatus::Update { .. } => {
                panic!(
                    "`{name}` has an update available; {hint}",
                    hint = hint(name)
                )
            }
            InstallStatus::Available { .. } => panic!(
                "the registry offers `{name}`, which the site has not installed; \
                 run `topcoat ui add {name} --registry {REGISTRY}` in site \
                 and document it in src/catalog.rs",
            ),
            InstallStatus::Orphaned { .. } => panic!(
                "the site has `{name}` installed, which the registry no longer offers; \
                 run `topcoat ui remove {name}` in site",
            ),
        }
    }
}
