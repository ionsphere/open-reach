# OpenReach: Initial System Design

Status: design proposal for review

## 1. Goal

OpenReach makes a service running on an ordinary local machine securely reachable from the Internet under a stable user-controlled DNS name.

The design assumes no cooperation from the user's router or ISP beyond ordinary outbound Internet access.

The essential guarantee is:

```text
local service -> outbound-capable endpoint -> globally reachable named service
```

Direct connectivity is preferred whenever possible, but outbound-established relay reachability is the correctness baseline.

## 2. Architectural principles

1. **Outbound-only correctness.** No unsolicited inbound packet to the home network is required for baseline operation.
2. **DNS remains the namespace.** The public hostname is stable and user-controlled.
3. **Service identity is cryptographic.** DNS binds the hostname to a stable service key independent of current network location.
4. **Reuse ICE rather than reinvent NAT traversal.** ICE/STUN/TURN are the starting point for native-client path selection.
5. **No router dependency.** UPnP, NAT-PMP, PCP, public IPv4, and delegated IPv6 are optional optimizations.
6. **No ISP dependency.** Static addresses, CGN mappings, or ISP-side OpenReach support are not required.
7. **Local machine remains the server.** Public infrastructure transports traffic; it does not host application state.
8. **Direct paths are opportunistic.** The system removes relay traffic whenever a safe direct path is available.
9. **Open protocol, replaceable infrastructure.** Rendezvous and relay providers must be separable from service identity.
10. **Application transparency.** Existing local applications should not need OpenReach-specific code.

## 3. User model

A user publishes a service:

```text
openreach publish \
  --from http://127.0.0.1:8080 \
  --as photos.example.com \
  --visibility public
```

Conceptually:

```text
Publication {
    id
    dns_name
    service_identity
    local_target
    protocol
    visibility
    transport_policy
}
```

## 4. Naming and identity

### 4.1 DNS name

The public service name is an ordinary DNS hostname such as:

```text
photos.example.com
```

This is the human-facing identity, administrative authority, and compatibility surface.

### 4.2 Service key

Each publication or endpoint has a stable asymmetric identity key.

The DNS zone controlled by the user binds the hostname to that public key.

Conceptually:

```text
photos.example.com
        |
        v
DNS authoritative binding
        |
        v
service public key K
```

The exact DNS encoding is not yet frozen. Candidate mechanisms include:

- namespaced TXT records for initial prototypes;
- SVCB/HTTPS parameters where appropriate;
- a future dedicated RR type if standardization warrants it.

An illustrative early record might be:

```text
_openreach.photos.example.com TXT "v=or1 id=<key> provider=<provider>"
```

### 4.3 Compatibility records

The same hostname can also expose conventional records for ordinary clients:

```text
A / AAAA / CNAME / HTTPS -> compatibility ingress
```

OpenReach-aware clients may additionally discover service identity and rendezvous metadata from DNS.

This allows the same URL to support both today's Internet and a more direct OpenReach path.

## 5. Components

### 5.1 Endpoint agent

Runs on the user's machine.

Responsibilities:

- generate and store endpoint/service keys;
- configure publications;
- establish outbound control and relay sessions;
- proxy traffic to local applications;
- participate in ICE candidate gathering and checks;
- manage direct/relay path migration;
- manage certificates where endpoint TLS is used;
- enforce local publication policy;
- expose diagnostics and health state.

The endpoint agent must work entirely in user space without router administration.

### 5.2 Rendezvous/control service

A low-bandwidth public service used to locate live service instances and coordinate connectivity.

Responsibilities:

- endpoint presence;
- publication registration;
- signed service-state publication;
- exchange of ICE candidates for OpenReach-aware clients;
- relay assignment/discovery;
- short-lived transport credentials;
- authorization metadata;
- provider discovery and failover coordination.

It should not be in the bulk data path after a transport has been selected.

### 5.3 Relay service

Provides guaranteed reachability when no direct path succeeds.

The endpoint establishes the relay relationship outbound.

For a native OpenReach client:

```text
client -> relay -> endpoint
```

For an ordinary browser:

```text
browser -> compatibility ingress/relay -> endpoint
```

Those two relay roles may use different protocols or infrastructure.

### 5.4 Compatibility ingress

A globally reachable endpoint for unmodified Internet clients.

It exists because an ordinary browser opening `https://photos.example.com` will not run OpenReach rendezvous and ICE before opening HTTP.

The compatibility ingress should be as thin and replaceable as possible.

### 5.5 DNS integration

DNS has two responsibilities:

1. preserve ordinary browser compatibility;
2. bind the service's human-readable name to stable OpenReach identity and discovery metadata.

OpenReach should not require a new global naming hierarchy.

## 6. Connectivity architecture

### 6.1 Native OpenReach path selection

For OpenReach-aware clients, ICE is the default starting point.

Candidates may include:

- host candidates;
- globally routable IPv6 addresses;
- server-reflexive STUN candidates;
- TURN relay candidates;
- optional explicitly mapped candidates from PCP/NAT-PMP/UPnP;
- OpenReach-specific relay candidates if TURN does not fit all requirements.

The initial design position is:

> Use standardized ICE behavior unless a concrete OpenReach requirement demonstrates that it is insufficient.

### 6.2 Path preference

Conceptually:

```text
native direct
   > NAT-traversed direct
   > explicit mapped direct
   > relay
```

Actual ICE priority and policy may also consider latency, stability, cost, metering, privacy, and user preferences.

### 6.3 Router-assisted optimization

If available, the endpoint may attempt:

- PCP;
- NAT-PMP;
- UPnP IGD;
- platform/router-specific APIs.

These are optimization candidates only. Their absence must never prevent publication.

## 7. Relay protocol choices

There are two distinct relay problems.

### 7.1 Native-client relay

TURN is a strong candidate because it already supplies relay candidates that integrate with ICE.

OpenReach should evaluate whether standard TURN allocations and permissions fit persistent service semantics well enough or require profiling/extensions.

### 7.2 Browser compatibility ingress

This path has different requirements:

- persistent endpoint sessions;
- many unrelated clients;
- hostname routing;
- HTTP/TLS compatibility;
- multiplexing;
- low idle overhead;
- provider federation;
- abuse controls.

QUIC, HTTP/2/3, CONNECT, MASQUE-family mechanisms, or a small standardized framing layer may fit better here than TURN.

OpenReach should not force both relay roles into one protocol merely for implementation uniformity.

## 8. Transport from endpoint to relay

The endpoint should maintain a small number of authenticated long-lived outbound sessions.

Preferred transports:

1. QUIC/HTTP/3;
2. HTTP/2 over TLS;
3. WebSocket/TLS only as a restrictive-network fallback if necessary.

The transport must tolerate:

- reconnects;
- NAT rebinding;
- address changes;
- network transitions;
- multiplexed publications;
- multiplexed client streams.

## 9. TLS model

Two modes are useful.

### Endpoint-terminated TLS

The public relay forwards opaque TLS bytes to the endpoint, which owns the certificate private key.

Advantages:

- provider cannot inspect application plaintext;
- service keys and TLS keys remain local;
- strong provider portability.

Tradeoffs:

- hostname routing must rely on visible handshake metadata or explicit routing metadata;
- some edge security features are unavailable.

This should be the preferred long-term mode where practical.

### Edge-terminated TLS

The compatibility ingress terminates public TLS and establishes an independently encrypted authenticated tunnel to the endpoint.

Advantages:

- straightforward HTTP routing;
- easier compatibility and edge abuse controls;
- simpler certificate automation for early implementations.

Tradeoff:

- ingress can observe plaintext application traffic.

This should be an explicit policy choice rather than the only architecture.

## 10. Connection flows

### 10.1 Ordinary browser

```text
1. browser resolves photos.example.com
2. DNS returns conventional compatibility-ingress records
3. browser connects normally over TCP/QUIC + TLS
4. ingress identifies the publication
5. ingress selects a live endpoint session
6. endpoint proxies to localhost:8080
```

No inbound connection to the home network is required.

### 10.2 OpenReach-aware client

```text
1. client queries DNS
2. client obtains service identity/discovery metadata
3. client verifies the DNS-bound service identity
4. client contacts rendezvous/control service
5. client and endpoint gather/exchange ICE candidates
6. ICE connectivity checks select a direct path if possible
7. TURN/OpenReach relay remains fallback
8. service identity authenticates the endpoint independently of path
```

## 11. Service mobility

The stable identity model should permit:

```text
Windows PC -> NAS -> mini-PC -> laptop -> new network
```

without changing:

```text
https://photos.example.com
```

A machine migration updates signed reachability state, not the public service identity.

## 12. Public and private services

### Public

Anyone may connect to the public hostname.

### Authenticated

Ingress or endpoint policy authenticates the caller before exposing the local service.

Potential mechanisms include passkeys/WebAuthn, OIDC, mTLS/client keys, or invitation credentials.

### Private native mode

Only authorized OpenReach clients may discover or connect to the service.

This overlaps with mesh VPNs, but the abstraction remains **published service** rather than **joined private LAN**.

## 13. Provider federation

A long-term provider descriptor might expose:

```text
Provider {
    identity
    rendezvous_endpoints
    relay_endpoints
    ingress_endpoints
    supported_transports
    regions
    policy
}
```

A service may maintain sessions with multiple providers simultaneously.

Changing providers must not require changing the DNS service name or cryptographic service identity.

## 14. Failure behavior

### Endpoint disconnect

Ingress should return an explicit unavailable response.

### Relay/provider failure

Endpoint reconnects to another configured provider; clients select another advertised path.

### Home IP changes

Endpoint re-establishes outbound presence; service identity and DNS name remain unchanged.

### Router replacement

No baseline action is required.

### ISP moves customer behind CGNAT

Direct candidates disappear; relay continues working.

### IPv6 becomes available later

ICE gains a new direct candidate automatically.

### UDP blocked

Control/data transport falls back to TLS-based alternatives where required.

## 15. Privacy and trust

OpenReach separates payload privacy from metadata privacy.

A relay or rendezvous operator may learn some combination of:

- endpoint/provider account identity;
- service being accessed;
- client IP;
- timing;
- traffic volume.

Endpoint-terminated TLS prevents relay visibility into application plaintext but does not by itself provide Tor-style anonymity.

Anonymity is not an initial OpenReach goal.

## 16. Abuse resistance

Public compatibility ingress requires operational controls including:

- authenticated publication enrollment;
- rate limits;
- connection quotas;
- anti-amplification rules;
- destination restrictions;
- revocable publication credentials;
- complaint/takedown mechanisms for public operators;
- separation between disabling one publication and accessing the user's LAN.

HTTP should likely precede raw TCP/UDP support because it has a smaller abuse surface.

## 17. Initial implementation scope

### V0: prove outbound-only publication

- endpoint agent;
- single combined rendezvous/ingress relay;
- persistent outbound TLS or QUIC session;
- HTTP reverse proxy;
- provider-owned test hostname;
- basic service identity;
- no router integration.

### V1: DNS-bound user-owned services

- custom hostname;
- DNS service-key binding;
- ACME automation;
- public HTTP/HTTPS;
- reconnect/failover semantics;
- diagnostics.

### V2: standardized native connectivity

- ICE implementation/profile;
- STUN candidate gathering;
- TURN evaluation/integration;
- native IPv6 paths;
- optional PCP/NAT-PMP/UPnP candidates;
- automatic direct/relay selection.

### V3: native clients and provider federation

- OpenReach-aware client library/agent;
- multi-provider support;
- self-hosted interoperable providers;
- private/authenticated publications;
- provider discovery.

### V4: broader protocols

- raw TCP;
- UDP;
- MASQUE-compatible tunneling where useful;
- mobile/embedded endpoint agents;
- richer OS/browser integration experiments.

## 18. Repository shape

Possible implementation layout:

```text
cmd/
  openreach/
  reachd/

internal/
  agent/
  identity/
  publication/
  dns/
  rendezvous/
  ice/
  stun/
  turn/
  relay/
  ingress/
  transport/
  proxy/
  acme/

docs/
  MOTIVATION.md
  DESIGN.md
  WEBRTC_COMPARISON.md
  TOR_COMPARISON.md
```

Rust and Go are natural implementation candidates; language choice should follow protocol and deployment review.

## 19. Questions for design review

1. Should a publication have its own key, inherit an endpoint key, or use a hierarchy?
2. Which DNS representation best binds hostname to service identity without requiring new resolver behavior for ordinary clients?
3. Can SVCB/HTTPS carry enough OpenReach discovery metadata cleanly, or should early versions use namespaced TXT?
4. How literally should OpenReach adopt ICE, and which persistent-service requirements differ from ordinary ICE use?
5. Can standard TURN serve native OpenReach relay needs without undesirable allocation/lifecycle behavior?
6. What is the best protocol for ordinary-browser ingress-to-endpoint transport: HTTP/2, HTTP/3, MASQUE, or custom framing?
7. Should v1 prefer endpoint-terminated or edge-terminated TLS?
8. How should DNS/provider failover avoid long convergence delays?
9. How should signed reachability descriptors be cached and revoked?
10. What is the minimum viable abuse-control system for a public community ingress provider?

## 20. Architectural invariants

OpenReach should preserve these through every revision:

> A published endpoint remains reachable when the local network permits outbound connections only.

> The normal public service name remains an ordinary DNS name controlled by the user.

> DNS can bind that name to a stable cryptographic service identity independent of IP address, router, ISP, physical machine, and relay provider.

> Existing connectivity standards are reused unless a concrete requirement demonstrates that they are insufficient.

> Router and ISP features improve paths but are never prerequisites for correctness.
