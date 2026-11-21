use crate::cli::DeleteArgs;
use crate::error::{Error, Result};
use crate::nm::{NetworkManager, Runner};

pub fn run(nm: &NetworkManager<impl Runner>, args: &DeleteArgs) -> Result<()> {
    let mut failed = false;
    for &network in args.network.network.networks() {
        let existing = nm.find_by_ssid(network.ssid())?;
        if existing.is_empty() {
            println!("No connection found for {}.", network.ssid());
            continue;
        }
        if !args.yes && !super::confirm("Delete", network.ssid(), &existing)? {
            println!("Skipped {}.", network.ssid());
            continue;
        }
        for connection in &existing {
            match nm.delete(&connection.uuid) {
                Ok(()) => println!(
                    "Deleted '{}' ({}) for {}.",
                    connection.name,
                    connection.uuid,
                    network.ssid()
                ),
                Err(e) => {
                    eprintln!(
                        "error: failed to delete '{}' ({}): {e}",
                        connection.name, connection.uuid
                    );
                    failed = true;
                }
            }
        }
    }
    if failed { Err(Error::Reported) } else { Ok(()) }
}
