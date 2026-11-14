use std::borrow::Cow;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

mod utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[allow(clippy::exhaustive_enums)]
pub enum ParseError {
    #[error("truncated address")]
    Truncated,
    #[error("{0}")]
    Invalid(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum SocksAddr {
    V4(SocketAddrV4),
    Domain(String, u16),
    V6(SocketAddrV6),
}

impl SocksAddr {
    pub fn parse_socks(buf: &[u8]) -> Result<(Self, usize), ParseError> {
        Self::parse(buf, true)
    }

    pub fn parse_xray(buf: &[u8]) -> Result<(Self, usize), ParseError> {
        Self::parse(buf, false)
    }

    pub fn host(&self) -> Cow<'_, str> {
        match self {
            Self::V4(addr) => Cow::Owned(addr.ip().to_string()),
            Self::Domain(domain, _) => Cow::Borrowed(domain),
            Self::V6(addr) => Cow::Owned(addr.ip().to_string()),
        }
    }

    pub const fn port(&self) -> u16 {
        match self {
            Self::V4(addr) => addr.port(),
            Self::Domain(_, port) => *port,
            Self::V6(addr) => addr.port(),
        }
    }

    pub fn is_routable(&self) -> bool {
        self.port() != 0
            && match self {
                Self::V4(_) | Self::V6(_) => true,
                Self::Domain(host, _) => {
                    host.strip_suffix('.').unwrap_or(host).len() <= 254
                        && host
                            .strip_suffix('.')
                            .unwrap_or(host)
                            .as_bytes()
                            .split(|byte| *byte == b'.')
                            .all(|label| {
                                label.len() <= 63
                                    && label.first().is_some_and(u8::is_ascii_alphanumeric)
                                    && label.last().is_some_and(u8::is_ascii_alphanumeric)
                                    && label
                                        .iter()
                                        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
                            })
                }
            }
    }

    fn parse(buf: &[u8], socks: bool) -> Result<(Self, usize), ParseError> {
        let mut remaining = buf;
        let mut port = if socks {
            0
        } else {
            u16::from_be_bytes(utils::take(&mut remaining)?)
        };
        let mut addr = match (socks, utils::take(&mut remaining)?) {
            (_, [1]) => Self::V4(SocketAddrV4::new(
                Ipv4Addr::from(utils::take::<4>(&mut remaining)?),
                port,
            )),
            (true, [3]) | (false, [2]) => {
                let [len] = utils::take(&mut remaining)?;
                Self::Domain(
                    std::str::from_utf8(
                        remaining
                            .split_off(..usize::from(len))
                            .ok_or(ParseError::Truncated)?,
                    )
                    .map_err(|_| ParseError::Invalid("invalid domain encoding"))?
                    .to_owned(),
                    port,
                )
            }
            (true, [4]) | (false, [3]) => Self::V6(SocketAddrV6::new(
                Ipv6Addr::from(utils::take::<16>(&mut remaining)?),
                port,
                0,
                0,
            )),
            _ => return Err(ParseError::Invalid("invalid address family")),
        };
        if socks {
            port = u16::from_be_bytes(utils::take(&mut remaining)?);
            match &mut addr {
                Self::V4(addr) => addr.set_port(port),
                Self::Domain(_, addr_port) => *addr_port = port,
                Self::V6(addr) => addr.set_port(port),
            }
        }
        Ok((addr, buf.len() - remaining.len()))
    }
}

impl fmt::Display for SocksAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V4(addr) => write!(f, "{}:{}", addr.ip(), addr.port()),
            Self::Domain(host, port) => write!(f, "{host}:{port}"),
            Self::V6(addr) => write!(f, "{}:{}", addr.ip(), addr.port()),
        }
    }
}
