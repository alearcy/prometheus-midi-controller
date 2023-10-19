use std::error::Error;

use ui::run;

mod faders;
mod midi;
mod serial;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run().await?;
    Ok(())
}
