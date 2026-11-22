use std::sync::atomic::{AtomicBool, Ordering};

use crate::cli::{NetworkSelection, SetupArgs};
use crate::credentials::{
    normalize_identity, prompt_identity, prompt_password, read_password_stdin,
};
use crate::error::{Error, Result};
use crate::network::{Network, verified_settings};
use crate::nm::{Connection, NetworkManager, Runner};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn run(nm: &NetworkManager<impl Runner>, args: &SetupArgs) -> Result<()> {
    let user_identity = args
        .user_id
        .as_deref()
        .map(normalize_identity)
        .transpose()?;
    let verify = !args.no_verify;
    let visible_ssids = if verify {
        nm.visible_ssids()?
    } else {
        Vec::new()
    };

    let networks = args.network.unwrap_or(NetworkSelection::All).networks();
    let out_of_range: Vec<Network> = networks
        .iter()
        .copied()
        .filter(|network| verify && !visible_ssids.iter().any(|ssid| ssid == network.ssid()))
        .collect();
    if !out_of_range.is_empty() && (args.network.is_some() || out_of_range.len() == networks.len())
    {
        return Err(Error::NotInRange(
            out_of_range
                .iter()
                .map(|network| network.ssid().to_owned())
                .collect(),
        ));
    }

    let mut failed = false;
    let mut targets = Vec::new();
    for &network in networks {
        if out_of_range.contains(&network) {
            println!(
                "Skipped {}: not in range (retry where it is in range, or use --no-verify to set it up without connecting).",
                network.ssid()
            );
            continue;
        }
        let existing = nm.find_by_ssid(network.ssid())?;
        if !existing.is_empty()
            && !args.yes
            && !super::confirm("Replace", network.ssid(), &existing)?
        {
            println!("Skipped {}.", network.ssid());
            continue;
        }
        targets.push((network, existing));
    }

    if !targets.is_empty() {
        let identity = match user_identity {
            Some(identity) => identity,
            None => prompt_identity()?,
        };
        let password = if args.password_stdin {
            read_password_stdin()?
        } else {
            prompt_password()?
        };

        ctrlc::set_handler(|| INTERRUPTED.store(true, Ordering::SeqCst))?;
        for (network, existing) in targets {
            if INTERRUPTED.load(Ordering::SeqCst) {
                println!("Skipped {}.", network.ssid());
                failed = true;
                continue;
            }
            match apply(nm, network, &existing, &identity, &password, verify) {
                Ok(uuid) if verify => println!("Set up {} ({uuid}), verified.", network.ssid()),
                Ok(uuid) => println!("Set up {} ({uuid}) without verifying it.", network.ssid()),
                Err(e) => {
                    eprintln!("error: failed to set up {}: {e}", network.ssid());
                    failed = true;
                }
            }
        }
    }

    if !failed {
        Ok(())
    } else if INTERRUPTED.load(Ordering::SeqCst) {
        Err(Error::Interrupted)
    } else {
        Err(Error::Reported)
    }
}

fn apply(
    nm: &NetworkManager<impl Runner>,
    network: Network,
    existing: &[Connection],
    identity: &str,
    password: &str,
    verify: bool,
) -> Result<String> {
    let uuid = nm.add(&network.connection_settings(identity))?;
    let mut previous = Vec::new();
    let result = nm.set_password(&uuid, password).and_then(|()| {
        if verify {
            let active = nm.active_wifi_uuids()?;
            if INTERRUPTED.load(Ordering::SeqCst) {
                return Err(Error::Interrupted);
            }
            previous = active;
            nm.activate(&uuid)?;
        }
        nm.modify(&uuid, &verified_settings())
    });
    if let Err(e) = result {
        if let Err(delete_error) = nm.delete(&uuid) {
            eprintln!("warning: failed to delete the new connection {uuid}: {delete_error}");
        }
        for previous_uuid in &previous {
            if let Err(restore_error) = nm.activate(previous_uuid) {
                eprintln!("warning: failed to reconnect to {previous_uuid}: {restore_error}");
            }
        }
        return Err(e);
    }
    for connection in existing {
        if let Err(e) = nm.delete(&connection.uuid) {
            eprintln!(
                "warning: failed to delete the existing connection '{}' ({}): {e}",
                connection.name, connection.uuid
            );
        }
    }
    Ok(uuid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nm::testing::FakeRunner;

    const ACTIVE: &str = "-t -f UUID,TYPE connection show --active";

    fn existing() -> Vec<Connection> {
        vec![Connection {
            uuid: "old".to_owned(),
            name: "eduroam".to_owned(),
        }]
    }

    fn commands(runner: &FakeRunner) -> Vec<String> {
        runner
            .calls
            .borrow()
            .iter()
            .map(|(args, _)| args.join(" "))
            .collect()
    }

    fn new_uuid(runner: &FakeRunner) -> String {
        runner.calls.borrow()[0].0[3].clone()
    }

    #[test]
    fn apply_replaces_existing_connections_after_activation() {
        let runner = FakeRunner::default()
            .respond(true, "")
            .respond(true, "")
            .respond(true, "secret\n")
            .respond(true, "prev:802-11-wireless\n")
            .respond(true, "")
            .respond(true, "")
            .respond(true, "");
        let nm = NetworkManager::new(&runner);

        let uuid = apply(&nm, Network::Eduroam, &existing(), "id", "secret", true).unwrap();

        let commands = commands(&runner);
        assert!(commands[0].starts_with("connection add"));
        assert!(commands[0].contains("connection.autoconnect no"));
        assert_eq!(commands[1], format!("connection edit uuid {uuid}"));
        assert_eq!(commands[3], ACTIVE);
        assert_eq!(commands[4], format!("--wait 60 connection up uuid {uuid}"));
        assert_eq!(
            commands[5],
            format!(
                "connection modify uuid {uuid} connection.autoconnect yes connection.auth-retries -1"
            )
        );
        assert_eq!(commands[6], "connection delete uuid old");
        assert_eq!(commands.len(), 7);
    }

    #[test]
    fn apply_rolls_back_when_activation_fails() {
        let runner = FakeRunner::default()
            .respond(true, "")
            .respond(true, "")
            .respond(true, "secret\n")
            .respond(true, "prev:802-11-wireless\n")
            .fail_with("Error: Secrets were required, but not provided\n")
            .respond(true, "")
            .respond(true, "");
        let nm = NetworkManager::new(&runner);

        let error = apply(&nm, Network::Eduroam, &existing(), "id", "secret", true).unwrap_err();

        assert!(matches!(error, Error::AuthenticationFailed));
        let uuid = new_uuid(&runner);
        let commands = commands(&runner);
        assert_eq!(commands[5], format!("connection delete uuid {uuid}"));
        assert_eq!(commands[6], "--wait 60 connection up uuid prev");
        assert_eq!(commands.len(), 7);
    }

    #[test]
    fn apply_deletes_new_connection_without_reconnecting_when_password_is_not_stored() {
        let runner = FakeRunner::default()
            .respond(true, "")
            .respond(true, "")
            .respond(true, "other\n")
            .respond(true, "");
        let nm = NetworkManager::new(&runner);

        let error = apply(&nm, Network::Eduroam, &existing(), "id", "secret", true).unwrap_err();

        assert!(matches!(error, Error::PasswordMismatch));
        let uuid = new_uuid(&runner);
        let commands = commands(&runner);
        assert_eq!(commands[3], format!("connection delete uuid {uuid}"));
        assert_eq!(commands.len(), 4);
    }

    #[test]
    fn apply_without_verification_does_not_activate() {
        let runner = FakeRunner::default()
            .respond(true, "")
            .respond(true, "")
            .respond(true, "secret\n")
            .respond(true, "")
            .respond(true, "");
        let nm = NetworkManager::new(&runner);

        let uuid = apply(&nm, Network::Eduroam, &existing(), "id", "secret", false).unwrap();

        let commands = commands(&runner);
        assert!(commands[3].starts_with(&format!("connection modify uuid {uuid}")));
        assert_eq!(commands[4], "connection delete uuid old");
        assert_eq!(commands.len(), 5);
    }

    #[test]
    fn apply_succeeds_when_existing_connection_cannot_be_deleted() {
        let runner = FakeRunner::default()
            .respond(true, "")
            .respond(true, "")
            .respond(true, "secret\n")
            .respond(true, "")
            .respond(false, "");
        let nm = NetworkManager::new(&runner);

        assert!(apply(&nm, Network::Eduroam, &existing(), "id", "secret", false).is_ok());
    }
}
