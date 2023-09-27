use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::{bail, Context, Result};

static PROVIDERS: [&str; 3] = [
    "https://ifconfig.co",
    "https://ipinfo.io/ip",
    "https://ifconfig.me",
];

pub async fn get_ips() -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>)> {
    let mut ipv4 = None;
    let mut ipv6 = None;

    for provider in PROVIDERS {
        let addresses = reqwest::get(provider)
            .await
            .context("Failed to send GET request (get_ips)")?
            .text()
            .await
            .context("Failed to read GET body (get_ips)")?;
        for address in addresses.lines() {
            if let Ok(address) = address.parse::<Ipv4Addr>() {
                ipv4.get_or_insert(address);
            } else if let Ok(address) = address.parse::<Ipv6Addr>() {
                ipv6.get_or_insert(address);
            }
        }
    }

    if ipv4.is_none() && ipv6.is_none() {
        bail!("No provider returned an IP address")
    }

    Ok((ipv4, ipv6))
}
