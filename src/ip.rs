use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
    time::Duration,
};

use anyhow::{Context, Result};
use reqwest::{Client, IntoUrl};

static PROVIDERS: [&str; 2] = ["https://ifconfig.me", "https://ifconfig.co"];

pub async fn get_ips(
    get_ipv4: bool,
    get_ipv6: bool,
) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>)> {
    let mut ipv4 = None;
    let mut ipv6 = None;

    let ipv4_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        .build()
        .expect("Failed to build IPv4 client");
    let ipv6_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .local_address(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
        .build()
        .expect("Failed to build IPv6 client");

    for provider in PROVIDERS {
        log::debug!("Trying provider {provider}");

        if get_ipv4 && ipv4.is_none() {
            match try_get_ip(&ipv4_client, provider).await {
                Ok(Some(address)) => ipv4 = Some(address),
                Err(error) => log::debug!("Failed to get IPv4 address from {provider}: {error:?}"),
                _ => (),
            }
        }
        if get_ipv6 && ipv6.is_none() {
            match try_get_ip(&ipv6_client, provider).await {
                Ok(Some(address)) => ipv6 = Some(address),
                Err(error) => log::debug!("Failed to get IPv6 address from {provider}: {error:?}"),
                _ => (),
            }
        }

        if (get_ipv4 == ipv4.is_some()) && (get_ipv6 == ipv6.is_some()) {
            break;
        }
    }

    Ok((ipv4, ipv6))
}

async fn try_get_ip<Addr: FromStr>(
    client: &Client,
    provider: impl IntoUrl,
) -> Result<Option<Addr>> {
    let addresses = client
        .get(provider)
        .send()
        .await
        .context("Failed to send GET request (get_ips)")?
        .text()
        .await
        .context("Failed to read GET body (get_ips)")?;
    for address in addresses.lines() {
        if let Ok(address) = address.parse::<Addr>() {
            return Ok(Some(address));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
    }

    #[tokio::test]
    async fn test_get_ips() {
        setup();
        let (ipv4, ipv6) = get_ips(true, true).await.unwrap();
        println!("ipv4: {:?}", ipv4.unwrap());
        println!("ipv6: {:?}", ipv6.unwrap());
    }
}
