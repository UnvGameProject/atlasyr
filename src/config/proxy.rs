use std::{
    env::{self, VarError},
    error::Error,
    fmt::{self, Display, Formatter},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

const TRUSTED_PROXY_NETWORKS_ENV: &str = "TRUSTED_PROXY_NETWORKS";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProxyConfig {
    trusted_networks: Vec<String>,
}

impl ProxyConfig {
    pub(crate) fn load() -> Result<Self, ProxyConfigError> {
        match env::var(TRUSTED_PROXY_NETWORKS_ENV) {
            Ok(value) => Self::parse(Some(&value)),

            Err(VarError::NotPresent) => Self::parse(None),

            Err(VarError::NotUnicode(_)) => Err(ProxyConfigError::NonUnicode),
        }
    }

    pub(crate) fn trusted_networks(&self) -> impl Iterator<Item = &str> {
        self.trusted_networks.iter().map(String::as_str)
    }

    fn parse(value: Option<&str>) -> Result<Self, ProxyConfigError> {
        let Some(value) = value else {
            return Ok(Self {
                trusted_networks: Vec::new(),
            });
        };

        if value.trim().is_empty() {
            return Err(ProxyConfigError::Empty);
        }

        let raw_networks = value.split(',').map(str::trim).collect::<Vec<_>>();

        let trusted_networks = raw_networks
            .into_iter()
            .map(normalize_network)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { trusted_networks })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProxyConfigError {
    NonUnicode,
    Empty,
    EmptyEntry,
    InvalidNetwork(String),
    CatchAllNetwork(String),
}

impl Display for ProxyConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonUnicode => write!(
                formatter,
                "{TRUSTED_PROXY_NETWORKS_ENV} must contain valid UTF-8",
            ),

            Self::Empty => write!(
                formatter,
                "{TRUSTED_PROXY_NETWORKS_ENV} must be unset or contain at least one trusted proxy network",
            ),

            Self::EmptyEntry => write!(
                formatter,
                "{TRUSTED_PROXY_NETWORKS_ENV} contains an empty network entry",
            ),

            Self::InvalidNetwork(network) => write!(
                formatter,
                "{TRUSTED_PROXY_NETWORKS_ENV} contains invalid IP/CIDR network {network:?}",
            ),

            Self::CatchAllNetwork(network) => write!(
                formatter,
                "{TRUSTED_PROXY_NETWORKS_ENV} must not trust a catch-all network: {network:?}",
            ),
        }
    }
}

impl Error for ProxyConfigError {}

fn normalize_network(value: &str) -> Result<String, ProxyConfigError> {
    if value.is_empty() {
        return Err(ProxyConfigError::EmptyEntry);
    }

    if let Ok(address) = value.parse::<IpAddr>() {
        return Ok(address.to_string());
    }

    let Some((address, prefix)) = value.split_once('/') else {
        return Err(ProxyConfigError::InvalidNetwork(value.to_owned()));
    };

    if address.is_empty() || prefix.is_empty() || prefix.contains('/') {
        return Err(ProxyConfigError::InvalidNetwork(value.to_owned()));
    }

    let address = address
        .parse::<IpAddr>()
        .map_err(|_| ProxyConfigError::InvalidNetwork(value.to_owned()))?;

    let prefix = prefix
        .parse::<u8>()
        .map_err(|_| ProxyConfigError::InvalidNetwork(value.to_owned()))?;

    match address {
        IpAddr::V4(address) => normalize_ipv4_network(address, prefix, value),

        IpAddr::V6(address) => normalize_ipv6_network(address, prefix, value),
    }
}

fn normalize_ipv4_network(
    address: Ipv4Addr,
    prefix: u8,
    original: &str,
) -> Result<String, ProxyConfigError> {
    if prefix > 32 {
        return Err(ProxyConfigError::InvalidNetwork(original.to_owned()));
    }

    if prefix == 0 {
        return Err(ProxyConfigError::CatchAllNetwork(original.to_owned()));
    }

    let shift = 32_u32 - u32::from(prefix);

    let mask = u32::MAX << shift;

    let network = Ipv4Addr::from(u32::from(address) & mask);

    Ok(format!("{network}/{prefix}"))
}

fn normalize_ipv6_network(
    address: Ipv6Addr,
    prefix: u8,
    original: &str,
) -> Result<String, ProxyConfigError> {
    if prefix > 128 {
        return Err(ProxyConfigError::InvalidNetwork(original.to_owned()));
    }

    if prefix == 0 {
        return Err(ProxyConfigError::CatchAllNetwork(original.to_owned()));
    }

    let shift = 128_u32 - u32::from(prefix);

    let mask = u128::MAX << shift;

    let network = Ipv6Addr::from(u128::from(address) & mask);

    Ok(format!("{network}/{prefix}"))
}

#[cfg(test)]
mod tests {
    use super::{ProxyConfig, ProxyConfigError};

    #[test]
    fn absent_configuration_trusts_no_proxy_networks() {
        let config = ProxyConfig::parse(None).expect("absent configuration should be valid");

        assert_eq!(
            config.trusted_networks().collect::<Vec<_>>(),
            Vec::<&str>::new(),
        );
    }

    #[test]
    fn parses_and_normalizes_explicit_networks() {
        let config = ProxyConfig::parse(Some(
            "127.0.0.1, 10.9.8.7/8, 2001:db8::1, fd12:3456::abcd/16",
        ))
        .expect("valid trusted proxy networks should parse");

        assert_eq!(
            config.trusted_networks().collect::<Vec<_>>(),
            vec!["127.0.0.1", "10.0.0.0/8", "2001:db8::1", "fd12::/16",],
        );
    }

    #[test]
    fn explicit_empty_configuration_fails_closed() {
        assert_eq!(
            ProxyConfig::parse(Some("   ")),
            Err(ProxyConfigError::Empty),
        );
    }

    #[test]
    fn empty_network_entry_fails_closed() {
        assert_eq!(
            ProxyConfig::parse(Some("10.0.0.0/8,,127.0.0.1",)),
            Err(ProxyConfigError::EmptyEntry),
        );
    }

    #[test]
    fn malformed_network_fails_closed() {
        assert_eq!(
            ProxyConfig::parse(Some("not-a-network")),
            Err(ProxyConfigError::InvalidNetwork("not-a-network".to_owned(),),),
        );

        assert_eq!(
            ProxyConfig::parse(Some("10.0.0.0/33")),
            Err(ProxyConfigError::InvalidNetwork("10.0.0.0/33".to_owned(),),),
        );

        assert_eq!(
            ProxyConfig::parse(Some("2001:db8::/129")),
            Err(ProxyConfigError::InvalidNetwork(
                "2001:db8::/129".to_owned(),
            ),),
        );
    }

    #[test]
    fn catch_all_networks_are_rejected() {
        assert_eq!(
            ProxyConfig::parse(Some("0.0.0.0/0")),
            Err(ProxyConfigError::CatchAllNetwork("0.0.0.0/0".to_owned(),),),
        );

        assert_eq!(
            ProxyConfig::parse(Some("::/0")),
            Err(ProxyConfigError::CatchAllNetwork("::/0".to_owned(),),),
        );
    }

    #[test]
    fn valid_proxy_network_lists_are_not_arbitrarily_capped() {
        let networks = (1..=65)
            .map(|index| format!("10.0.0.{index}"))
            .collect::<Vec<_>>();
        let value = networks.join(",");

        let config =
            ProxyConfig::parse(Some(&value)).expect("valid configured networks should parse");

        assert_eq!(config.trusted_networks().count(), 65);
    }

    #[test]
    fn malformed_network_in_long_list_still_fails_closed() {
        let mut networks = vec!["127.0.0.1"; 65];
        networks.push("not-a-network");

        assert_eq!(
            ProxyConfig::parse(Some(&networks.join(","))),
            Err(ProxyConfigError::InvalidNetwork("not-a-network".to_owned(),)),
        );
    }
}
