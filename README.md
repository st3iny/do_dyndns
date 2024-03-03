# do_dyndns

Dynamically update DNS records with the current IP addresses on DigitalOcean Domains.
Supply the DigitalOcean API token via the environment variable `DIGITALOCEAN_TOKEN`.

Get your API token from the
[DigitalOcean control panel](https://cloud.digitalocean.com/account/api/tokens).

## Build

The program can be built using cargo.

Debug build: `cargo build`

Production build: `cargo build --release`

Install to `~/.cargo/bin`: `cargo install --path=.`

Or just grab a binary from the releases page.

## Usage

```
Usage: do_dyndns [OPTIONS] <DOMAIN>

Arguments:
  <DOMAIN>
          The domain to update

Options:
  -n, --dry-run
          Don't actually change anything, just log changes

  -o, --once
          Run once and exit

  -4, --ipv4
          Create and update A record

  -6, --ipv6
          Create and update AAAA record

  -i, --sleep-interval <SLEEP_INTERVAL>
          Sleep interval in seconds

          [default: 300]

  -t, --ttl <TTL>
          TTL for the record

          [default: 60]

  -s, --subdomain <SUBDOMAIN>
          The subdomain to update or create [default: the domain itself]

          [default: @]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

Configure the log level using the `RUST_LOG` environment variable. Available log levels are
`trace`, `debug`, `info`, `warn`, and `error`. The default log level is `info`.

More information about the `RUST_LOG` environment variable is available
[here](https://docs.rs/env_logger/latest/env_logger/#enabling-logging).
