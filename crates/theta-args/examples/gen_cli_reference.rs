/// generate docs/reference/cli.md from clap definitions
///
/// usage:
///     cargo run -p theta-args --features docgen --example gen_cli_reference > docs/reference/cli.md
fn main() -> std::io::Result<()> {
    use std::io::Write;

    let md = theta_args::generate_cli_reference();

    let mut out = std::io::stdout().lock();
    out.write_all(b"# CLI reference\n\n")?;
    out.write_all(b"<!-- generated - do not edit by hand -->\n")?;
    out.write_all(
        b"<!-- regenerate: cargo run -p theta-args --features docgen --example gen_cli_reference > docs/reference/cli.md -->\n\n",
    )?;
    out.write_all(md.as_bytes())?;
    Ok(())
}
