pub mod completions;
pub mod delete;
pub mod setup;
pub mod status;

use dialoguer::Confirm;

use crate::error::Result;
use crate::nm::Connection;

fn confirm(action: &str, ssid: &str, connections: &[Connection]) -> Result<bool> {
    eprintln!("Existing connection(s) for {ssid}:");
    for connection in connections {
        eprintln!("  '{}' ({})", connection.name, connection.uuid);
    }
    Ok(Confirm::new()
        .with_prompt(format!("{action} them?"))
        .default(false)
        .interact()?)
}
