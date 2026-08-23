# OpenReach Discovery and Rendezvous

Status: architecture refinement

## 1. Question

Does OpenReach need a separately hosted service-discovery system if DNS is already hosted by globally available DNS providers?

The proposed answer is **no, not as a separate product or mandatory infrastructure tier**.

OpenReach needs three different things that are easy to conflate:

1. **durable discovery** — what service is this name, and which providers/protocols can reach it?
2. **live presence** — is the endpoint online right now, and which outbound session currently represents it?
3. **connection rendezvous** — exchange short-lived connection state such as ICE offers, answers, candidates, and credentials.

DNS is well suited to the first. It is a poor fit for the second and third.

The relay/control edge already required for guaranteed outbound-only reachability is a natural place for live presence and rendezvous. Therefore OpenReach does not require another separately deployed hosted discovery service.

## 2. DNS is the durable discovery root

A normal public hostname remains the root of discovery:

```text
photos.example.com
```

DNS can carry:

- the stable OpenReach service identity;
- protocol/version information;
- one or more OpenReach provider or rendezvous entry points;
- ordinary A/AAAA/CNAME/HTTPS records for browser compatibility;
- provider priority/failover information;
- future SVCB/HTTPS parameters once standardized/appropriate.

Conceptually:

```text
photos.example.com
    |
    +-- normal web compatibility -> ingress.example.net
    |
    `-- OpenReach metadata
          service key: K
          providers:
            - reach1.example.net
            - reach2.example.net
```

The service key remains stable while provider choice, endpoint IP, home network, and active relay session change.

## 3. Why live presence should not be stored in DNS

It is technically possible to update DNS frequently. DNS UPDATE exists, and hosted DNS providers expose APIs. That does not make DNS a good real-time signaling bus.

OpenReach presence may change because:

- a laptop sleeps or wakes;
- Wi-Fi changes;
- a mobile client moves networks;
- NAT mappings expire;
- an endpoint reconnects to a different edge;
- ICE candidates change;
- a relay fails;
- a direct path becomes available or disappears.

Publishing every such transition into authoritative DNS would interact badly with:

- recursive resolver caching;
- TTL behavior;
- negative caching;
- provider update latency/rate limits;
- DNS propagation semantics;
- the need for rapid offer/answer exchange;
- privacy leakage from publishing ephemeral network candidates globally.

DNS should therefore describe **how to enter the OpenReach system**, not every current detail of an active connection.

## 4. No standalone discovery server is required

The baseline can be:

```text
                   DNS
                    |
         name -> identity + provider
                    |
                    v
              relay/control edge
                 ^          ^
                 |          |
          outbound session  |
                 |          |
             endpoint     client
```

The endpoint maintains an authenticated outbound session to one or more selected relay/control edges.

That live session itself is the presence record.

A native client:

1. resolves the DNS name;
2. verifies the DNS-bound service identity;
3. discovers one or more provider/control edges;
4. contacts a provider edge;
5. asks for service `K`;
6. the edge matches `K` to the endpoint's current authenticated session;
7. the edge brokers ICE exchange;
8. peers move direct if ICE succeeds, otherwise the edge/associated TURN relay remains the path.

There is no separate global database that must be queried between DNS and the edge.

## 5. Ordinary browser flow is even simpler

An unmodified browser does not understand OpenReach discovery.

DNS supplies a conventional destination:

```text
photos.example.com -> compatibility ingress
```

The compatibility ingress is already connected to or co-located with the control system.

```text
browser
   |
   v
compatibility ingress / edge
   |
   | lookup live service K in local/cluster presence state
   v
outbound endpoint session
```

Again, no separately hosted discovery tier is necessary.

## 6. What still has to be hosted

DNS alone cannot guarantee reachability behind arbitrary CGNAT because some globally reachable node must accept the first connection.

At least one public OpenReach provider therefore needs to host some combination of:

- relay/control edge;
- TURN/STUN service;
- ordinary-browser ingress;
- provider metadata/health endpoint.

These can initially be one deployment artifact.

The architectural goal is not **zero hosted infrastructure**. It is:

> Host only the connectivity machinery that cannot live behind the user's NAT; keep naming under DNS and keep application compute/state on the endpoint.

## 7. Presence can be ephemeral and mostly memory-resident

A control edge does not necessarily require a large durable service database.

For baseline operation it can derive presence from active authenticated sessions:

```text
service key K -> active endpoint connection object
```

If the process dies, the endpoint reconnects and re-registers.

Durable state can be minimized to:

- provider accounts/quotas if a provider chooses to have them;
- revocations;
- optional publication policy;
- operational abuse records.

Service ownership itself remains grounded in DNS + cryptographic identity, rather than in the provider's database.

A multi-node provider will need distributed ephemeral presence or deterministic routing so that an incoming request reaches the edge holding the endpoint session. That is an implementation concern of the provider, not a new Internet-wide discovery service.

## 8. Provider discovery

Provider entry points themselves should be discoverable using existing DNS mechanisms whenever possible.

Candidates include:

- SRV records for protocol-specific endpoints;
- SVCB-style service bindings;
- namespaced TXT records during experimentation;
- standard STUN/TURN DNS discovery where those protocols are used directly.

OpenReach should prefer standards-compatible records rather than require a proprietary global directory.

## 9. DNS security and service-key binding

If DNS is the authority binding a human-readable name to service key `K`, spoofing that binding matters.

The design should support progressively stronger verification:

1. normal DNS plus Web PKI for the compatibility web path;
2. DNSSEC validation for OpenReach-aware clients when available;
3. service-key continuity/pinning after first trusted resolution;
4. well-defined key rotation signed by the old key and/or authorized through DNS;
5. possible DANE/TLSA integration where it materially improves the design.

DNSSEC should not be required for initial deployability because adoption is incomplete, but the protocol must not assume unsigned DNS is cryptographically authoritative by itself.

## 10. Multi-provider operation

DNS can list several providers while the endpoint maintains outbound sessions to more than one:

```text
DNS: service K
  provider A priority 10
  provider B priority 20

endpoint -> A
endpoint -> B
```

A native client can try providers according to policy/priority.

For browser compatibility, DNS/HTTPS records, anycast, or normal load-balancing techniques can direct traffic to available ingress.

Provider failover therefore does not require moving the service identity or application.

## 11. Recommended v1 architecture

Keep the first deployment deliberately compact:

```text
                     authoritative DNS
                           |
             +-------------+-------------+
             |                           |
      ordinary browser              OpenReach client
             |                           |
             v                           v
      +------------------------------------------------+
      |       OpenReach relay/control/ingress edge      |
      |                                                |
      |  browser ingress | rendezvous | STUN/TURN*     |
      +------------------------+-----------------------+
                               ^
                               |
                       outbound session
                               |
                            endpoint
                               |
                         local service
```

`*` STUN/TURN may be separate standard daemons operationally while remaining part of the same provider role.

This is sufficient to prove the product without creating a standalone service-discovery database/API.

## 12. Future decomposition

At scale, provider components can separate:

```text
DNS
 |
 v
provider entry point
 |
 +-- rendezvous/control cluster
 +-- TURN relays
 +-- browser ingress edges
 +-- abuse/account service
```

That is operational decomposition, not an additional architectural dependency for users.

## 13. Design rule

> Durable identity and provider discovery belong in DNS. Live presence and connection negotiation belong on the already-required outbound-connected relay/control edge.

This keeps OpenReach dependent on as little new hosted infrastructure as possible without abusing DNS as a real-time signaling system.
