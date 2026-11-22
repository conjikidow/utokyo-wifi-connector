use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`nmcli` was not found; install NetworkManager")]
    NmcliNotFound,

    #[error("failed to run `nmcli`: {0}")]
    NmcliIo(#[source] io::Error),

    #[error("`nmcli {command}` failed: {message}")]
    NmcliFailed { command: String, message: String },

    #[error("unexpected output from `nmcli {command}`")]
    NmcliUnexpectedOutput { command: String },

    #[error("failed to generate a connection UUID: {0}")]
    Uuid(#[source] io::Error),

    #[error(
        "invalid user ID `{0}`: expected `u` followed by 7 digits, optionally with `@wifi.u-tokyo.ac.jp`"
    )]
    InvalidUserId(String),

    #[error("the password stored by NetworkManager does not match the input")]
    PasswordMismatch,

    #[error(
        "authentication failed; check the user ID and password, or see `journalctl -u NetworkManager` for details"
    )]
    AuthenticationFailed,

    #[error("{}", not_in_range(.0))]
    NotInRange(Vec<String>),

    #[error("failed to process some networks")]
    Reported,

    #[error("interrupted")]
    Interrupted,

    #[error("failed to set the Ctrl-C handler: {0}")]
    SignalHandler(#[from] ctrlc::Error),

    #[error("failed to read the password from standard input: {0}")]
    PasswordStdin(#[source] io::Error),

    #[error("the password is empty")]
    EmptyPassword,

    #[error(
        "cannot prompt without a terminal; use --yes, and for `setup` also --user-id and --password-stdin"
    )]
    NotATerminal,

    #[error(transparent)]
    Prompt(dialoguer::Error),
}

impl From<dialoguer::Error> for Error {
    fn from(error: dialoguer::Error) -> Self {
        match error {
            dialoguer::Error::IO(e) if e.kind() == io::ErrorKind::NotConnected => {
                Self::NotATerminal
            }
            error => Self::Prompt(error),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn not_in_range(ssids: &[String]) -> String {
    let (verb, pronoun, object) = if ssids.len() == 1 {
        ("is", "it", "it")
    } else {
        ("are", "they", "them")
    };
    format!(
        "{} {verb} not in range; nothing was set up (retry where {pronoun} {verb} in range, or use --no-verify to set {object} up without connecting)",
        ssids.join(" and ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_in_range_agrees_with_number_of_networks() {
        assert_eq!(
            Error::NotInRange(vec!["0000UTokyo".to_owned()]).to_string(),
            "0000UTokyo is not in range; nothing was set up (retry where it is in range, or use --no-verify to set it up without connecting)"
        );
        assert_eq!(
            Error::NotInRange(vec!["0000UTokyo".to_owned(), "eduroam".to_owned()]).to_string(),
            "0000UTokyo and eduroam are not in range; nothing was set up (retry where they are in range, or use --no-verify to set them up without connecting)"
        );
    }
}
