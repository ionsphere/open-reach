use anyhow::{anyhow, bail, Context, Result};
use rand::RngCore;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use tokio::{
    io,
    net::{lookup_host, TcpListener, TcpStream, UdpSocket},
    time::timeout,
};
use tracing::{info, warn};

const STUN_MAGIC_COOKIE: u32 = 0x2112_A442;
const STUN_BINDING_REQUEST: u16 = 0x0001;
const STUN_BINDING_SUCCESS: u16 = 0x0101;
const ATTR_MAPPED_ADDRESS: u16 = 0x0001;
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;

#[derive(Debug, Clone, Serialize)]
pub struct ReachabilityReport {
    pub public_ipv4: Option<Ipv4Addr>,
    pub public_ipv6: Option<Ipv6Addr>,
    pub stun_mapped_addr: Option<SocketAddr>,
    pub stun_server: String,
}

#[derive(Debug, Clone)]
pub struct ProbeOptions {
    pub stun_server: String,
    pub timeout: Duration,
}

impl Default for ProbeOptions {
    fn default() -> Self {
        Self {
            stun_server: "stun.l.google.com:19302".to_owned(),
            timeout: Duration::from_secs(4),
        }
    }
}

pub async fn probe_reachability(options: &ProbeOptions) -> ReachabilityReport {
    let public_ipv6 = discover_outbound_ipv6().await;
    let stun_mapped_addr = match stun_binding(&options.stun_server, options.timeout).await {
        Ok(addr) => Some(addr),
        Err(err) => {
            warn!(error = %err, "STUN probe failed");
            None
        }
    };

    let public_ipv4 = stun_mapped_addr.and_then(|addr| match addr.ip() {
        IpAddr::V4(ip) => Some(ip),
        IpAddr::V6(_) => None,
    });

    ReachabilityReport {
        public_ipv4,
        public_ipv6,
        stun_mapped_addr,
        stun_server: options.stun_server.clone(),
    }
}

async fn discover_outbound_ipv6() -> Option<Ipv6Addr> {
    let socket = UdpSocket::bind("[::]:0").await.ok()?;
    // connect() on UDP does not send packets, but asks the OS to choose the
    // interface/source address it would use for this globally routed target.
    socket.connect("[2606:4700:4700::1111]:53").await.ok()?;
    let local = socket.local_addr().ok()?;
    match local.ip() {
        IpAddr::V6(ip) if is_globalish_ipv6(ip) => Some(ip),
        _ => None,
    }
}

fn is_globalish_ipv6(ip: Ipv6Addr) -> bool {
    let octets = ip.octets();
    if ip.is_unspecified() || ip.is_loopback() || ip.is_multicast() {
        return false;
    }
    // fc00::/7 unique-local
    if octets[0] & 0xfe == 0xfc {
        return false;
    }
    // fe80::/10 link-local
    if octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80 {
        return false;
    }
    true
}

pub async fn stun_binding(server: &str, duration: Duration) -> Result<SocketAddr> {
    let mut addrs = lookup_host(server)
        .await
        .with_context(|| format!("resolving STUN server {server}"))?;
    let remote = addrs
        .next()
        .ok_or_else(|| anyhow!("STUN server {server} resolved to no addresses"))?;

    let bind = if remote.is_ipv4() { "0.0.0.0:0" } else { "[::]:0" };
    let socket = UdpSocket::bind(bind).await?;
    socket.connect(remote).await?;

    let mut txid = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut txid);

    let mut request = [0u8; 20];
    request[0..2].copy_from_slice(&STUN_BINDING_REQUEST.to_be_bytes());
    request[2..4].copy_from_slice(&0u16.to_be_bytes());
    request[4..8].copy_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
    request[8..20].copy_from_slice(&txid);

    socket.send(&request).await?;

    let mut response = [0u8; 2048];
    let n = timeout(duration, socket.recv(&mut response))
        .await
        .context("STUN response timed out")??;

    parse_stun_response(&response[..n], txid)
}

fn parse_stun_response(packet: &[u8], txid: [u8; 12]) -> Result<SocketAddr> {
    if packet.len() < 20 {
        bail!("short STUN response");
    }
    let msg_type = u16::from_be_bytes([packet[0], packet[1]]);
    if msg_type != STUN_BINDING_SUCCESS {
        bail!("unexpected STUN message type 0x{msg_type:04x}");
    }
    let len = u16::from_be_bytes([packet[2], packet[3]]) as usize;
    if packet.len() < 20 + len {
        bail!("truncated STUN response");
    }
    let cookie = u32::from_be_bytes(packet[4..8].try_into().unwrap());
    if cookie != STUN_MAGIC_COOKIE || packet[8..20] != txid {
        bail!("STUN transaction mismatch");
    }

    let mut offset = 20;
    while offset + 4 <= 20 + len {
        let kind = u16::from_be_bytes([packet[offset], packet[offset + 1]]);
        let attr_len = u16::from_be_bytes([packet[offset + 2], packet[offset + 3]]) as usize;
        let value_start = offset + 4;
        let value_end = value_start + attr_len;
        if value_end > packet.len() {
            bail!("truncated STUN attribute");
        }

        let value = &packet[value_start..value_end];
        if kind == ATTR_XOR_MAPPED_ADDRESS {
            return parse_address_attr(value, true, txid);
        }
        if kind == ATTR_MAPPED_ADDRESS {
            return parse_address_attr(value, false, txid);
        }

        offset = value_end + ((4 - (attr_len % 4)) % 4);
    }

    bail!("STUN response contained no mapped address")
}

fn parse_address_attr(value: &[u8], xor: bool, txid: [u8; 12]) -> Result<SocketAddr> {
    if value.len() < 4 {
        bail!("short STUN mapped-address attribute");
    }
    let family = value[1];
    let mut port = u16::from_be_bytes([value[2], value[3]]);
    if xor {
        port ^= (STUN_MAGIC_COOKIE >> 16) as u16;
    }

    match family {
        0x01 => {
            if value.len() < 8 {
                bail!("short STUN IPv4 address");
            }
            let mut octets: [u8; 4] = value[4..8].try_into().unwrap();
            if xor {
                let cookie = STUN_MAGIC_COOKIE.to_be_bytes();
                for (byte, key) in octets.iter_mut().zip(cookie) {
                    *byte ^= key;
                }
            }
            Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(octets)), port))
        }
        0x02 => {
            if value.len() < 20 {
                bail!("short STUN IPv6 address");
            }
            let mut octets: [u8; 16] = value[4..20].try_into().unwrap();
            if xor {
                let mut key = [0u8; 16];
                key[0..4].copy_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
                key[4..16].copy_from_slice(&txid);
                for (byte, key) in octets.iter_mut().zip(key) {
                    *byte ^= key;
                }
            }
            Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(octets)), port))
        }
        other => bail!("unknown STUN address family {other}"),
    }
}

#[derive(Debug, Clone)]
pub struct CloudflareConfig {
    pub token: String,
    pub zone_id: String,
    pub hostname: String,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsSyncResult {
    pub hostname: String,
    pub a: Option<Ipv4Addr>,
    pub aaaa: Option<Ipv6Addr>,
    pub metadata_name: String,
}

pub async fn sync_cloudflare_dns(
    config: &CloudflareConfig,
    report: &ReachabilityReport,
    public_tcp_port: u16,
) -> Result<DnsSyncResult> {
    let client = CloudflareClient::new(config.token.clone());

    if let Some(ip) = report.public_ipv4 {
        client
            .upsert_record(&config.zone_id, "A", &config.hostname, &ip.to_string(), config.ttl)
            .await?;
    }
    if let Some(ip) = report.public_ipv6 {
        client
            .upsert_record(&config.zone_id, "AAAA", &config.hostname, &ip.to_string(), config.ttl)
            .await?;
    }

    let metadata_name = format!("_openreach.{}", config.hostname);
    let stun = report
        .stun_mapped_addr
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "none".to_owned());
    let metadata = format!(
        "v=or1; tcp={public_tcp_port}; stun={stun}; ipv6={}",
        report
            .public_ipv6
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "none".to_owned())
    );
    client
        .upsert_record(&config.zone_id, "TXT", &metadata_name, &metadata, config.ttl)
        .await?;

    Ok(DnsSyncResult {
        hostname: config.hostname.clone(),
        a: report.public_ipv4,
        aaaa: report.public_ipv6,
        metadata_name,
    })
}

#[derive(Debug, Clone)]
struct CloudflareClient {
    http: Client,
    token: String,
}

#[derive(Debug, Deserialize)]
struct CfEnvelope<T> {
    success: bool,
    result: T,
    #[serde(default)]
    errors: Vec<CfMessage>,
}

#[derive(Debug, Deserialize)]
struct CfMessage {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Deserialize)]
struct CfRecord {
    id: String,
}

impl CloudflareClient {
    fn new(token: String) -> Self {
        Self {
            http: Client::new(),
            token,
        }
    }

    async fn upsert_record(
        &self,
        zone_id: &str,
        record_type: &str,
        name: &str,
        content: &str,
        ttl: u32,
    ) -> Result<()> {
        let base = format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records");
        let listed: CfEnvelope<Vec<CfRecord>> = self
            .http
            .get(&base)
            .bearer_auth(&self.token)
            .query(&[("type", record_type), ("name", name)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        ensure_cf_success(&listed.success, &listed.errors)?;

        let body = serde_json::json!({
            "type": record_type,
            "name": name,
            "content": content,
            "ttl": ttl,
            "proxied": false
        });

        let response = if let Some(existing) = listed.result.first() {
            self.http
                .patch(format!("{base}/{}", existing.id))
                .bearer_auth(&self.token)
                .json(&body)
                .send()
                .await?
        } else {
            self.http
                .post(&base)
                .bearer_auth(&self.token)
                .json(&body)
                .send()
                .await?
        };

        let updated: CfEnvelope<serde_json::Value> = response.error_for_status()?.json().await?;
        ensure_cf_success(&updated.success, &updated.errors)?;
        info!(record_type, name, content, "Cloudflare DNS record synchronized");
        Ok(())
    }
}

fn ensure_cf_success(success: &bool, errors: &[CfMessage]) -> Result<()> {
    if *success {
        return Ok(());
    }
    let detail = errors
        .iter()
        .map(|e| format!("{}: {}", e.code, e.message))
        .collect::<Vec<_>>()
        .join(", ");
    bail!("Cloudflare API rejected request: {detail}")
}

pub async fn run_tcp_forwarder(listen: SocketAddr, target: SocketAddr) -> Result<()> {
    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("binding public listener {listen}"))?;
    info!(%listen, %target, "TCP forwarder listening");

    loop {
        let (mut inbound, peer) = listener.accept().await?;
        tokio::spawn(async move {
            let result: Result<()> = async {
                let mut outbound = TcpStream::connect(target)
                    .await
                    .with_context(|| format!("connecting local target {target}"))?;
                let (up, down) = io::copy_bidirectional(&mut inbound, &mut outbound).await?;
                info!(%peer, up, down, "forwarded connection closed");
                Ok(())
            }
            .await;
            if let Err(err) = result {
                warn!(%peer, error = %err, "forwarded connection failed");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_ipv6() {
        assert!(!is_globalish_ipv6("fc00::1".parse().unwrap()));
        assert!(!is_globalish_ipv6("fe80::1".parse().unwrap()));
        assert!(!is_globalish_ipv6("::1".parse().unwrap()));
        assert!(is_globalish_ipv6("2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn parses_xor_mapped_ipv4() {
        let txid = [7u8; 12];
        let ip = Ipv4Addr::new(203, 0, 113, 44);
        let port = 42424u16;
        let cookie = STUN_MAGIC_COOKIE.to_be_bytes();
        let mut attr = vec![0, 0x01];
        attr.extend_from_slice(&(port ^ (STUN_MAGIC_COOKIE >> 16) as u16).to_be_bytes());
        let raw = ip.octets();
        attr.extend(raw.iter().zip(cookie).map(|(a, b)| a ^ b));

        let parsed = parse_address_attr(&attr, true, txid).unwrap();
        assert_eq!(parsed, SocketAddr::new(IpAddr::V4(ip), port));
    }
}
