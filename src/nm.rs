use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

const UUID_SOURCE: &str = "/proc/sys/kernel/random/uuid";
const ACTIVATION_TIMEOUT_SECS: &str = "60";
const SECRETS_REQUIRED: &str = "Secrets were required, but not provided";

pub struct Output {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub trait Runner {
    fn run(&self, args: &[&str], stdin: Option<&str>) -> Result<Output>;
}

pub struct Nmcli;

impl Runner for Nmcli {
    fn run(&self, args: &[&str], stdin: Option<&str>) -> Result<Output> {
        let mut child = Command::new("nmcli")
            .args(args)
            .env("LC_ALL", "C")
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => Error::NmcliNotFound,
                _ => Error::NmcliIo(e),
            })?;
        if let Some(input) = stdin {
            child
                .stdin
                .take()
                .expect("stdin is piped")
                .write_all(input.as_bytes())
                .map_err(Error::NmcliIo)?;
        }
        let output = child.wait_with_output().map_err(Error::NmcliIo)?;
        Ok(Output {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Connection {
    pub uuid: String,
    pub name: String,
}

pub struct NetworkManager<R> {
    runner: R,
}

impl<R: Runner> NetworkManager<R> {
    pub const fn new(runner: R) -> Self {
        Self { runner }
    }

    pub fn find_by_ssid(&self, ssid: &str) -> Result<Vec<Connection>> {
        let listing = self.nmcli(&["-t", "-f", "UUID,TYPE,NAME", "connection", "show"])?;
        let mut wifi_connections = Vec::new();
        for line in listing.lines() {
            let [uuid, kind, name] = <[String; 3]>::try_from(split_terse(line)).map_err(|_| {
                Error::NmcliUnexpectedOutput {
                    command: "connection show".to_owned(),
                }
            })?;
            if kind == "802-11-wireless" {
                wifi_connections.push(Connection { uuid, name });
            }
        }
        let uuids: Vec<&str> = wifi_connections
            .iter()
            .map(|connection| connection.uuid.as_str())
            .collect();
        let ssids = self.properties(&uuids, ["802-11-wireless.ssid"])?;
        Ok(wifi_connections
            .into_iter()
            .zip(ssids)
            .filter(|(_, [connection_ssid])| connection_ssid == ssid)
            .map(|(connection, _)| connection)
            .collect())
    }

    pub fn properties<const N: usize>(
        &self,
        uuids: &[&str],
        fields: [&str; N],
    ) -> Result<Vec<[String; N]>> {
        if uuids.is_empty() {
            return Ok(Vec::new());
        }
        let fields = fields.join(",");
        let mut args = vec!["-g", fields.as_str(), "connection", "show"];
        for uuid in uuids {
            args.push("uuid");
            args.push(uuid);
        }
        let output = self.nmcli(&args)?;
        let lines: Vec<&str> = output.lines().collect();
        if lines.len() != uuids.len() * (N + 1) - 1 {
            return Err(Error::NmcliUnexpectedOutput {
                command: "connection show".to_owned(),
            });
        }
        Ok(lines
            .chunks(N + 1)
            .map(|block| std::array::from_fn(|i| unescape(block[i])))
            .collect())
    }

    pub fn optional_properties<const N: usize>(
        &self,
        uuid: &str,
        fields: [&str; N],
    ) -> Result<Option<[String; N]>> {
        let fields = fields.join(",");
        let output = self.nmcli(&["-g", &fields, "connection", "show", "uuid", uuid])?;
        if output.is_empty() {
            return Ok(None);
        }
        let lines: Vec<&str> = output.lines().collect();
        let lines = <[&str; N]>::try_from(lines).map_err(|_| Error::NmcliUnexpectedOutput {
            command: "connection show".to_owned(),
        })?;
        Ok(Some(lines.map(unescape)))
    }

    pub fn add(&self, settings: &[(&str, String)]) -> Result<String> {
        let uuid = fs::read_to_string(UUID_SOURCE).map_err(Error::Uuid)?;
        let uuid = uuid.trim();
        let mut args = vec!["connection", "add", "connection.uuid", uuid];
        for (key, value) in settings {
            args.push(key);
            args.push(value);
        }
        self.nmcli(&args)?;
        Ok(uuid.to_owned())
    }

    pub fn modify(&self, uuid: &str, settings: &[(&str, String)]) -> Result<()> {
        let mut args = vec!["connection", "modify", "uuid", uuid];
        for (key, value) in settings {
            args.push(key);
            args.push(value);
        }
        self.nmcli(&args)?;
        Ok(())
    }

    pub fn delete(&self, uuid: &str) -> Result<()> {
        self.nmcli(&["connection", "delete", "uuid", uuid])?;
        Ok(())
    }

    pub fn activate(&self, uuid: &str) -> Result<()> {
        match self.nmcli(&[
            "--wait",
            ACTIVATION_TIMEOUT_SECS,
            "connection",
            "up",
            "uuid",
            uuid,
        ]) {
            Err(Error::NmcliFailed { message, .. }) if message.contains(SECRETS_REQUIRED) => {
                Err(Error::AuthenticationFailed)
            }
            result => result.map(|_| ()),
        }
    }

    pub fn active_wifi_uuids(&self) -> Result<Vec<String>> {
        let listing = self.nmcli(&["-t", "-f", "UUID,TYPE", "connection", "show", "--active"])?;
        Ok(listing
            .lines()
            .map(split_terse)
            .filter(|fields| fields.get(1).is_some_and(|kind| kind == "802-11-wireless"))
            .map(|mut fields| fields.swap_remove(0))
            .collect())
    }

    pub fn visible_ssids(&self) -> Result<Vec<String>> {
        let listing = self.nmcli(&[
            "-t", "-f", "SSID", "device", "wifi", "list", "--rescan", "auto",
        ])?;
        Ok(listing.lines().map(unescape).collect())
    }

    pub fn set_password(&self, uuid: &str, password: &str) -> Result<()> {
        let script = format!("set 802-1x.password\n{password}\nsave\nquit\n");
        let output = self
            .runner
            .run(&["connection", "edit", "uuid", uuid], Some(&script))?;
        if !output.success || output.stdout.contains("Error") || output.stderr.contains("Error") {
            return Err(Error::NmcliFailed {
                command: "connection edit".to_owned(),
                message: "failed to set the password".to_owned(),
            });
        }
        let stored = self.nmcli(&[
            "-s",
            "-g",
            "802-1x.password",
            "connection",
            "show",
            "uuid",
            uuid,
        ])?;
        if unescape(stored.strip_suffix('\n').unwrap_or(&stored)) != password {
            return Err(Error::PasswordMismatch);
        }
        Ok(())
    }

    fn nmcli(&self, args: &[&str]) -> Result<String> {
        let output = self.runner.run(args, None)?;
        if !output.success {
            return Err(Error::NmcliFailed {
                command: args
                    .iter()
                    .skip_while(|&&arg| !matches!(arg, "connection" | "device"))
                    .take(2)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" "),
                message: output
                    .stderr
                    .trim()
                    .trim_start_matches("Error: ")
                    .to_owned(),
            });
        }
        Ok(output.stdout)
    }
}

fn split_terse(line: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => fields
                .last_mut()
                .expect("fields is non-empty")
                .extend(chars.next()),
            ':' => fields.push(String::new()),
            _ => fields.last_mut().expect("fields is non-empty").push(c),
        }
    }
    fields
}

fn unescape(value: &str) -> String {
    let mut unescaped = String::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => unescaped.extend(chars.next()),
            _ => unescaped.push(c),
        }
    }
    unescaped
}

#[cfg(test)]
pub mod testing {
    use std::cell::RefCell;
    use std::collections::VecDeque;

    use super::{Output, Runner};
    use crate::error::Result;

    #[derive(Default)]
    pub struct FakeRunner {
        outputs: RefCell<VecDeque<Output>>,
        pub calls: RefCell<Vec<(Vec<String>, Option<String>)>>,
    }

    impl FakeRunner {
        pub fn respond(self, success: bool, stdout: &str) -> Self {
            self.outputs.borrow_mut().push_back(Output {
                success,
                stdout: stdout.to_owned(),
                stderr: if success {
                    String::new()
                } else {
                    "Error: something went wrong.\n".to_owned()
                },
            });
            self
        }

        pub fn fail_with(self, stderr: &str) -> Self {
            self.outputs.borrow_mut().push_back(Output {
                success: false,
                stdout: String::new(),
                stderr: stderr.to_owned(),
            });
            self
        }
    }

    impl Runner for &FakeRunner {
        fn run(&self, args: &[&str], stdin: Option<&str>) -> Result<Output> {
            self.calls.borrow_mut().push((
                args.iter().map(|&arg| arg.to_owned()).collect(),
                stdin.map(str::to_owned),
            ));
            Ok(self
                .outputs
                .borrow_mut()
                .pop_front()
                .expect("unexpected nmcli call"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::FakeRunner;
    use super::*;

    #[test]
    fn split_terse_handles_escaped_separators() {
        assert_eq!(
            split_terse(r"uuid:802-11-wireless:my\:wifi\\1"),
            ["uuid", "802-11-wireless", r"my:wifi\1"]
        );
        assert_eq!(split_terse("a::"), ["a", "", ""]);
    }

    #[test]
    fn find_by_ssid_matches_wifi_connections_by_ssid() {
        let runner = FakeRunner::default()
            .respond(
                true,
                "u1:802-11-wireless:eduroam\nu2:bridge:br0\nu3:802-11-wireless:eduroam 1\nu4:802-11-wireless:home\n",
            )
            .respond(true, "eduroam\n\neduroam\n\nhome\n");
        let nm = NetworkManager::new(&runner);

        let connections = nm.find_by_ssid("eduroam").unwrap();

        assert_eq!(
            connections,
            [
                Connection {
                    uuid: "u1".to_owned(),
                    name: "eduroam".to_owned()
                },
                Connection {
                    uuid: "u3".to_owned(),
                    name: "eduroam 1".to_owned()
                },
            ]
        );
        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(
            calls[1].0,
            [
                "-g",
                "802-11-wireless.ssid",
                "connection",
                "show",
                "uuid",
                "u1",
                "uuid",
                "u3",
                "uuid",
                "u4"
            ]
        );
    }

    #[test]
    fn properties_splits_blocks_and_keeps_empty_values() {
        let runner = FakeRunner::default().respond(
            true,
            "u1234567@wifi.u-tokyo.ac.jp\nno\n\n\nu7654321@wifi.u-tokyo.ac.jp\nyes\nu-tokyo.ac.jp\n",
        );
        let nm = NetworkManager::new(&runner);

        let values = nm
            .properties(
                &["u1", "u2"],
                [
                    "802-1x.identity",
                    "802-1x.system-ca-certs",
                    "802-1x.domain-suffix-match",
                ],
            )
            .unwrap();

        assert_eq!(
            values,
            [
                ["u1234567@wifi.u-tokyo.ac.jp", "no", ""],
                ["u7654321@wifi.u-tokyo.ac.jp", "yes", "u-tokyo.ac.jp"],
            ]
        );
    }

    #[test]
    fn properties_rejects_unexpected_line_count() {
        let runner = FakeRunner::default().respond(true, "eduroam\n\n");
        let nm = NetworkManager::new(&runner);

        assert!(matches!(
            nm.properties(&["u1", "u2"], ["802-11-wireless.ssid"]),
            Err(Error::NmcliUnexpectedOutput { .. })
        ));
    }

    #[test]
    fn properties_without_uuids_does_not_call_nmcli() {
        let runner = FakeRunner::default();
        let nm = NetworkManager::new(&runner);

        let values = nm.properties(&[], ["802-11-wireless.ssid"]).unwrap();

        assert!(values.is_empty());
        assert!(runner.calls.borrow().is_empty());
    }

    #[test]
    fn optional_properties_reads_fields_of_one_connection() {
        let runner = FakeRunner::default().respond(true, "u1234567@wifi.u-tokyo.ac.jp\n\n");
        let nm = NetworkManager::new(&runner);

        let values = nm
            .optional_properties("u1", ["802-1x.identity", "802-1x.ca-cert"])
            .unwrap();

        assert_eq!(
            values,
            Some(["u1234567@wifi.u-tokyo.ac.jp".to_owned(), String::new()])
        );
        assert_eq!(
            runner.calls.borrow()[0].0,
            [
                "-g",
                "802-1x.identity,802-1x.ca-cert",
                "connection",
                "show",
                "uuid",
                "u1"
            ]
        );
    }

    #[test]
    fn optional_properties_returns_none_without_setting() {
        let runner = FakeRunner::default().respond(true, "");
        let nm = NetworkManager::new(&runner);

        assert_eq!(
            nm.optional_properties("u1", ["802-1x.identity", "802-1x.ca-cert"])
                .unwrap(),
            None
        );
    }

    #[test]
    fn add_passes_settings_with_generated_uuid() {
        let runner = FakeRunner::default().respond(true, "");
        let nm = NetworkManager::new(&runner);

        let uuid = nm
            .add(&[("802-11-wireless.ssid", "eduroam".to_owned())])
            .unwrap();

        let calls = runner.calls.borrow();
        assert_eq!(
            calls[0].0,
            [
                "connection",
                "add",
                "connection.uuid",
                &uuid,
                "802-11-wireless.ssid",
                "eduroam"
            ]
        );
        assert_eq!(uuid.len(), 36);
    }

    #[test]
    fn set_password_writes_password_to_stdin_only() {
        let runner = FakeRunner::default()
            .respond(true, "Connection 'eduroam' successfully updated.\n")
            .respond(true, "secret pass\n");
        let nm = NetworkManager::new(&runner);

        nm.set_password("u1", "secret pass").unwrap();

        let calls = runner.calls.borrow();
        for (args, _) in calls.iter() {
            assert!(!args.iter().any(|arg| arg.contains("secret")));
        }
        let (args, stdin) = &calls[0];
        assert_eq!(args, &["connection", "edit", "uuid", "u1"]);
        assert_eq!(
            stdin.as_deref(),
            Some("set 802-1x.password\nsecret pass\nsave\nquit\n")
        );
        assert_eq!(
            calls[1].0,
            [
                "-s",
                "-g",
                "802-1x.password",
                "connection",
                "show",
                "uuid",
                "u1"
            ]
        );
    }

    #[test]
    fn set_password_keeps_surrounding_whitespace() {
        let runner = FakeRunner::default()
            .respond(true, "")
            .respond(true, " secret \n");
        let nm = NetworkManager::new(&runner);

        nm.set_password("u1", " secret ").unwrap();

        assert_eq!(
            runner.calls.borrow()[0].1.as_deref(),
            Some("set 802-1x.password\n secret \nsave\nquit\n")
        );
    }

    #[test]
    fn set_password_compares_unescaped_stored_password() {
        let runner = FakeRunner::default()
            .respond(true, "")
            .respond(true, "32%6o#+u\\:x\n");
        let nm = NetworkManager::new(&runner);

        nm.set_password("u1", "32%6o#+u:x").unwrap();
    }

    #[test]
    fn set_password_fails_on_stored_password_mismatch() {
        let runner = FakeRunner::default()
            .respond(true, "")
            .respond(true, "32%6o\n");
        let nm = NetworkManager::new(&runner);

        assert!(matches!(
            nm.set_password("u1", "32%6o#+u"),
            Err(Error::PasswordMismatch)
        ));
    }

    #[test]
    fn set_password_fails_on_editor_error() {
        let runner =
            FakeRunner::default().respond(true, "Error: failed to set 'password' property\n");
        let nm = NetworkManager::new(&runner);

        assert!(matches!(
            nm.set_password("u1", "secret"),
            Err(Error::NmcliFailed { .. })
        ));
    }

    #[test]
    fn nmcli_failure_reports_subcommand_and_stderr() {
        let runner = FakeRunner::default().respond(false, "");
        let nm = NetworkManager::new(&runner);

        let error = nm.delete("u1").unwrap_err();

        assert_eq!(
            error.to_string(),
            "`nmcli connection delete` failed: something went wrong."
        );
    }

    #[test]
    fn activate_waits_for_activation() {
        let runner = FakeRunner::default().respond(true, "");
        let nm = NetworkManager::new(&runner);

        nm.activate("u1").unwrap();

        assert_eq!(
            runner.calls.borrow()[0].0,
            ["--wait", "60", "connection", "up", "uuid", "u1"]
        );
    }

    #[test]
    fn activate_failure_reports_subcommand() {
        let runner = FakeRunner::default().respond(false, "");
        let nm = NetworkManager::new(&runner);

        let error = nm.activate("u1").unwrap_err();

        assert!(
            error
                .to_string()
                .starts_with("`nmcli connection up` failed:")
        );
    }

    #[test]
    fn activate_reports_authentication_failure() {
        let runner = FakeRunner::default().fail_with(
            "Error: Connection activation failed: Secrets were required, but not provided\n",
        );
        let nm = NetworkManager::new(&runner);

        assert!(matches!(
            nm.activate("u1"),
            Err(Error::AuthenticationFailed)
        ));
    }

    #[test]
    fn active_wifi_uuids_keeps_wifi_connections_only() {
        let runner =
            FakeRunner::default().respond(true, "u1:802-11-wireless\nu2:tun\nu3:802-11-wireless\n");
        let nm = NetworkManager::new(&runner);

        assert_eq!(nm.active_wifi_uuids().unwrap(), ["u1", "u3"]);
        assert_eq!(
            runner.calls.borrow()[0].0,
            ["-t", "-f", "UUID,TYPE", "connection", "show", "--active"]
        );
    }

    #[test]
    fn visible_ssids_unescapes_listing() {
        let runner = FakeRunner::default().respond(true, "eduroam\n\nmy\\:wifi\n");
        let nm = NetworkManager::new(&runner);

        assert_eq!(nm.visible_ssids().unwrap(), ["eduroam", "", "my:wifi"]);
        assert_eq!(
            runner.calls.borrow()[0].0,
            [
                "-t", "-f", "SSID", "device", "wifi", "list", "--rescan", "auto"
            ]
        );
    }
}
