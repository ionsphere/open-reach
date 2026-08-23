# OpenReach

**Persistent Internet service identity with reachability for local endpoints.**

OpenReach explores a missing consumer Internet primitive: make a service running on an ordinary local computer securely reachable under a stable public DNS name, without requiring the user to deploy cloud infrastructure.

The local machine remains the server. OpenReach uses the user's existing Internet connection and DNS authority to obtain the strongest direct reachability available, and can use optional third-party rendezvous, relay, or compatibility ingress only when the network topology requires it.

## Core idea

A user should be able to do this:

```text
openreach publish localhost:8080 --as photos.example.com
```

OpenReach should then:

1. create a stable cryptographic service identity;
2. maintain the DNS records for `photos.example.com`;
3. discover every viable direct path;
4. publish the strongest secure reachability the current network allows;
5. accurately report when a desired capability requires optional public infrastructure.

The default installation requires no OpenReach cloud account or OpenReach-operated edge.

## User-owned baseline

The minimum deployment is:

```text
local applications
      |
      v
OpenReach endpoint
      |
      +---- scoped DNS access ----> authoritative DNS provider
```

The user supplies:

- an application running locally;
- a domain or delegated DNS name;
- authorization for OpenReach to maintain the necessary DNS records.

That is enough whenever the endpoint can establish a viable direct Internet path.

## Existing Internet infrastructure first

OpenReach is not intended to replace the Internet stack. It should compose existing pieces:

```text
DNS
  |
  | name -> service identity + current reachability
  v
OpenReach-aware client / ordinary client
  |
  +---- direct IPv6 / IPv4
  +---- explicit NAT mapping
  +---- STUN-assisted direct path
  `---- optional rendezvous / relay / ingress when required
```

ICE/STUN/TURN remain important standards, but OpenReach does not assume that an OpenReach project service supplies them.

## DNS stays the namespace and durable control plane

OpenReach does not require a `.onion`-style namespace.

A normal hostname such as:

```text
photos.example.com
```

remains the human-facing public name. DNS can carry normal compatibility records, a binding to the service's stable cryptographic identity, current direct reachability metadata, and optional assistance-provider information.

The service identity remains stable while the endpoint's IP address, machine, ISP, network, or assistance provider changes.

## No-hosting does not mean every topology is reachable

Some network combinations cannot accept an unsolicited direct connection and cannot complete NAT traversal without interactive rendezvous. Some cannot establish a direct path at all and require a relay.

An ordinary browser also cannot perform OpenReach-specific rendezvous before opening `https://photos.example.com`.

OpenReach therefore distinguishes:

- **zero-hosting direct reachability** — works with only the endpoint and DNS when the network permits it;
- **native assisted reachability** — optional signaling/rendezvous helps OpenReach-aware peers establish a direct path;
- **relayed reachability** — optional relay carries traffic when no direct path exists;
- **browser compatibility ingress** — optional public ingress is required when an ordinary browser must reach a non-directly-addressable endpoint.

OpenReach itself is software and protocol infrastructure; it should not require the project to subsidize a global relay network.

## Design influences

Two existing architectures are especially important:

- **Tor onion services** demonstrate how identity and location can be separated and why outbound-established reachability works when a public rendezvous/relay network exists.
- **WebRTC/ICE/STUN/TURN** provide mature standardized machinery for discovering direct peer paths and falling back to relays.

OpenReach combines those lessons with ordinary DNS, persistent named services, and a no-mandatory-cloud product model.

## Design documents

- [Motivation](docs/MOTIVATION.md) — why local hosting is easy but global reachability is still unnecessarily difficult.
- [Initial system design](docs/DESIGN.md) — naming, service identity, rendezvous, ICE, relays, browser ingress, TLS, federation, security, failure behavior, and staged implementation.
- [User model](docs/USER_MODEL.md) — the local-app + domain + DNS-authority setup flow and when optional assistance is introduced.
- [Implementation plan](docs/IMPLEMENTATION_PLAN.md) — code workstreams, platform/build matrix, mobile support, CI, and phased milestones.
- [Discovery and rendezvous](docs/DISCOVERY.md) — zero-hosting discovery through DNS, direct-path capabilities, and the precise cases where rendezvous/relay/ingress become necessary.
- [WebRTC comparison](docs/WEBRTC_COMPARISON.md) — what ICE/STUN/TURN already solve and what persistent named services add.
- [Tor comparison](docs/TOR_COMPARISON.md) — what onion services teach about outbound-only reachability, identity, rendezvous, and what OpenReach deliberately does differently.

## Architectural invariants

> The normal public service name remains an ordinary DNS name controlled by the user.

> OpenReach requires no OpenReach-operated hosted infrastructure for its baseline installation.

> DNS binds the public name to a stable cryptographic service identity independent of current location.

> OpenReach attempts direct reachability first and clearly identifies when the requested reachability requires optional rendezvous, relay, or browser ingress.

> Router and ISP features may improve a path but are never required integrations.

> Existing standards should be reused unless a concrete requirement demonstrates that they are insufficient.
