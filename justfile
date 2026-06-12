# theta development task runner

alias t := test
alias c := check
alias f := fmt

# install cargo dev tools via cargo-binstall
[group: 'setup']
install-tools:
    cargo install cargo-binstall --locked
    cargo binstall cargo-nextest cargo-deny cargo-shear typos-cli --locked -y

# bootstrap a fresh clone (tools + hooks + deps)
[group: 'setup']
setup: install-tools
    lefthook install
    cargo fetch --locked

# run all tests (local only)
[group: 'dev']
test:
    cargo nextest run --workspace --locked

# run all tests including online integration tests
[group: 'dev']
test-online:
    cargo nextest run --workspace --locked --features online-tests

# run tests for a single crate
[group: 'dev']
test-crate name:
    cargo nextest run -p {{ name }} --locked

# format all code
[group: 'dev']
fmt:
    cargo fmt --all

# run all CI checks locally
[group: 'check']
check: check-fmt lint test check-deny check-typos check-shear check-cli-docs

# check formatting without modifying
[group: 'check']
check-fmt:
    cargo fmt --all --check

# clippy pedantic + deny warnings
[group: 'check']
lint:
    cargo clippy --workspace --all-targets --locked -- -D warnings

# license + advisory + source audit
[group: 'check']
check-deny:
    cargo deny check advisories licenses sources

# spell check
[group: 'check']
check-typos:
    typos

# unused dependency check
[group: 'check']
check-shear:
    cargo shear

# regenerate docs/reference/cli.md from clap definitions
[group: 'docs']
gen-cli-docs output="docs/reference/cli.md":
    cargo run -p theta-args --features docgen --example gen_cli_reference 2>/dev/null > {{ output }}

# regenerate the schema
[group: 'docs']
gen-schema: 
    cargo run -- schema 

# fail if docs/reference/cli.md is stale
[group: 'check']
check-cli-docs:
    cargo run -p theta-args --features docgen --example gen_cli_reference 2>/dev/null | diff -u docs/reference/cli.md - || { echo "run: just gen-cli-docs"; exit 1; }

# build and serve docs locally
[group: 'docs']
docs-serve:
    mkdocs serve --livereload

# install theta from source into ~/.local/bin (takes priority over ~/.cargo/bin)
# cargo install --root sets the installation root; binary lands at <root>/bin/theta.
[group: 'dev']
install:
    cargo install --path crates/theta --root ~/.local --locked
