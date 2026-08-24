use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use openreach_core::{
    CloudflareConfig, ProbeOptions, probe_reachability, run_tcp_forwarder, sync_cloudflare_dns,
};
use std::{net::SocketAddr, time::Duration};
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "openreach",
    version,
    about = "Publish local services through direct Internet reachability"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Probe this machine's currently observable Internet reachability.
    Probe {
        #[arg(long, default_value = "stun.l.google.com:19302")]
        stun_server: String,
        #[arg(long, default_value_t = 4)]
        timeout_seconds: u64,
    },

    /// Probe reachability and synchronize Cloudflare DNS once.
    SyncDns {
        #[arg(long, env = "CLOUDFLARE_API_TOKEN")]
        cloudflare_token: String,
        #[arg(long, env = "CLOUDFLARE_ZONE_ID")]
        cloudflare_zone_id: String,
        #[arg(long)]
        hostname: String,
        #[arg(long, default_value_t = 443)]
        public_tcp_port: u16,
        #[arg(long, default_value_t = 60)]
        ttl: u32,
        #[arg(long, default_value = "stun.l.google.com:19302")]
        stun_server: String,
    },

    /// Listen on a public/local interface and forward raw TCP to a local app.
    Forward {
        #[arg(long, default_value = "0.0.0.0:8443")]
        listen: SocketAddr,
        #[arg(long)]
        target: SocketAddr,
    },

    /// Production experiment: keep DNS fresh while forwarding traffic to a local app.
    Run {
        #[arg(long, env = "CLOUDFLARE_API_TOKEN")]
        cloudflare_token: String,
        #[arg(long, env = "CLOUDFLARE_ZONE_ID")]
        cloudflare_zone_id: String,
        #[arg(long)]
        hostname: String,
        #[arg(long)]
        target: SocketAddr,
        #[arg(long, default_value = "0.0.0.0:8443")]
        listen: SocketAddr,
        #[arg(long, default_value_t = 60)]
        ttl: u32,
        #[arg(long, default_value_t = 60)]
        refresh_seconds: u64,
        #[arg(long, default_value = "stun.l.google.com:19302")]
        stun_server: String,
    },
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn probe_options(stun_server: String) -> ProbeOptions {
    ProbeOptions {
        stun_server,
        timeout: Duration::from_secs(4),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Command::Probe {
            stun_server,
            timeout_seconds,
        } => {
            let report = probe_reachability(&ProbeOptions {
                stun_server,
                timeout: Duration::from_secs(timeout_seconds),
            })
            .await;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::SyncDns {
            cloudflare_token,
            cloudflare_zone_id,
            hostname,
            public_tcp_port,
            ttl,
            stun_server,
        } => {
            let report = probe_reachability(&probe_options(stun_server)).await;
            let result = sync_cloudflare_dns(
                &CloudflareConfig {
                    token: cloudflare_token,
                    zone_id: cloudflare_zone_id,
                    hostname,
                    ttl,
                },
                &report,
                public_tcp_port,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Command::Forward { listen, target } => {
            run_tcp_forwarder(listen, target).await?;
        }
        Command::Run {
            cloudflare_token,
            cloudflare_zone_id,
            hostname,
            target,
            listen,
            ttl,
            refresh_seconds,
            stun_server,
        } => {
            let config = CloudflareConfig {
                token: cloudflare_token,
                zone_id: cloudflare_zone_id,
                hostname,
                ttl,
            };
            let public_tcp_port = listen.port();
            let options = probe_options(stun_server);

            let dns_task = tokio::spawn(async move {
                loop {
                    let report = probe_reachability(&options).await;
                    match sync_cloudflare_dns(&config, &report, public_tcp_port).await {
                        Ok(result) => info!(hostname = %result.hostname, "DNS synchronized"),
                        Err(err) => tracing::warn!(error = %err, "DNS synchronization failed"),
                    }
                    sleep(Duration::from_secs(refresh_seconds.max(15))).await;
                }
            });

            let forward_task = tokio::spawn(async move {
                run_tcp_forwarder(listen, target)
                    .await
                    .context("TCP forwarder stopped")
            });

            tokio::select! {
                result = dns_task => result.context("DNS task panicked")?,
                result = forward_task => result.context("forward task panicked")??,
                _ = tokio::signal::ctrl_c() => info!("shutdown requested"),
            }
        }
    }

    Ok(())
}
