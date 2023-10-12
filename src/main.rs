use std::error::Error;

use ui::run;

mod faders;
mod midi;
mod serial;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    run()?;
    Ok(())
}
