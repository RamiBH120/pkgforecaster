mod ui;
use anyhow::Result;
use gtk4::prelude::*;


fn main() -> Result<()> {
    // initialize GTK
    gtk4::init()?;
    ui::run()?;
    Ok(())
}
