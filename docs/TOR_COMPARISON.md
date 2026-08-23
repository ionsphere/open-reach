# OpenReach and Tor Onion Services

Tor onion services demonstrate an important property at Internet scale: a server can be reachable without accepting unsolicited inbound packets from the public Internet.

That is directly relevant to OpenReach.

## Why onion services work behind NAT

An onion service creates outbound Tor circuits to introduction points. A client discovers the service, chooses a rendezvous point, and sends rendezvous information through an introduction point. The service then creates another outbound circuit to that rendezvous point.

Conceptually:

```text
service --outbound--> introduction infrastructure
                           |
client  --outbound--> rendezvous <--outbound-- service
```

The home router never needs to expose an inbound port.

This suggests OpenReach's primary reachability invariant:

> A published service must remain reachable when its network permits outbound Internet connections only.

## What OpenReach should borrow

### Identity separate from location

An onion address identifies a cryptographic service rather than a current IP location.

OpenReach should preserve the same separation while keeping ordinary DNS as the public namespace:

```text
photos.example.com
        |
        v
DNS-authoritative binding
        |
        v
stable service public key
        |
        v
current reachability information
```

The service may move between machines, networks, ISPs, and relay providers without changing its public name or cryptographic identity.

### Discovery separate from data transport

Tor separates descriptor/discovery infrastructure, introduction, rendezvous, and application traffic.

OpenReach should likewise avoid assuming that DNS, control/rendezvous, and bulk relay must all be provided by one operator.

### Outbound-only baseline

OpenReach should never require router or ISP participation for correctness. Direct reachability is an optimization layered on top of a guaranteed outbound-established path.

## What OpenReach should not inherit

Tor is designed for anonymity. Its multi-hop circuits deliberately hide client and service network locations from one another and from individual relays.

OpenReach's primary goal is reachability, not anonymity. The preferred native path is therefore much simpler:

```text
client --------------------> endpoint
```

when direct connectivity succeeds, or:

```text
client ---> relay ---> endpoint
```

when it does not.

There is no architectural requirement for Tor-style multi-hop routing.

## The browser difference

Tor works transparently for onion services because Tor Browser and the Tor client participate in onion-service discovery and rendezvous.

An ordinary browser navigating to:

```text
https://photos.example.com
```

expects ordinary DNS plus TCP/QUIC/TLS. It will not automatically perform an OpenReach-specific rendezvous protocol.

Therefore OpenReach needs two compatible modes:

### Ordinary Internet client

```text
browser
   |
   v
public compatibility ingress
   |
   v
endpoint
```

### OpenReach-aware client

```text
client
  |
  +-- DNS discovers service identity
  +-- rendezvous obtains live reachability data
  +-- ICE attempts a direct path
  `-- relay fallback if needed
```

The compatibility ingress is the price of supporting today's web without requiring browser changes.

## Why DNS should remain the namespace

OpenReach should not create a mandatory `.onion` equivalent.

The user already owns a globally delegated namespace: DNS.

A hostname may simultaneously expose:

- conventional A/AAAA/CNAME/HTTPS records for ordinary clients;
- OpenReach metadata binding the hostname to a service key;
- provider/rendezvous discovery information;
- optional direct-path hints.

For early experimentation, an illustrative binding could use a namespaced TXT record:

```text
_openreach.photos.example.com TXT "v=or1 id=<service-public-key> provider=<provider>"
```

This is not a frozen wire format. SVCB/HTTPS parameters or a future dedicated DNS record may prove better.

The important architectural choice is that DNS remains authoritative for the human-facing name while the service key remains authoritative for cryptographic identity.

## Tor-derived design rule

Tor demonstrates that **location need not be identity, and inbound reachability need not be assumed**.

OpenReach combines that lesson with the ordinary Internet stack:

```text
DNS name
   |
service identity
   |
discovery/rendezvous
   |
ICE/path selection
  / \
direct relay
```

The result is intended to feel like ordinary hosting while retaining the ability to run the actual server on an ordinary local machine.
