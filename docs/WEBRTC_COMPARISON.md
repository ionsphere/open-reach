# OpenReach and WebRTC

WebRTC already standardizes much of the difficult connectivity machinery that OpenReach needs. OpenReach should treat that work as infrastructure to reuse, not merely as inspiration.

## Where the architectures overlap

A WebRTC peer connection typically gathers several classes of candidates and uses ICE connectivity checks to choose a working path:

```text
Peer A                               Peer B
  |                                    |
  | host candidates                    | host candidates
  | server-reflexive candidates        | server-reflexive candidates
  | relay candidates                   | relay candidates
  |                                    |
  +---------- ICE checks --------------+
                 |
            best path wins
```

The OpenReach equivalent is:

```text
client                                local endpoint
  |                                      |
  | native/direct candidates             | native/direct candidates
  | STUN-derived candidates               | STUN-derived candidates
  | relay candidates                      | relay candidates
  |                                      |
  +---------- ICE checks ----------------+
                 |
            best path wins
```

The mapping is close:

| WebRTC concept | OpenReach role |
| --- | --- |
| ICE agent | path discovery and selection |
| STUN | public/reflexive address discovery |
| ICE candidate | reachability candidate |
| TURN | guaranteed relay candidate |
| signaling | service rendezvous/control |
| peer identity | DNS-bound service identity |

## The key difference: session versus service

WebRTC answers roughly:

> How can two participating peers establish a session right now?

OpenReach answers:

> How can a persistent named service be reached over time as though it were an ordinary Internet host?

That adds requirements WebRTC itself intentionally leaves to applications:

- stable service naming;
- persistent service identity;
- DNS binding;
- publication lifecycle;
- browser-compatible ingress;
- provider-independent relay discovery;
- endpoint availability across reconnects and network changes.

## Signaling remains outside ICE

ICE does not define how two parties discover one another or exchange the application-level information required to begin connectivity checks. WebRTC applications therefore provide signaling separately.

OpenReach's control/rendezvous plane plays that role, but it is service-oriented rather than session-oriented.

A service publication can be thought of as continuously advertising:

```text
service identity K
current endpoint presence
current candidate set
relay availability
policy/authorization metadata
```

## Ordinary browsers are still different

A WebRTC-capable browser can participate in ICE when JavaScript explicitly creates a peer connection.

That does not mean normal navigation to:

```text
https://photos.example.com
```

will perform OpenReach rendezvous and ICE before opening HTTP.

The ordinary browser still expects DNS followed by a normal TCP/QUIC/TLS connection. Therefore OpenReach needs a compatibility ingress path whenever the local endpoint is not conventionally reachable.

This creates two modes:

```text
ordinary browser
      |
      v
public compatibility ingress
      |
      v
OpenReach endpoint
```

and, for an OpenReach-aware client:

```text
OpenReach client
      |
      +-- rendezvous
      +-- ICE connectivity checks
      |
      +-- direct path if possible
      `-- relay fallback otherwise
```

## Should OpenReach use ICE literally?

The default design position should be:

> Use ICE unless a concrete OpenReach requirement demonstrates that ICE is insufficient.

Reasons:

- mature NAT traversal semantics;
- standardized candidate priority and connectivity checking;
- existing STUN/TURN ecosystem;
- extensive implementation experience across hostile networks;
- avoids creating another subtly incompatible NAT traversal protocol.

OpenReach may still require extensions or a narrower profile for persistent service use, but those should be identified explicitly.

## TURN and OpenReach relay semantics

TURN is not inherently a video protocol. It provides a relay transport address when direct peer connectivity is unavailable.

That makes it a candidate component of OpenReach's native-client relay path.

However, a persistent public service has somewhat different requirements from a temporary interactive peer session:

- long-lived endpoint presence;
- many unrelated incoming clients;
- multiplexing many service streams;
- HTTP/TLS hostname routing;
- ordinary-browser compatibility;
- provider federation;
- low idle cost;
- abuse controls appropriate for public ingress.

Therefore OpenReach should evaluate two separate questions:

1. Can TURN be reused directly for native OpenReach client-to-endpoint relay connectivity?
2. Should browser-compatible ingress use TURN, or is QUIC/HTTP/MASQUE-style forwarding a better fit?

The answers do not need to be the same.

## Design consequence

OpenReach should not describe its connectivity subsystem as merely "ICE-inspired". The architecture should be decomposed into:

```text
DNS + service identity
        |
service discovery / rendezvous
        |
       ICE
     /     \
 direct   relay
          |
     TURN or other
     standardized transport
```

OpenReach's novel layer is primarily the persistent named-service model, DNS-bound identity, publication semantics, and compatibility bridge into this connectivity machinery.
