use std::{
  io,
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs},
};

use reqwest::Url;

use super::{NetworkError, NetworkErrorCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AddressPolicy {
  PublicOnly,
  #[cfg(test)]
  AllowLoopbackForTests,
}

pub(super) struct ResolvedDestination {
  pub host: String,
  pub addresses: Vec<SocketAddr>,
}

pub(super) fn resolve_destination(
  url: &Url,
  policy: AddressPolicy,
) -> Result<ResolvedDestination, NetworkError> {
  let host = url
    .host_str()
    .ok_or_else(|| NetworkError::at(NetworkErrorCode::InvalidRequest, "url_validation"))?;
  let port = url
    .port_or_known_default()
    .ok_or_else(|| NetworkError::at(NetworkErrorCode::InvalidRequest, "url_validation"))?;
  let addresses = (host, port)
    .to_socket_addrs()
    .map_err(map_dns_error)?
    .collect::<Vec<_>>();
  if addresses.is_empty() {
    return Err(NetworkError::at(NetworkErrorCode::Network, "dns"));
  }
  if addresses
    .iter()
    .any(|address| !address_allowed(address.ip(), policy))
  {
    return Err(NetworkError::at(
      NetworkErrorCode::PermissionDenied,
      "address_validation",
    ));
  }
  Ok(ResolvedDestination {
    host: host.to_string(),
    addresses,
  })
}

fn map_dns_error(_error: io::Error) -> NetworkError {
  NetworkError::at(NetworkErrorCode::Network, "dns")
}

fn address_allowed(address: IpAddr, _policy: AddressPolicy) -> bool {
  #[cfg(test)]
  if _policy == AddressPolicy::AllowLoopbackForTests && address.is_loopback() {
    return true;
  }
  match address {
    IpAddr::V4(address) => ipv4_is_public(address),
    IpAddr::V6(address) => ipv6_is_public(address),
  }
}

fn ipv4_is_public(address: Ipv4Addr) -> bool {
  let [a, b, c, d] = address.octets();
  !(a == 0
    || a == 10
    || a == 127
    || (a == 100 && (64..=127).contains(&b))
    || (a == 169 && b == 254)
    || (a == 172 && (16..=31).contains(&b))
    || (a == 192 && b == 0 && c == 0 && !matches!(d, 9 | 10))
    || (a == 192 && b == 0 && c == 2)
    || (a == 192 && b == 88 && c == 99)
    || (a == 192 && b == 168)
    || (a == 198 && (b == 18 || b == 19))
    || (a == 198 && b == 51 && c == 100)
    || (a == 203 && b == 0 && c == 113)
    || a >= 224)
}

fn ipv6_is_public(address: Ipv6Addr) -> bool {
  if let Some(embedded) = address.to_ipv4() {
    return ipv4_is_public(embedded);
  }
  let segments = address.segments();
  !(address.is_unspecified()
    || address.is_loopback()
    || address.is_multicast()
    || (segments[0] == 0x0064 && segments[1] == 0xff9b)
    || (segments[0] == 0x0100 && segments[1] == 0)
    || (segments[0] & 0xfe00) == 0xfc00
    || (segments[0] & 0xffc0) == 0xfe80
    || (segments[0] & 0xffc0) == 0xfec0
    || (segments[0] == 0x2001 && segments[1] == 0x0002)
    || (segments[0] == 0x2001 && (segments[1] & 0xfff0) == 0x0010)
    || (segments[0] == 0x2001 && (segments[1] & 0xfff0) == 0x0020)
    || (segments[0] == 0x2001 && segments[1] == 0x0db8)
    || segments[0] == 0x2002)
}

pub(super) fn is_forbidden_request_header(name: &str) -> bool {
  matches!(
    name,
    "host"
      | "content-length"
      | "connection"
      | "transfer-encoding"
      | "te"
      | "trailer"
      | "upgrade"
      | "via"
      | "forwarded"
      | "accept-encoding"
  ) || name.starts_with("proxy-")
    || name.starts_with("x-forwarded-")
}

pub(super) fn is_safe_response_header(name: &str) -> bool {
  matches!(
    name,
    "content-type"
      | "content-length"
      | "content-encoding"
      | "cache-control"
      | "expires"
      | "etag"
      | "last-modified"
      | "retry-after"
      | "ratelimit-limit"
      | "ratelimit-remaining"
      | "ratelimit-reset"
      | "x-ratelimit-limit"
      | "x-ratelimit-remaining"
      | "x-ratelimit-reset"
  )
}

pub(super) fn redacted_url(url: &Url) -> String {
  let mut redacted = url.clone();
  redacted.set_query(None);
  redacted.set_fragment(None);
  redacted.to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn public_address_check_rejects_special_ipv4_ranges() {
    for address in [
      Ipv4Addr::LOCALHOST,
      Ipv4Addr::new(10, 0, 0, 1),
      Ipv4Addr::new(100, 64, 0, 1),
      Ipv4Addr::new(169, 254, 1, 1),
      Ipv4Addr::new(172, 16, 0, 1),
      Ipv4Addr::new(192, 168, 0, 1),
      Ipv4Addr::new(198, 51, 100, 1),
      Ipv4Addr::new(224, 0, 0, 1),
    ] {
      assert!(!ipv4_is_public(address), "{address} must be denied");
    }
    assert!(ipv4_is_public(Ipv4Addr::new(8, 8, 8, 8)));
  }

  #[test]
  fn public_address_check_rejects_special_ipv6_ranges() {
    for address in [
      Ipv6Addr::LOCALHOST,
      "fc00::1".parse().unwrap(),
      "fe80::1".parse().unwrap(),
      "64:ff9b::c0a8:1".parse().unwrap(),
      "100::1".parse().unwrap(),
      "2001:2::1".parse().unwrap(),
      "2001:10::1".parse().unwrap(),
      "2001:db8::1".parse().unwrap(),
      "2002:c0a8:1::1".parse().unwrap(),
      "::192.168.0.1".parse().unwrap(),
      "::ffff:127.0.0.1".parse().unwrap(),
    ] {
      assert!(!ipv6_is_public(address), "{address} must be denied");
    }
    assert!(ipv6_is_public("2606:4700:4700::1111".parse().unwrap()));
  }

  #[test]
  fn request_header_filter_rejects_transport_control_headers() {
    for name in [
      "host",
      "content-length",
      "connection",
      "proxy-authorization",
      "x-forwarded-for",
      "accept-encoding",
    ] {
      assert!(is_forbidden_request_header(name));
    }
    assert!(!is_forbidden_request_header("authorization"));
    assert!(!is_forbidden_request_header("x-game-token"));
  }
}
