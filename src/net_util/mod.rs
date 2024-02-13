use anyhow::{Error, Result};
use log::debug;
use std::process::Command;
use std::{net::IpAddr, str::FromStr};
/*
https://man7.org/linux/man-pages/man8/ip-neighbour.8.html
   PERMANENT
    the neighbour entry is valid forever and can
    be only be removed administratively.

   NOARP
    the neighbour entry is valid. No attempts to
    validate this entry will be made but it can
    be removed when its lifetime expires.

   REACHABLE
    the neighbour entry is valid until the
    reachability timeout expires.

   STALE
    the neighbour entry is valid but suspicious.
    This option to ip neigh does not change the
    neighbour state if it was valid and the
    address is not changed by this command.

   NONE
    this is a pseudo state used when initially
    creating a neighbour entry or after trying
    to remove it before it becomes free to do
    so.

   INCOMPLETE
    the neighbour entry has not (yet) been
    validated/resolved.

   DELAY
    neighbor entry validation is currently
    delayed.

   PROBE
    neighbor is being probed.

   FAILED
      max number of probes exceeded without
      success, neighbor validation has ultimately
      failed.
*/
#[derive(Debug)]
pub enum NudState {
  UNKNOWN,
  PERMANENT,
  NOARP,
  REACHABLE,
  STALE,
  NONE,
  INCOMPLETE,
  DELAY,
  PROBE,
  FAILED,
}

pub fn parse_nud_from_str(nud_state_str: &str) -> NudState {
  match nud_state_str.to_uppercase().as_str() {
    "PERMANENT" => NudState::PERMANENT,
    "NOARP" => NudState::NOARP,
    "REACHABLE" => NudState::REACHABLE,
    "STALE" => NudState::STALE,
    "NONE" => NudState::NONE,
    "INCOMPLETE" => NudState::INCOMPLETE,
    "DELAY" => NudState::DELAY,
    "PROBE" => NudState::PROBE,
    "FAILED" => NudState::FAILED,
    _ => NudState::UNKNOWN,
  }
}

/*
  192.168.0.33 dev br-lan lladdr dc:a6:32:57:46:d6 ref 1 used 0/0/0 probes 1 REACHABLE
  192.168.0.5 dev br-lan lladdr dc:a6:32:a3:48:b1 ref 1 used 0/0/0 probes 1 REACHABLE
  192.168.0.2 dev br-lan  used 0/0/0 probes 6 FAILED
  192.168.0.147 dev br-lan lladdr 24:4b:fe:06:f8:3c ref 1 used 0/0/0 probes 1 REACHABLE
  192.168.0.200 dev br-lan lladdr 0a:99:ad:f6:ce:e6 used 0/0/0 probes 1 STALE
  172.119.56.1 dev eth1 lladdr 00:01:5c:68:3c:46 ref 1 used 0/0/0 probes 1 REACHABLE
  192.168.0.100 dev br-lan lladdr 1a:42:85:a2:22:fb ref 1 used 0/0/0 probes 1 REACHABLE
  192.168.0.11 dev br-lan lladdr 54:af:97:06:5d:7c ref 1 used 0/0/0 probes 1 REACHABLE
  192.168.0.8 dev br-lan lladdr 88:66:5a:49:16:b3 used 0/0/0 probes 1 STALE
  fd35:e227:2f15::169 dev br-lan lladdr 24:4b:fe:06:f8:3c used 0/0/0 probes 1 STALE
  fe80::e132:56de:1eac:d560 dev br-lan lladdr 24:4b:fe:06:f8:3c used 0/0/0 probes 1 STALE
  fe80::1866:4ccf:140e:95b0 dev br-lan lladdr 1a:42:85:a2:22:fb used 0/0/0 probes 4 STALE

  So we're parsing:
  <ipv(4|6) address> <dev> <iface> <lladdr> <mac> .* <nud state>
*/
#[derive(Debug)]
pub struct ArpTable {
  pub ip: IpAddr,
  pub iface: String,
  pub mac_addr: String,
  pub nud_state: NudState,
}

impl ArpTable {
  ///
  /// Parses a string into an IpNeighbor instance, given the string matches
  /// a line result from:
  /// https://man7.org/linux/man-pages/man8/ip-neighbour.8.html.
  ///
  /// Args:
  ///  - s: Row result as a string.
  ///
  /// Returns:
  ///  Result reflecting a successful parse.
  ///
  pub fn parse_from_string(s: &str) -> Result<Self> {
    // <ipv(4|6) address> <dev> <iface> <lladdr> <mac> .* <nud state>
    let sliced_str: Vec<&str> = s.split(' ').collect();
    debug!("Sliced string -> {:?}", sliced_str);

    // Expect to have at least 6 elements.
    if sliced_str.len() < 6 {
      return Err(Error::msg(format!("Unexpected string -> {}", s)));
    }

    // Attempt to parse the ip.
    let ip_addr_str = sliced_str.first().unwrap();
    let ip_addr = IpAddr::from_str(&ip_addr_str)
      .map_err(|e| Error::msg(format!("Failed to parse {}: {:?}", ip_addr_str, e)))?;
    debug!("Parsed ip address -> {:?}", ip_addr);

    // Extract the device name.
    let dev_str = sliced_str.get(1).unwrap().to_lowercase();
    if dev_str != "dev" {
      return Err(Error::msg(format!(
        "No device name found, expected 'dev' but got '{}'",
        dev_str,
      )));
    }
    let dev_name = sliced_str.get(2).unwrap().to_string();
    debug!("Extracted device name -> {:?}", dev_name);

    // Extract the device's mac address.
    let link_layer_addr_str = sliced_str.get(3).unwrap().to_lowercase();
    if link_layer_addr_str != "lladdr" {
      return Err(Error::msg(format!(
        "No link layer address found, expected 'lladdr' but got '{}'",
        link_layer_addr_str,
      )));
    }

    let mac_address = sliced_str.get(4).unwrap().to_lowercase();
    debug!("Extracted device mac address -> {:?}", mac_address);

    // Attempt to parse the nud state.
    let nud_state = parse_nud_from_str(sliced_str.last().unwrap());
    debug!("Parsed NUD State -> {:?}", nud_state);

    Ok(ArpTable {
      ip: ip_addr,
      iface: dev_name,
      mac_addr: mac_address,
      nud_state: nud_state,
    })
  }
}

/// Generates a parsed array of ArpTable results from the host.
pub fn get_ip_neighbors() -> Result<Vec<ArpTable>> {
  let ip_neigh_cmd = Command::new("ip").arg("neigh").output();

  match ip_neigh_cmd {
    Ok(output) => {
      if !output.status.success() {
        return Err(Error::msg(format!(
          "Command failed {:?}",
          output.stderr.to_ascii_lowercase()
        )));
      }

      let stdout: String =
        String::from_utf8(output.stdout).expect("Failed to convert output to string:");

      let stdout_filtered: Vec<&str> = stdout
        .split('\n')
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

      stdout_filtered
        .iter()
        .map(|v| ArpTable::parse_from_string(v))
        .collect()
    }
    Err(err) => Err(Error::msg(format!(
      "Failed to execute 'ip' command: {}",
      err.to_string()
    ))),
  }
}
