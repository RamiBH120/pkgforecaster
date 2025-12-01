mod ui;
use anyhow::Result;

fn main() -> Result<()> {
    // initialize GTK
    gtk4::init()?;
    ui::run()?;
    Ok(())
}
