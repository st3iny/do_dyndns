use std::{
    net::{Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use anyhow::{bail, Context, Result};
use api::ApiClient;
use clap::Parser;
use ip::get_ips;
use tokio::time::sleep;

mod api;
mod ip;

#[derive(Parser)]
#[command(
    version,
    long_about = "Update DNS record with the current IP addresses on DigitalOcean Domains. Supply the DigitalOcean API token via the environment variable DIGITALOCEAN_TOKEN."
)]
struct Args {
    /// Don't actually change anything, just log changes
    #[clap(short = 'n', long)]
    dry_run: bool,

    /// Run once and exit
    #[clap(short = 'o', long)]
    once: bool,

    /// Create and update A record
    #[clap(short = '4', long)]
    ipv4: bool,

    /// Create and update AAAA record
    #[clap(short = '6', long)]
    ipv6: bool,

    /// Sleep interval in seconds
    #[clap(short = 'i', long, default_value_t = 300)]
    sleep_interval: u64,

    /// TTL for the record
    #[clap(short = 't', long, default_value_t = 60)]
    ttl: u32,

    /// The subdomain to update or create [default: the domain itself]
    #[clap(short = 's', long, default_value = "@")]
    subdomain: String,

    /// The domain to update
    domain: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", format!("{}=info", env!("CARGO_PKG_NAME")));
    }
    env_logger::init();

    let args = Args::parse();
    if !args.ipv4 && !args.ipv6 {
        bail!("At least one of -4 or -6 must be specified");
    }
    if args.ttl == 0 {
        bail!("TTL must be greater than 0");
    }
    if args.sleep_interval == 0 {
        bail!("Sleep interval must be greater than 0");
    }

    let token = get_token()?;
    let client = ApiClient::new(&token);
    let sleep_interval = Duration::from_secs(args.sleep_interval);
    let mut last_ipv4 = None;
    let mut last_ipv6 = None;
    loop {
        if let Err(error) = dyndns(&args, &client, &mut last_ipv4, &mut last_ipv6).await {
            log::error!("{error:?}");
        }

        if args.once {
            break;
        } else {
            sleep(sleep_interval).await;
        }
    }

    Ok(())
}

async fn dyndns(
    args: &Args,
    client: &ApiClient,
    last_ipv4: &mut Option<Ipv4Addr>,
    last_ipv6: &mut Option<Ipv6Addr>,
) -> Result<()> {
    let (ipv4, ipv6) = get_ips(args.ipv4, args.ipv6)
        .await
        .context("Failed to get IP addresses")?;
    if let Some(ipv4) = &ipv4 {
        log::debug!("Current IPv4 address: {ipv4}");
    }
    if let Some(ipv6) = &ipv6 {
        log::debug!("Current IPv6 address: {ipv6}");
    }

    if args.ipv4 && ipv4.is_none() {
        bail!("No IPv4 address found");
    }
    if args.ipv6 && ipv6.is_none() {
        bail!("No IPv6 address found");
    }

    if args.ipv4 && &ipv4 != last_ipv4 {
        if let Some(ipv4) = &ipv4 {
            log::info!("New IPv4 address: {ipv4}");
            handle_a_record(args, ipv4, client)
                .await
                .context("Failed to update or create A record")?;
            *last_ipv4 = Some(*ipv4);
        } else {
            log::warn!("No IPv4 address found");
        }
    }

    if args.ipv6 && &ipv6 != last_ipv6 {
        if let Some(ipv6) = &ipv6 {
            log::info!("New IPv6 address: {ipv6}");
            handle_aaaa_record(args, ipv6, client)
                .await
                .context("Failed to update or create AAAA record")?;
            *last_ipv6 = Some(*ipv6);
        } else {
            log::warn!("No IPv6 address found");
        }
    }

    Ok(())
}

fn get_token() -> Result<String> {
    std::env::var("DIGITALOCEAN_TOKEN").context("DIGITALOCEAN_TOKEN not set or invalid UTF-8")
}

async fn handle_a_record(args: &Args, addr: &Ipv4Addr, client: &ApiClient) -> Result<()> {
    handle_record(args, "A", &addr.to_string(), client).await
}

async fn handle_aaaa_record(args: &Args, addr: &Ipv6Addr, client: &ApiClient) -> Result<()> {
    handle_record(args, "AAAA", &addr.to_string(), client).await
}

async fn handle_record(args: &Args, kind: &str, addr: &str, client: &ApiClient) -> Result<()> {
    let name_filter = match args.subdomain.as_str() {
        "@" => Some(args.domain.clone()),
        name => Some(format!("{name}.{}", args.domain)),
    };
    let records = client
        .get_records(&args.domain, Some(200), Some(kind), name_filter.as_deref())
        .await
        .with_context(|| format!("Failed to get {kind} records"))?
        .into_iter()
        .filter(|record| record.name == args.subdomain)
        .collect::<Vec<_>>();

    let name = &args.subdomain;
    let data = addr.to_string();
    let ttl = args.ttl;
    match records.len() {
        0 => create_record(client, args, name, kind, &data, ttl)
            .await
            .with_context(|| format!("Failed to create {kind} record"))?,
        1 => {
            let record = records.first().unwrap();
            if record.data == data {
                log::info!("{kind} record is up to date");
                return Ok(());
            }

            update_record(client, args, record.id, name, kind, &data, ttl)
                .await
                .with_context(|| format!("Failed to update {kind} record"))?;
        }
        _ => {
            bail!("More than one {kind} record found");
        }
    }

    Ok(())
}

async fn create_record(
    client: &ApiClient,
    args: &Args,
    name: &str,
    kind: &str,
    data: &str,
    ttl: u32,
) -> Result<()> {
    log::info!("Creating new {kind} record");
    if args.dry_run {
        log::info!(
            "Would create record: {{ name: {name:?}, type: {kind:?}, data: {data:?}, ttl: {ttl} }}"
        );
    } else {
        client
            .create_record(&args.domain, name, kind, data, ttl)
            .await?;
    }

    Ok(())
}

async fn update_record(
    client: &ApiClient,
    args: &Args,
    id: i64,
    name: &str,
    kind: &str,
    data: &str,
    ttl: u32,
) -> Result<()> {
    log::info!("Updating existing {kind} record");
    if args.dry_run {
        log::info!(
            "Would update record: {{ name: {name:?}, type: {kind:?}, data: {data:?}, ttl: {ttl} }}"
        );
    } else {
        client
            .update_record(&args.domain, id, name, kind, data, ttl)
            .await?;
    }

    Ok(())
}
