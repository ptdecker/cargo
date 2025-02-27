//! Tests for inheriting Cargo.toml fields with field.workspace = true
use cargo_test_support::registry::{Dependency, Package};
use cargo_test_support::{
    basic_lib_manifest, basic_manifest, git, path2url, paths, project, publish, registry,
};

#[cargo_test]
fn permit_additional_workspace_fields() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            [workspace.package]
            version = "1.2.3"
            authors = ["Rustaceans"]
            description = "This is a crate"
            documentation = "https://www.rust-lang.org/learn"
            readme = "README.md"
            homepage = "https://www.rust-lang.org"
            repository = "https://github.com/example/example"
            license = "MIT"
            license-file = "LICENSE"
            keywords = ["cli"]
            categories = ["development-tools"]
            publish = false
            edition = "2018"
            rust-version = "1.60"
            exclude = ["foo.txt"]
            include = ["bar.txt", "**/*.rs", "Cargo.toml", "LICENSE", "README.md"]

            [workspace.package.badges]
            gitlab = { repository = "https://gitlab.com/rust-lang/rust", branch = "master" }

            [workspace.dependencies]
            dep = "0.1"
        "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              workspace = ".."
              "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        // Should not warn about unused fields.
        .with_stderr(
            "\
[COMPILING] bar v0.1.0 ([CWD]/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("check").run();
    let lockfile = p.read_lockfile();
    assert!(!lockfile.contains("dep"));
}

#[cargo_test]
fn deny_optional_dependencies() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]

            [workspace.dependencies]
            dep1 = { version = "0.1", optional = true }
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "bar/Cargo.toml",
            r#"
              [package]
              name = "bar"
              version = "0.1.0"
              authors = []
              workspace = ".."
              "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]foo/Cargo.toml`

Caused by:
  dep1 is optional, but workspace dependencies cannot be optional
",
        )
        .run();
}

#[cargo_test]
fn inherit_own_workspace_fields() {
    registry::init();

    let p = project().build();

    let _ = git::repo(&paths::root().join("foo"))
        .file(
            "Cargo.toml",
            r#"
            badges.workspace = true

            [package]
            name = "foo"
            version.workspace = true
            authors.workspace = true
            description.workspace = true
            documentation.workspace = true
            homepage.workspace = true
            repository.workspace = true
            license.workspace = true
            keywords.workspace = true
            categories.workspace = true
            publish.workspace = true
            edition.workspace = true
            rust-version.workspace = true
            exclude.workspace = true
            include.workspace = true

            [workspace]
            members = []
            [workspace.package]
            version = "1.2.3"
            authors = ["Rustaceans"]
            description = "This is a crate"
            documentation = "https://www.rust-lang.org/learn"
            homepage = "https://www.rust-lang.org"
            repository = "https://github.com/example/example"
            license = "MIT"
            keywords = ["cli"]
            categories = ["development-tools"]
            publish = true
            edition = "2018"
            rust-version = "1.60"
            exclude = ["foo.txt"]
            include = ["bar.txt", "**/*.rs", "Cargo.toml"]
            [workspace.package.badges]
            gitlab = { repository = "https://gitlab.com/rust-lang/rust", branch = "master" }
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("foo.txt", "") // should be ignored when packaging
        .file("bar.txt", "") // should be included when packaging
        .build();

    p.cargo("publish --token sekrit").run();
    publish::validate_upload_with_contents(
        r#"
        {
          "authors": ["Rustaceans"],
          "badges": {
            "gitlab": { "branch": "master", "repository": "https://gitlab.com/rust-lang/rust" }
          },
          "categories": ["development-tools"],
          "deps": [],
          "description": "This is a crate",
          "documentation": "https://www.rust-lang.org/learn",
          "features": {},
          "homepage": "https://www.rust-lang.org",
          "keywords": ["cli"],
          "license": "MIT",
          "license_file": null,
          "links": null,
          "name": "foo",
          "readme": null,
          "readme_file": null,
          "repository": "https://github.com/example/example",
          "vers": "1.2.3"
          }
        "#,
        "foo-1.2.3.crate",
        &[
            "Cargo.lock",
            "Cargo.toml",
            "Cargo.toml.orig",
            "src/main.rs",
            ".cargo_vcs_info.json",
            "bar.txt",
        ],
        &[(
            "Cargo.toml",
            &format!(
                r#"{}
[package]
edition = "2018"
rust-version = "1.60"
name = "foo"
version = "1.2.3"
authors = ["Rustaceans"]
exclude = ["foo.txt"]
include = [
    "bar.txt",
    "**/*.rs",
    "Cargo.toml",
]
publish = true
description = "This is a crate"
homepage = "https://www.rust-lang.org"
documentation = "https://www.rust-lang.org/learn"
keywords = ["cli"]
categories = ["development-tools"]
license = "MIT"
repository = "https://github.com/example/example"
resolver = "1"

[badges.gitlab]
branch = "master"
repository = "https://gitlab.com/rust-lang/rust"
"#,
                cargo::core::package::MANIFEST_PREAMBLE
            ),
        )],
    );
}

#[cargo_test]
fn inherit_own_dependencies() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "bar"
            version = "0.2.0"
            authors = []

            [dependencies]
            dep.workspace = true

            [build-dependencies]
            dep-build.workspace = true

            [dev-dependencies]
            dep-dev.workspace = true

            [workspace]
            members = []

            [workspace.dependencies]
            dep = "0.1"
            dep-build = "0.8"
            dep-dev = "0.5.2"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("dep", "0.1.2").publish();
    Package::new("dep-build", "0.8.2").publish();
    Package::new("dep-dev", "0.5.2").publish();

    p.cargo("build")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] dep-build v0.8.2 ([..])
[DOWNLOADED] dep v0.1.2 ([..])
[COMPILING] dep v0.1.2
[COMPILING] bar v0.2.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("check").run();
    let lockfile = p.read_lockfile();
    assert!(lockfile.contains("dep"));
    assert!(lockfile.contains("dep-dev"));
    assert!(lockfile.contains("dep-build"));
    p.cargo("publish --token sekrit").run();
    publish::validate_upload_with_contents(
        r#"
        {
          "authors": [],
          "badges": {},
          "categories": [],
          "deps": [
            {
              "default_features": true,
              "features": [],
              "kind": "normal",
              "name": "dep",
              "optional": false,
              "registry": "https://github.com/rust-lang/crates.io-index",
              "target": null,
              "version_req": "^0.1"
            },
            {
              "default_features": true,
              "features": [],
              "kind": "dev",
              "name": "dep-dev",
              "optional": false,
              "registry": "https://github.com/rust-lang/crates.io-index",
              "target": null,
              "version_req": "^0.5.2"
            },
            {
              "default_features": true,
              "features": [],
              "kind": "build",
              "name": "dep-build",
              "optional": false,
              "registry": "https://github.com/rust-lang/crates.io-index",
              "target": null,
              "version_req": "^0.8"
            }
          ],
          "description": null,
          "documentation": null,
          "features": {},
          "homepage": null,
          "keywords": [],
          "license": null,
          "license_file": null,
          "links": null,
          "name": "bar",
          "readme": null,
          "readme_file": null,
          "repository": null,
          "vers": "0.2.0"
          }
        "#,
        "bar-0.2.0.crate",
        &["Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "src/main.rs"],
        &[(
            "Cargo.toml",
            &format!(
                r#"{}
[package]
name = "bar"
version = "0.2.0"
authors = []
resolver = "1"

[dependencies.dep]
version = "0.1"

[dev-dependencies.dep-dev]
version = "0.5.2"

[build-dependencies.dep-build]
version = "0.8"
"#,
                cargo::core::package::MANIFEST_PREAMBLE
            ),
        )],
    );
}

#[cargo_test]
fn inherit_own_detailed_dependencies() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "bar"
            version = "0.2.0"
            authors = []

            [dependencies]
            dep.workspace = true

            [workspace]
            members = []

            [workspace.dependencies]
            dep = { version = "0.1.2", features = ["testing"] }
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("dep", "0.1.2")
        .feature("testing", &vec![])
        .publish();

    p.cargo("build")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] dep v0.1.2 ([..])
[COMPILING] dep v0.1.2
[COMPILING] bar v0.2.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("check").run();
    let lockfile = p.read_lockfile();
    assert!(lockfile.contains("dep"));
    p.cargo("publish --token sekrit").run();
    publish::validate_upload_with_contents(
        r#"
        {
          "authors": [],
          "badges": {},
          "categories": [],
          "deps": [
            {
              "default_features": true,
              "features": ["testing"],
              "kind": "normal",
              "name": "dep",
              "optional": false,
              "registry": "https://github.com/rust-lang/crates.io-index",
              "target": null,
              "version_req": "^0.1.2"
            }
          ],
          "description": null,
          "documentation": null,
          "features": {},
          "homepage": null,
          "keywords": [],
          "license": null,
          "license_file": null,
          "links": null,
          "name": "bar",
          "readme": null,
          "readme_file": null,
          "repository": null,
          "vers": "0.2.0"
          }
        "#,
        "bar-0.2.0.crate",
        &["Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "src/main.rs"],
        &[(
            "Cargo.toml",
            &format!(
                r#"{}
[package]
name = "bar"
version = "0.2.0"
authors = []
resolver = "1"

[dependencies.dep]
version = "0.1.2"
features = ["testing"]
"#,
                cargo::core::package::MANIFEST_PREAMBLE
            ),
        )],
    );
}

#[cargo_test]
fn inherit_from_own_undefined_field() {
    registry::init();

    let p = project().build();

    let _ = git::repo(&paths::root().join("foo"))
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "1.2.5"
            authors = ["rustaceans"]
            description.workspace = true

            [workspace]
            members = []
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  error inheriting `description` from workspace root manifest's `workspace.package.description`

Caused by:
  `workspace.package.description` was not defined
",
        )
        .run();
}

#[cargo_test]
fn inherited_dependencies_union_features() {
    Package::new("dep", "0.1.0")
        .feature("fancy", &["fancy_dep"])
        .feature("dancy", &["dancy_dep"])
        .add_dep(Dependency::new("fancy_dep", "0.2").optional(true))
        .add_dep(Dependency::new("dancy_dep", "0.6").optional(true))
        .file("src/lib.rs", "")
        .publish();

    Package::new("fancy_dep", "0.2.4").publish();
    Package::new("dancy_dep", "0.6.8").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "bar"
            version = "0.2.0"
            authors = []
            [dependencies]
            dep = { workspace = true, features = ["dancy"] }

            [workspace]
            members = []
            [workspace.dependencies]
            dep = { version = "0.1", features = ["fancy"] }
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] fancy_dep v0.2.4 ([..])
[DOWNLOADED] dep v0.1.0 ([..])
[DOWNLOADED] dancy_dep v0.6.8 ([..])
[COMPILING] [..]
[COMPILING] [..]
[COMPILING] dep v0.1.0
[COMPILING] bar v0.2.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    let lockfile = p.read_lockfile();
    assert!(lockfile.contains("dep"));
    assert!(lockfile.contains("fancy_dep"));
    assert!(lockfile.contains("dancy_dep"));
}

#[cargo_test]
fn inherit_workspace_fields() {
    registry::init();

    let p = project().build();

    let _ = git::repo(&paths::root().join("foo"))
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            [workspace.package]
            version = "1.2.3"
            authors = ["Rustaceans"]
            description = "This is a crate"
            documentation = "https://www.rust-lang.org/learn"
            readme = "README.md"
            homepage = "https://www.rust-lang.org"
            repository = "https://github.com/example/example"
            license = "MIT"
            license-file = "LICENSE"
            keywords = ["cli"]
            categories = ["development-tools"]
            publish = true
            edition = "2018"
            rust-version = "1.60"
            exclude = ["foo.txt"]
            include = ["bar.txt", "**/*.rs", "Cargo.toml", "LICENSE", "README.md"]
            [workspace.package.badges]
            gitlab = { repository = "https://gitlab.com/rust-lang/rust", branch = "master" }
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "bar/Cargo.toml",
            r#"
            badges.workspace = true
            [package]
            name = "bar"
            workspace = ".."
            version.workspace = true
            authors.workspace = true
            description.workspace = true
            documentation.workspace = true
            readme.workspace = true
            homepage.workspace = true
            repository.workspace = true
            license.workspace = true
            license-file.workspace = true
            keywords.workspace = true
            categories.workspace = true
            publish.workspace = true
            edition.workspace = true
            rust-version.workspace = true
            exclude.workspace = true
            include.workspace = true
        "#,
        )
        .file("LICENSE", "license")
        .file("README.md", "README.md")
        .file("bar/src/main.rs", "fn main() {}")
        .file("bar/foo.txt", "") // should be ignored when packaging
        .file("bar/bar.txt", "") // should be included when packaging
        .build();

    p.cargo("publish --token sekrit").cwd("bar").run();
    publish::validate_upload_with_contents(
        r#"
        {
          "authors": ["Rustaceans"],
          "badges": {
            "gitlab": { "branch": "master", "repository": "https://gitlab.com/rust-lang/rust" }
          },
          "categories": ["development-tools"],
          "deps": [],
          "description": "This is a crate",
          "documentation": "https://www.rust-lang.org/learn",
          "features": {},
          "homepage": "https://www.rust-lang.org",
          "keywords": ["cli"],
          "license": "MIT",
          "license_file": "../LICENSE",
          "links": null,
          "name": "bar",
          "readme": "README.md",
          "readme_file": "../README.md",
          "repository": "https://github.com/example/example",
          "vers": "1.2.3"
          }
        "#,
        "bar-1.2.3.crate",
        &[
            "Cargo.lock",
            "Cargo.toml",
            "Cargo.toml.orig",
            "src/main.rs",
            "README.md",
            "LICENSE",
            ".cargo_vcs_info.json",
            "bar.txt",
        ],
        &[(
            "Cargo.toml",
            &format!(
                r#"{}
[package]
edition = "2018"
rust-version = "1.60"
name = "bar"
version = "1.2.3"
authors = ["Rustaceans"]
exclude = ["foo.txt"]
include = [
    "bar.txt",
    "**/*.rs",
    "Cargo.toml",
    "LICENSE",
    "README.md",
]
publish = true
description = "This is a crate"
homepage = "https://www.rust-lang.org"
documentation = "https://www.rust-lang.org/learn"
readme = "README.md"
keywords = ["cli"]
categories = ["development-tools"]
license = "MIT"
license-file = "LICENSE"
repository = "https://github.com/example/example"
resolver = "1"

[badges.gitlab]
branch = "master"
repository = "https://gitlab.com/rust-lang/rust"
"#,
                cargo::core::package::MANIFEST_PREAMBLE
            ),
        )],
    );
}

#[cargo_test]
fn inherit_dependencies() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            [workspace.dependencies]
            dep = "0.1"
            dep-build = "0.8"
            dep-dev = "0.5.2"
        "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
            [project]
            workspace = ".."
            name = "bar"
            version = "0.2.0"
            authors = []
            [dependencies]
            dep.workspace = true
            [build-dependencies]
            dep-build.workspace = true
            [dev-dependencies]
            dep-dev.workspace = true
        "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    Package::new("dep", "0.1.2").publish();
    Package::new("dep-build", "0.8.2").publish();
    Package::new("dep-dev", "0.5.2").publish();

    p.cargo("build")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] dep-build v0.8.2 ([..])
[DOWNLOADED] dep v0.1.2 ([..])
[COMPILING] dep v0.1.2
[COMPILING] bar v0.2.0 ([CWD]/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("check").run();
    let lockfile = p.read_lockfile();
    assert!(lockfile.contains("dep"));
    assert!(lockfile.contains("dep-dev"));
    assert!(lockfile.contains("dep-build"));
    p.cargo("publish --token sekrit").cwd("bar").run();
    publish::validate_upload_with_contents(
        r#"
        {
          "authors": [],
          "badges": {},
          "categories": [],
          "deps": [
            {
              "default_features": true,
              "features": [],
              "kind": "normal",
              "name": "dep",
              "optional": false,
              "registry": "https://github.com/rust-lang/crates.io-index",
              "target": null,
              "version_req": "^0.1"
            },
            {
              "default_features": true,
              "features": [],
              "kind": "dev",
              "name": "dep-dev",
              "optional": false,
              "registry": "https://github.com/rust-lang/crates.io-index",
              "target": null,
              "version_req": "^0.5.2"
            },
            {
              "default_features": true,
              "features": [],
              "kind": "build",
              "name": "dep-build",
              "optional": false,
              "registry": "https://github.com/rust-lang/crates.io-index",
              "target": null,
              "version_req": "^0.8"
            }
          ],
          "description": null,
          "documentation": null,
          "features": {},
          "homepage": null,
          "keywords": [],
          "license": null,
          "license_file": null,
          "links": null,
          "name": "bar",
          "readme": null,
          "readme_file": null,
          "repository": null,
          "vers": "0.2.0"
          }
        "#,
        "bar-0.2.0.crate",
        &["Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "src/main.rs"],
        &[(
            "Cargo.toml",
            &format!(
                r#"{}
[package]
name = "bar"
version = "0.2.0"
authors = []
resolver = "1"

[dependencies.dep]
version = "0.1"

[dev-dependencies.dep-dev]
version = "0.5.2"

[build-dependencies.dep-build]
version = "0.8"
"#,
                cargo::core::package::MANIFEST_PREAMBLE
            ),
        )],
    );
}

#[cargo_test]
fn inherit_target_dependencies() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            [workspace.dependencies]
            dep = "0.1"
        "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
            [project]
            workspace = ".."
            name = "bar"
            version = "0.2.0"
            authors = []
            [target.'cfg(unix)'.dependencies]
            dep.workspace = true
            [target.'cfg(windows)'.dependencies]
            dep.workspace = true
        "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    Package::new("dep", "0.1.2").publish();

    p.cargo("build")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] dep v0.1.2 ([..])
[COMPILING] dep v0.1.2
[COMPILING] bar v0.2.0 ([CWD]/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    let lockfile = p.read_lockfile();
    assert!(lockfile.contains("dep"));
}

#[cargo_test]
fn inherit_dependency_override_optional() {
    Package::new("dep", "0.1.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            [workspace.dependencies]
            dep = "0.1.0"
        "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
            [project]
            workspace = ".."
            name = "bar"
            version = "0.2.0"
            authors = []
            [dependencies]
            dep = { workspace = true, optional = true }
        "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[COMPILING] bar v0.2.0 ([CWD]/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test]
fn inherit_dependency_features() {
    Package::new("dep", "0.1.0")
        .feature("fancy", &["fancy_dep"])
        .add_dep(Dependency::new("fancy_dep", "0.2").optional(true))
        .file("src/lib.rs", "")
        .publish();

    Package::new("fancy_dep", "0.2.4").publish();
    Package::new("dancy_dep", "0.6.8").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [project]
            name = "bar"
            version = "0.2.0"
            authors = []
            [dependencies]
            dep = { workspace = true, features = ["fancy"] }

            [workspace]
            members = []
            [workspace.dependencies]
            dep = "0.1"
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[UPDATING] `[..]` index
[DOWNLOADING] crates ...
[DOWNLOADED] fancy_dep v0.2.4 ([..])
[DOWNLOADED] dep v0.1.0 ([..])
[COMPILING] fancy_dep v0.2.4
[COMPILING] dep v0.1.0
[COMPILING] bar v0.2.0 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    let lockfile = p.read_lockfile();
    assert!(lockfile.contains("dep"));
    assert!(lockfile.contains("fancy_dep"));
}

#[cargo_test]
fn inherit_detailed_dependencies() {
    let git_project = git::new("detailed", |project| {
        project
            .file("Cargo.toml", &basic_lib_manifest("detailed"))
            .file(
                "src/detailed.rs",
                r#"
                pub fn hello() -> &'static str {
                    "hello world"
                }
            "#,
            )
    });

    // Make a new branch based on the current HEAD commit
    let repo = git2::Repository::open(&git_project.root()).unwrap();
    let head = repo.head().unwrap().target().unwrap();
    let head = repo.find_commit(head).unwrap();
    repo.branch("branchy", &head, true).unwrap();

    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
            [workspace]
            members = ["bar"]
            [workspace.dependencies]
            detailed = {{ git = '{}', branch = "branchy" }}
        "#,
                git_project.url()
            ),
        )
        .file(
            "bar/Cargo.toml",
            r#"
            [project]
            workspace = ".."
            name = "bar"
            version = "0.2.0"
            authors = []
            [dependencies]
            detailed.workspace = true
        "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    let git_root = git_project.root();

    p.cargo("build")
        .with_stderr(&format!(
            "\
[UPDATING] git repository `{}`\n\
[COMPILING] detailed v0.5.0 ({}?branch=branchy#[..])\n\
[COMPILING] bar v0.2.0 ([CWD]/bar)\n\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]\n",
            path2url(&git_root),
            path2url(&git_root),
        ))
        .run();
}

#[cargo_test]
fn inherit_path_dependencies() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            [workspace.dependencies]
            dep = { path = "dep" }
        "#,
        )
        .file(
            "bar/Cargo.toml",
            r#"
            [project]
            workspace = ".."
            name = "bar"
            version = "0.2.0"
            authors = []
            [dependencies]
            dep.workspace = true
        "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .file("dep/Cargo.toml", &basic_manifest("dep", "0.9.0"))
        .file("dep/src/lib.rs", "")
        .build();

    p.cargo("build")
        .with_stderr(
            "\
[COMPILING] dep v0.9.0 ([CWD]/dep)
[COMPILING] bar v0.2.0 ([CWD]/bar)
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    let lockfile = p.read_lockfile();
    assert!(lockfile.contains("dep"));
}

#[cargo_test]
fn error_workspace_false() {
    registry::init();

    let p = project().build();

    let _ = git::repo(&paths::root().join("foo"))
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            workspace = ".."
            version = "1.2.3"
            authors = ["rustaceans"]
            description = { workspace = false }
        "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .cwd("bar")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  `workspace=false` is unsupported for `package.description`
",
        )
        .run();
}

#[cargo_test]
fn error_workspace_dependency_looked_for_workspace_itself() {
    registry::init();

    let p = project().build();

    let _ = git::repo(&paths::root().join("foo"))
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "bar"
            version = "1.2.3"
            workspace = ".."

            [dependencies]
            dep.workspace = true

            [workspace]
            members = ["bar"]

            [workspace.dependencies]
            dep.workspace = true

            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  dep was specified as `workspace.dependencies.dep.workspace = true`, but \
  workspace dependencies cannot specify `workspace = true`
",
        )
        .run();
}

#[cargo_test]
fn error_malformed_workspace_root() {
    registry::init();

    let p = project().build();

    let _ = git::repo(&paths::root().join("foo"))
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = [invalid toml
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            workspace = ".."
            version = "1.2.3"
            authors = ["rustaceans"]
        "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .cwd("bar")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]/foo/Cargo.toml`

Caused by:
  [..]

Caused by:
  [..]
    |
  3 |             members = [invalid toml
    |                        ^
  Unexpected `i`
  Expected newline or `#`
",
        )
        .run();
}

#[cargo_test]
fn error_no_root_workspace() {
    registry::init();

    let p = project().build();

    let _ = git::repo(&paths::root().join("foo"))
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            workspace = ".."
            version = "1.2.3"
            authors = ["rustaceans"]
            description.workspace = true
        "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .cwd("bar")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[..]/Cargo.toml`

Caused by:
  error inheriting `description` from workspace root manifest's `workspace.package.description`

Caused by:
  root of a workspace inferred but wasn't a root: [..]/Cargo.toml
",
        )
        .run();
}

#[cargo_test]
fn error_inherit_unspecified_dependency() {
    let p = project().build();

    let _ = git::repo(&paths::root().join("foo"))
        .file(
            "Cargo.toml",
            r#"
            [workspace]
            members = ["bar"]
            "#,
        )
        .file("src/main.rs", "fn main() {}")
        .file(
            "bar/Cargo.toml",
            r#"
            [package]
            name = "bar"
            workspace = ".."
            version = "1.2.3"
            authors = ["rustaceans"]
            [dependencies]
            foo.workspace = true
        "#,
        )
        .file("bar/src/main.rs", "fn main() {}")
        .build();

    p.cargo("build")
        .cwd("bar")
        .with_status(101)
        .with_stderr(
            "\
[ERROR] failed to parse manifest at `[CWD]/Cargo.toml`

Caused by:
  error reading `dependencies.foo` from workspace root manifest's `workspace.dependencies.foo`

Caused by:
  `workspace.dependencies` was not defined
",
        )
        .run();
}
