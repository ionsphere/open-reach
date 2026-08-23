# OpenReach

**Persistent Internet service identity with reachability for local endpoints.**

OpenReach explores a missing consumer Internet primitive: make a service running on an ordinary local computer securely reachable under a stable public DNS name, without requiring router configuration, a static IP address, or ISP cooperation.

The local machine remains the server. OpenReach supplies the missing reachability layer.

## Core idea

A user should be able to do this:

```text
openreach publish localhost:8080 --as photos.example.com
```

and have `https://photos.example.com` remain reachable even if the endpoint is behind NAT, CGNAT, a changing IP address, or a router that cannot be configured.

The baseline works when the endpoint can make **outbound connections only**. Direct connectivity is then discovered opportunistically.

## Existing Internet infrastructure first

OpenReach is not intended to replace the Internet stack. It should compose existing pieces:

```text
DNS
  |
  | hostname -> stable service identity
  v
service discovery / rendezvous
  |
  v
ICE
 /   \
direct relay
       |
   TURN and/or
   Internet-standard transport
```

For ordinary browsers that cannot participate in OpenReach rendezvous/ICE, a thin globally reachable compatibility ingress provides the bridge to the local endpoint.

## DNS stays the namespace

OpenReach does not require a `.onion`-style namespace.

A normal hostname such as:

```text
photos.example.com
```

remains the human-facing public name. DNS can carry both normal compatibility records and a binding to the service's stable cryptographic identity.

That identity remains stable while the endpoint's IP address, machine, ISP, network, and relay provider change.

## Design influences

Two existing architectures are especially important:

- **Tor onion services** demonstrate that a server can remain reachable without accepting unsolicited inbound Internet connections.
- **WebRTC/ICE/STUN/TURN** provide mature standardized machinery for discovering direct peer paths and falling back to relays.

OpenReach combines those lessons with ordinary DNS and ordinary web compatibility.

## Design documents

- [Motivation](docs/MOTIVATION.md) — why local hosting is easy but global reachability is still unnecessarily difficult.
- [Initial system design](docs/DESIGN.md) — naming, service identity, rendezvous, ICE, relays, browser ingress, TLS, federation, security, failure behavior, and staged implementation.
- [WebRTC comparison](docs/WEBRTC_COMPARISON.md) — what ICE/STUN/TURN already solve and what persistent named services add.
- [Tor comparison](docs/TOR_COMPARISON.md) — what onion services teach about outbound-only reachability, identity, rendezvous, and what OpenReach deliberately does differently.

## Architectural invariants

> A published endpoint remains reachable when its local network permits outbound connections only.

> The normal public service name remains an ordinary DNS name controlled by the user.

> DNS can bind that name to a stable cryptographic service identity independent of current location.

> Router and ISP features may improve a path but are never prerequisites for correctness.

> Existing standards should be reused unless a concrete requirement demonstrates that they are insufficient.
