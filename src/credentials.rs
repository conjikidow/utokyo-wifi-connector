use std::io;

use dialoguer::{Input, Password};

use crate::error::{Error, Result};

pub const IDENTITY_DOMAIN: &str = "wifi.u-tokyo.ac.jp";

pub fn normalize_identity(input: &str) -> Result<String> {
    let input = input.trim();
    let lowercase = input.to_ascii_lowercase();
    let (local, domain) = lowercase
        .split_once('@')
        .unwrap_or((&lowercase, IDENTITY_DOMAIN));
    let is_valid_local = local
        .strip_prefix('u')
        .is_some_and(|digits| digits.len() == 7 && digits.bytes().all(|b| b.is_ascii_digit()));
    if !is_valid_local || domain != IDENTITY_DOMAIN {
        return Err(Error::InvalidUserId(input.to_owned()));
    }
    Ok(format!("{local}@{IDENTITY_DOMAIN}"))
}

pub fn prompt_identity() -> Result<String> {
    let input: String = Input::new()
        .with_prompt("UTokyo Wi-Fi user ID")
        .validate_with(|input: &String| {
            normalize_identity(input)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .interact_text()?;
    normalize_identity(&input)
}

pub fn prompt_password() -> Result<String> {
    Ok(Password::new()
        .with_prompt("UTokyo Wi-Fi password")
        .interact()?)
}

pub fn read_password_stdin() -> Result<String> {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(Error::PasswordStdin)?;
    let password = line.trim_end_matches(['\n', '\r']);
    if password.is_empty() {
        return Err(Error::EmptyPassword);
    }
    Ok(password.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_identity_appends_domain() {
        assert_eq!(
            normalize_identity("u1234567").unwrap(),
            "u1234567@wifi.u-tokyo.ac.jp"
        );
    }

    #[test]
    fn normalize_identity_accepts_full_identity() {
        assert_eq!(
            normalize_identity(" u1234567@wifi.u-tokyo.ac.jp ").unwrap(),
            "u1234567@wifi.u-tokyo.ac.jp"
        );
    }

    #[test]
    fn normalize_identity_lowercases_input() {
        assert_eq!(
            normalize_identity("U1234567@WiFi.U-Tokyo.ac.jp").unwrap(),
            "u1234567@wifi.u-tokyo.ac.jp"
        );
    }

    #[test]
    fn normalize_identity_rejects_other_domain() {
        assert!(matches!(
            normalize_identity("u1234567@u-tokyo.ac.jp"),
            Err(Error::InvalidUserId(_))
        ));
    }

    #[test]
    fn normalize_identity_rejects_empty_local_part() {
        assert!(normalize_identity("").is_err());
        assert!(normalize_identity("@wifi.u-tokyo.ac.jp").is_err());
    }

    #[test]
    fn normalize_identity_rejects_malformed_user_id() {
        for user_id in ["u26443699", "u123456", "u12a4567", "1234567890", "x1234567"] {
            assert!(
                matches!(normalize_identity(user_id), Err(Error::InvalidUserId(_))),
                "{user_id} should be rejected"
            );
        }
    }
}
