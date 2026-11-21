use crate::cli::StatusArgs;
use crate::error::Result;
use crate::nm::{NetworkManager, Runner};

pub fn run(nm: &NetworkManager<impl Runner>, args: &StatusArgs) -> Result<()> {
    let mut unverified = false;
    for &network in args.network.network.networks() {
        let existing = nm.find_by_ssid(network.ssid())?;
        if existing.is_empty() {
            println!("{}: not configured", network.ssid());
            continue;
        }
        for connection in &existing {
            let [
                identity,
                system_ca_certs,
                ca_cert,
                domain_suffix_match,
                domain_match,
            ] = nm
                .optional_properties(
                    &connection.uuid,
                    [
                        "802-1x.identity",
                        "802-1x.system-ca-certs",
                        "802-1x.ca-cert",
                        "802-1x.domain-suffix-match",
                        "802-1x.domain-match",
                    ],
                )?
                .unwrap_or_default();
            let verified = (system_ca_certs == "yes" || !ca_cert.is_empty())
                && (!domain_suffix_match.is_empty() || !domain_match.is_empty());
            unverified |= !verified;
            println!(
                "{}: '{}' ({})",
                network.ssid(),
                connection.name,
                connection.uuid
            );
            println!(
                "  identity: {}",
                if identity.is_empty() {
                    "(none)"
                } else {
                    &identity
                }
            );
            println!(
                "  server certificate verification: {}",
                if verified { "enabled" } else { "disabled" }
            );
        }
    }
    if unverified {
        eprintln!(
            "warning: some connections do not verify the server certificate; run `utokyo-wifi setup` to replace them"
        );
    }
    Ok(())
}
