pub const DOMAIN_SUFFIX: &str = "u-tokyo.ac.jp";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    UTokyo,
    Eduroam,
}

impl Network {
    pub const ALL: [Self; 2] = [Self::UTokyo, Self::Eduroam];

    pub const fn ssid(self) -> &'static str {
        match self {
            Self::UTokyo => "0000UTokyo",
            Self::Eduroam => "eduroam",
        }
    }

    pub fn connection_settings(self, identity: &str) -> Vec<(&'static str, String)> {
        vec![
            ("connection.type", "802-11-wireless".to_owned()),
            ("connection.id", self.ssid().to_owned()),
            ("802-11-wireless.ssid", self.ssid().to_owned()),
            ("802-11-wireless-security.key-mgmt", "wpa-eap".to_owned()),
            ("802-1x.eap", "peap".to_owned()),
            ("802-1x.phase2-auth", "mschapv2".to_owned()),
            ("802-1x.identity", identity.to_owned()),
            ("802-1x.password-flags", "none".to_owned()),
            ("802-1x.system-ca-certs", "yes".to_owned()),
            ("802-1x.domain-suffix-match", DOMAIN_SUFFIX.to_owned()),
            ("connection.autoconnect", "no".to_owned()),
            ("connection.auth-retries", "1".to_owned()),
        ]
    }
}

pub fn verified_settings() -> Vec<(&'static str, String)> {
    vec![
        ("connection.autoconnect", "yes".to_owned()),
        ("connection.auth-retries", "-1".to_owned()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setting<'a>(settings: &'a [(&str, String)], key: &str) -> Option<&'a str> {
        settings
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn connection_settings_verify_server_certificate() {
        for network in Network::ALL {
            let settings = network.connection_settings("u1234567@wifi.u-tokyo.ac.jp");
            assert_eq!(setting(&settings, "802-1x.system-ca-certs"), Some("yes"));
            assert_eq!(
                setting(&settings, "802-1x.domain-suffix-match"),
                Some("u-tokyo.ac.jp")
            );
        }
    }

    #[test]
    fn connection_settings_use_ssid_and_identity() {
        let settings = Network::Eduroam.connection_settings("u1234567@wifi.u-tokyo.ac.jp");
        assert_eq!(setting(&settings, "connection.id"), Some("eduroam"));
        assert_eq!(setting(&settings, "802-11-wireless.ssid"), Some("eduroam"));
        assert_eq!(
            setting(&settings, "802-1x.identity"),
            Some("u1234567@wifi.u-tokyo.ac.jp")
        );
    }

    #[test]
    fn connection_settings_disable_autoconnect_and_retries() {
        let settings = Network::UTokyo.connection_settings("u1234567@wifi.u-tokyo.ac.jp");
        assert_eq!(setting(&settings, "connection.autoconnect"), Some("no"));
        assert_eq!(setting(&settings, "connection.auth-retries"), Some("1"));
    }

    #[test]
    fn verified_settings_enable_autoconnect_and_default_retries() {
        let settings = verified_settings();
        assert_eq!(setting(&settings, "connection.autoconnect"), Some("yes"));
        assert_eq!(setting(&settings, "connection.auth-retries"), Some("-1"));
    }
}
