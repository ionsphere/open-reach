# OpenReach Discovery and Rendezvous

Status: architecture refinement

## 1. Goal

OpenReach should not require either the user or the OpenReach project to operate a public VPS, cloud service, or globally shared edge as a prerequisite.

The minimum user-owned system is:

```text
local applications
      |
      v
OpenReach endpoint
      |
      +---- DNS provider API ----> authoritative DNS
```

The endpoint itself performs publication, identity management, reachability probing, DNS updates, and local proxying.

Public OpenReach infrastructure is therefore **optional capability**, not baseline ownership.

This creates an important distinction:

- **zero-hosting mode**: no OpenReach-hosted service is required; reachability is provided only by direct paths the Internet makes possible;
- **assisted mode**: an optional rendezvous or relay provider is used when a network topology cannot establish the desired connection directly;
- **universal browser mode**: a globally reachable compatibility ingress is required whenever an ordinary browser cannot directly reach the endpoint.

OpenReach must not conceal those differences.

## 2. DNS is the durable public control plane

A normal DNS name remains the root of the service:

```text
photos.example.com
```

OpenReach receives narrowly scoped authority to update records for the names it manages.

DNS can publish durable and semi-dynamic information such as:

- service identity/public key;
- OpenReach protocol version;
- current public IPv4/IPv6 addresses when available;
- ports and transport descriptors;
- public ICE/server-reflexive candidates where appropriate;
- certificate-related records;
- optional rendezvous/relay providers selected by the user;
- ordinary A/AAAA/HTTPS compatibility records.

Conceptually:

```text
photos.example.com
    |
    +-- service identity K
    +-- current direct reachability
    +-- optional assistance providers
    `-- ordinary web records
```

DNS is already globally hosted infrastructure chosen and paid for by the domain owner. OpenReach should use it aggressively rather than creating a second global directory.

## 3. What DNS can replace

For a long-lived server endpoint, DNS can replace much of the service-discovery role that WebRTC applications normally implement with their own signaling server.

The server side is not an unknown transient peer. It is a persistent named service.

OpenReach can therefore publish server-side reachability asynchronously:

```text
OpenReach endpoint
    |
    | discover addresses / mappings / STUN candidates
    |
    v
DNS update
    |
    v
client later resolves photos.example.com
```

That is fundamentally different from a WebRTC call where both peers appear at session time and exchange offers and candidates interactively.

For direct paths that allow the client to initiate communication from the advertised server information, no separately hosted OpenReach signaling system is required.

## 4. The limit: DNS is not a bidirectional ICE signaling bus

ICE assumes that agents can exchange candidate information through signaling outside ICE.

Publishing the server's candidates in DNS covers one direction well:

```text
server -> DNS -> client
```

It does not naturally provide the reverse session-time path:

```text
client -> server
```

That matters for network types where successful hole punching requires the server to know the client's current candidate and transmit toward it before the NAT/firewall will admit the client's traffic.

Examples include some address-dependent filtering and symmetric-NAT cases.

Giving arbitrary clients write access to the user's DNS zone is not acceptable, and authoritative DNS hosting is not normally an arbitrary low-latency message queue.

Therefore zero-hosting OpenReach cannot promise that every ICE topology will work.

This is a physical connectivity limit, not a missing implementation trick.

## 5. Zero-hosting reachability ladder

The endpoint should attempt every direct mechanism available without relying on OpenReach-operated infrastructure.

### A. Public IPv6

If the endpoint has usable globally routed IPv6 and local/network firewall policy permits the service, publish it directly.

```text
client ----------------------> endpoint
```

### B. Public IPv4

If the host has public IPv4 reachability, publish and maintain it dynamically through DNS.

### C. Explicit router mapping

Attempt supported mappings opportunistically:

- PCP;
- NAT-PMP;
- UPnP IGD.

Success creates a directly publishable endpoint. Failure is normal and requires no user intervention.

### D. STUN-discovered server-reflexive path

The endpoint may use STUN to learn how it is represented externally and publish appropriate short-lived reachability metadata.

STUN itself is a tiny generic Internet service and need not be operated by the OpenReach project. OpenReach should support configurable STUN services and standard discovery.

Some NAT/filtering combinations allow remote clients to connect using this information directly; others do not.

### E. Direct client-assisted negotiation

An OpenReach-aware client can try all advertised candidates and protocol-specific connectivity checks.

Where incoming checks reach the endpoint, peer-reflexive information can also be learned dynamically.

This still requires no OpenReach cloud.

### F. Assistance provider

Only if direct establishment cannot work does the system need a third party capable of rendezvous, relay, or both.

That provider is not inherently OpenReach-operated.

It may be:

- a commercial TURN provider;
- an ISP-provided service;
- a DNS/hosting provider that offers compatible rendezvous/relay functionality;
- an organization or employer service;
- a community provider;
- the user's own infrastructure if they already have it;
- any future interoperable OpenReach provider.

OpenReach defines interoperability with assistance providers; it does not promise to subsidize them.

## 6. Reachability classes

OpenReach should expose the actual capability obtained for each publication rather than flatten everything into "online".

Suggested states:

```text
DIRECT
  globally reachable without OpenReach assistance

DIRECT_NATIVE_ONLY
  reachable directly by OpenReach-aware clients, but not ordinary clients

ASSISTED
  requires a configured rendezvous/relay provider

BROWSER_ASSISTED
  ordinary browsers require a compatibility ingress

LOCAL_ONLY
  currently no viable Internet path
```

The UI can explain why and what optional step would improve reachability.

Example:

```text
photos.example.com
Online: direct IPv6
Browser access: yes
Hosted OpenReach infrastructure: none
```

or:

```text
home.example.com
Online: native OpenReach clients only
Reason: NAT traversal succeeds only with client participation
Browser access: unavailable without an ingress provider
Hosted OpenReach infrastructure: none
```

## 7. Ordinary browser compatibility remains the hard boundary

An unmodified browser opening:

```text
https://photos.example.com
```

performs ordinary DNS resolution and then connects to a conventional IP endpoint.

If the user's endpoint is directly reachable, this works with zero OpenReach hosting.

If it is not directly reachable, DNS cannot make the browser perform OpenReach rendezvous or NAT hole punching.

In that case there are only two choices:

1. browser compatibility is unavailable for that network configuration; or
2. the user selects a compatible public ingress provider.

OpenReach should not silently turn option 2 into a mandatory OpenReach-operated cloud service.

## 8. User-selected assistance is analogous to DNS hosting

The product model should resemble domain/DNS selection rather than SaaS lock-in.

A user may already choose:

```text
registrar:      provider A
DNS hosting:    provider B
OpenReach aid:  none
```

If their topology later needs assistance:

```text
registrar:      provider A
DNS hosting:    provider B
rendezvous:     provider C
relay:          provider D
```

Those roles may collapse into one provider but must remain protocol-level roles, not one required account system.

DNS records can advertise which optional providers the publication uses.

## 9. No-provider startup path

A first-run OpenReach installation should not ask for an OpenReach account.

The setup path should be:

1. create or import a service identity;
2. connect a DNS provider or configure delegated dynamic-DNS authority;
3. select one or more local applications/ports;
4. assign public DNS names;
5. probe local and external network capabilities;
6. publish the best direct records OpenReach can establish;
7. report the resulting reachability class;
8. optionally offer compatible assistance providers only if required for capabilities the user wants.

The user should be able to stop at step 7 with a fully functioning zero-hosting deployment whenever their network permits it.

## 10. DNS credential model

OpenReach should avoid asking for registrar-wide credentials.

Preferred setup mechanisms, in order of isolation, include:

1. provider OAuth with record/zone-scoped permission;
2. provider API token scoped to one zone or subdomain;
3. delegated sub-zone whose credentials belong only to OpenReach;
4. RFC 2136 dynamic-update credentials scoped to the OpenReach names;
5. manual initial delegation followed by automatic operation.

The endpoint should store credentials using the native OS secret store.

## 11. Dynamic DNS data lifecycle

Not all records need the same update cadence.

### Durable

- service public key;
- publication policy/version;
- delegated authority.

### Network-change driven

- A/AAAA addresses;
- direct transport endpoints;
- explicit NAT mappings.

### Short-lived

- server-reflexive candidates;
- temporary connection descriptors.

Short-lived records need appropriately low TTLs and conservative update behavior. OpenReach should measure real DNS-provider update/caching behavior rather than assume DNS is instantaneous.

## 12. Optional rendezvous without relay

A useful middle ground may exist between zero infrastructure and full TURN relay.

A third party can exchange transient client/server candidates without carrying application traffic:

```text
client ---- signaling ----+
                          | rendezvous
server ---- signaling ----+

client ================= server
           direct data
```

Such a service consumes tiny bandwidth compared with relay hosting.

OpenReach should define this as an optional provider capability separately from TURN/relay.

The project itself still need not operate one at scale.

## 13. Optional relay

Some topology combinations cannot establish direct communication. TURN explicitly exists to provide a relayed candidate for such cases.

OpenReach should support TURN and other standardized relay transports as pluggable user-selected services.

The economic boundary is therefore clear:

> Users who need relay bandwidth acquire relay bandwidth from a provider of their choice; OpenReach itself remains software and protocol infrastructure.

## 14. Design rule

> OpenReach must require no OpenReach-operated hosted infrastructure. The local endpoint plus user-controlled DNS is the baseline system. Public rendezvous, relay, and browser ingress are optional capabilities selected only when the desired reachability cannot be obtained directly.

A second rule follows:

> Zero hosting and universal reachability are different guarantees. OpenReach should maximize the first and accurately report when the second requires third-party public infrastructure.
