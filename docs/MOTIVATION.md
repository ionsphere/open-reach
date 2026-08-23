# Motivation

## The missing consumer Internet primitive

A consumer can cheaply buy a domain name and own enough compute, storage, and bandwidth to run useful services locally. Yet making one of those services reachable from anywhere still usually requires either expert networking work or moving the service onto infrastructure owned by somebody else.

The missing primitive is not local hosting. It is **simple, secure Internet reachability for a named local service**.

A user should be able to say:

```text
Publish http://localhost:8080 as photos.example.com
```

and have the system make the best available path from the public Internet to that local service.

The user should not need to understand static IPs, NAT, CGNAT, IPv6 prefix delegation, port forwarding, firewall rules, STUN, ICE, TURN, reverse proxies, ACME, or the particular capabilities of their router and ISP.

## Architectural regression

Dynamic addressing is inconvenient but solvable. DNS can follow a changing address.

The deeper problem is that many consumer Internet connections are no longer reliably reachable from the outside. A household may be behind one or more NAT layers, including carrier-grade NAT, while its router or ISP exposes no dependable inbound mapping mechanism.

The common modern answer is to move the application into the cloud or keep a cloud tunnel provider permanently in the path.

OpenReach starts from a different principle:

> The local machine remains the server. Public infrastructure provides identity discovery, rendezvous, compatibility ingress, and fallback transport only to the degree required to make that server reachable.

## Desired user experience

The consumer abstraction is **publishing a service**, not forwarding a port.

```text
Local service: http://localhost:8080
Name:          photos.example.com
Visibility:    public

[ Publish ]
```

After that, OpenReach owns the networking machinery required to keep that name working.

## Core requirements

### 1. Works with outbound connectivity only

The baseline supported environment may have:

- a commodity consumer router;
- no manual configuration;
- no public IPv4;
- CGNAT;
- blocked unsolicited inbound traffic;
- no UPnP, NAT-PMP, or PCP support;
- no ISP integration.

If the endpoint can make ordinary outbound Internet connections, OpenReach must still be able to publish the service through a relay path.

### 2. Direct when possible, relayed when necessary

Relay reachability is the correctness baseline. The system should opportunistically discover shorter paths through native IPv6, public IPv4, ICE connectivity checks, NAT traversal, or optional router-assisted mappings.

A successful direct path should remove bulk traffic from relay infrastructure automatically.

### 3. DNS stays the public namespace

OpenReach should not require a new `.onion`-style namespace.

A normal hostname such as:

```text
photos.example.com
```

remains the stable human-facing service name.

DNS may carry both compatibility records for ordinary clients and metadata binding the hostname to a stable cryptographic service identity.

### 4. Identity is independent of current location

The service should have a stable cryptographic identity even while these change:

- local IP address;
- public IP address;
- relay operator;
- ISP;
- physical machine;
- network location.

DNS provides the authoritative binding between the human-readable name and that service identity.

### 5. Reuse Internet standards

OpenReach should prefer existing standards over project-specific equivalents.

In particular:

- DNS for naming and discovery;
- ACME for certificate issuance;
- ICE for candidate gathering and path selection where appropriate;
- STUN for reflexive-address discovery;
- TURN when its relay semantics fit;
- QUIC/HTTP and MASQUE-family mechanisms where they fit transport and proxying needs.

### 6. Provider independence

Relay and rendezvous infrastructure should be replaceable. A user should be able to change providers without changing the application URL or moving application state.

### 7. Local compute and data stay local

Public infrastructure must not require application execution or persistent application storage.

### 8. Secure by default

Only explicitly published services become reachable. TLS, scoped credentials, revocation, rate limits, and authenticated private publication are part of the system rather than optional afterthoughts.

## Non-goals

OpenReach is not initially intended to:

- become a cloud compute platform;
- replace DNS;
- replace browsers;
- require new router firmware;
- require ISP deployment;
- guarantee a direct path when the surrounding network makes one impossible;
- invent a new connectivity protocol where ICE, TURN, QUIC, HTTP, or MASQUE already solve the problem well.

## Success criterion

OpenReach succeeds when this becomes ordinary:

> I bought a domain, selected a program running on my own computer, clicked Publish, and now it is securely reachable from anywhere.
