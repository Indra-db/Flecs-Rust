mod rustgen;

use rustgen::RustGenerator;

fn main() -> anyhow::Result<()> {
    let generator = RustGenerator;
    poly_doctest::run_cli(generator)?;
    Ok(())
}
