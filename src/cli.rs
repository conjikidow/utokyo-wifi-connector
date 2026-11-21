use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::network::Network;

#[derive(Parser)]
#[command(name = "utokyo-wifi", bin_name = "utokyo-wifi", version)]
#[command(about = "Manage NetworkManager connections to the UTokyo Wi-Fi and eduroam networks")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create the connections once they connect, replacing existing ones for the same SSIDs
    Setup(SetupArgs),

    /// Delete the connections
    Delete(DeleteArgs),

    /// Show the connections and whether they verify the server certificate
    Status(StatusArgs),

    /// Generate a shell completion script
    Completions(CompletionsArgs),
}

#[derive(Args)]
pub struct NetworkArgs {
    /// Network to operate on
    #[arg(long, value_enum, default_value_t = NetworkSelection::All)]
    pub network: NetworkSelection,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum NetworkSelection {
    Utokyo,
    Eduroam,
    All,
}

impl NetworkSelection {
    pub fn networks(self) -> &'static [Network] {
        match self {
            Self::Utokyo => &[Network::UTokyo],
            Self::Eduroam => &[Network::Eduroam],
            Self::All => &Network::ALL,
        }
    }
}

#[derive(Args)]
pub struct SetupArgs {
    /// Network to set up, which must be in range unless `--no-verify` is given
    /// [default: both, skipping those not in range]
    #[arg(long, value_enum)]
    pub network: Option<NetworkSelection>,

    /// UTokyo Wi-Fi user ID, with or without `@wifi.u-tokyo.ac.jp`
    #[arg(long, value_name = "ID")]
    pub user_id: Option<String>,

    /// Read the password from the first line of standard input
    #[arg(long, requires_all = ["user_id", "yes"])]
    pub password_stdin: bool,

    /// Replace existing connections without confirmation
    #[arg(short, long)]
    pub yes: bool,

    /// Create the connections without connecting to them first
    #[arg(long)]
    pub no_verify: bool,
}

#[derive(Args)]
pub struct DeleteArgs {
    #[command(flatten)]
    pub network: NetworkArgs,

    /// Delete without confirmation
    #[arg(short, long)]
    pub yes: bool,
}

#[derive(Args)]
pub struct StatusArgs {
    #[command(flatten)]
    pub network: NetworkArgs,
}

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate the completion script for
    #[arg(value_enum)]
    pub shell: Shell,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn setup_distinguishes_omitted_network_from_all() {
        let Commands::Setup(args) = Cli::parse_from(["utokyo-wifi", "setup"]).command else {
            panic!("expected setup");
        };
        assert!(args.network.is_none());

        let Commands::Setup(args) =
            Cli::parse_from(["utokyo-wifi", "setup", "--network", "all"]).command
        else {
            panic!("expected setup");
        };
        assert!(matches!(args.network, Some(NetworkSelection::All)));
    }

    #[test]
    fn password_stdin_requires_user_id_and_yes() {
        assert!(Cli::try_parse_from(["utokyo-wifi", "setup", "--password-stdin"]).is_err());
        assert!(
            Cli::try_parse_from([
                "utokyo-wifi",
                "setup",
                "--password-stdin",
                "--user-id",
                "u1234567"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "utokyo-wifi",
                "setup",
                "--password-stdin",
                "--user-id",
                "u1234567",
                "--yes"
            ])
            .is_ok()
        );
    }
}
