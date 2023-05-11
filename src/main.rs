use runtime::Runtime;
use std::time::Duration;

pub use crate::prelude::*;

pub mod device;
pub mod prelude;
pub mod runtime;

fn main() {
    env_logger::init();
    let mut rt = Runtime::new().expect("Error creating PAPR Runtime");
    rt.run(async move {
        log::info!("I'm running on the PAPR runtime!");
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    })
    .unwrap();
}
