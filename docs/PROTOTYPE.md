# OpenReach production probe

Status: first runnable experiment

This prototype is intentionally biased toward measurement. It does not claim that a STUN-observed public IPv4 address is automatically reachable for inbound TCP. Instead it publishes what the endpoint can observe, keeps DNS fresh, listens for direct TCP connections, and lets real networks tell us which paths work.

## What exists now

The `openreach` binary has four commands:

```text
openreach probe
openreach sync-dns
openreach forward
openreach run
```

`probe` discovers:

- a globally usable outbound IPv6 source address, when the OS has one;
- the endpoint's STUN-mapped UDP address;
- the public IPv4 address observed by STUN.

`sync-dns` writes the currently observed A/AAAA records plus an experimental `_openreach.<hostname>` TXT record to Cloudflare DNS.

`forward` listens on a TCP socket and forwards each connection to a local application without interpreting the application protocol.

`run` combines continuous probing/DNS refresh with the TCP forwarder.

## Cloudflare setup

Use an API token scoped to the one DNS zone used for the experiment. The prototype requires DNS record read/write access because it finds an existing record before updating it. Supplying the Cloudflare Zone ID avoids needing broad zone-discovery logic in the endpoint.

Set:

```text
CLOUDFLARE_API_TOKEN=<scoped token>
CLOUDFLARE_ZONE_ID=<zone id>
```

The token should not be a Global API Key.

## First test

Suppose a local application listens on `127.0.0.1:8080` and the desired hostname is `test.example.com`.

Run:

```text
openreach run \
  --hostname test.example.com \
  --target 127.0.0.1:8080 \
  --listen 0.0.0.0:8443
```

OpenReach will:

1. make a STUN binding request;
2. discover an outbound IPv6 address if available;
3. synchronize Cloudflare DNS;
4. publish `_openreach.test.example.com` metadata;
5. listen on TCP port 8443 and forward accepted connections to port 8080;
6. repeat discovery and DNS synchronization as the network changes.

The TXT record is experimental and currently resembles:

```text
v=or1; tcp=8443; stun=203.0.113.5:49152; ipv6=2001:db8::1234
```

It is diagnostic metadata, not a frozen wire standard.

## Important interpretation

An A record derived from the IP observed by STUN does **not** prove that TCP port 8443 is reachable through the NAT. The STUN mapping is UDP and may be endpoint-dependent. The first experiment deliberately exposes this distinction so it can be measured.

IPv6 is much more promising for unmodified clients when the endpoint has a global address and the local firewall permits the listening port. IPv4 may work directly on public-address connections but normally still needs a mapping when a consumer NAT is present.

The next traversal work should be driven by captured results from actual networks:

- direct global IPv6 success/failure;
- public IPv4 success/failure;
- NAT type and STUN mapping stability;
- whether automatic PCP/NAT-PMP/UPnP can create a TCP mapping;
- whether an OpenReach-aware client can establish a one-sided, DNS-bootstrapped direct UDP/QUIC path.

## Security

The current forwarder is a raw byte forwarder. It does not add authentication or TLS to the local application. For Internet tests, expose only an application that already speaks a secure protocol or use a temporary test service with no sensitive data.

Cloudflare records are created with `proxied=false`; enabling Cloudflare proxying would change the experiment into a Cloudflare-hosted ingress path, which is specifically not what this phase is trying to measure.

## Builds

CI builds and tests the workspace on:

- Linux;
- macOS;
- Windows.

Each CI run uploads the release binary as a workflow artifact so the same commit can be tried on multiple endpoint networks without installing a Rust toolchain.
