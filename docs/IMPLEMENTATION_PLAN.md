# OpenReach Implementation Plan

Status: implementation planning

## 1. Goal

Turn the design into a working, portable system while preserving the architectural invariants:

- baseline operation requires outbound connectivity only;
- DNS remains the stable public namespace;
- service identity is cryptographic and independent of location;
- ICE/STUN/TURN are reused where they fit;
- router and ISP integration are optional optimizations;
- local applications do not need OpenReach-specific code;
- public infrastructure is minimized and replaceable.

## 2. Recommended implementation shape

Use a shared portable core with thin platform shells.

A strong default is:

- **Rust** for protocol, identity, DNS metadata, ICE integration, transport, relay/client logic, and endpoint agent core;
- native wrappers only where the OS requires them;
- a daemon/CLI shell on desktop/server operating systems;
- a library/mobile shell on iOS and Android.

The main reason for a shared core is behavioral consistency. Service identity, candidate handling, protocol framing, signature verification, and relay behavior should not be reimplemented independently for every OS.

## 3. Repository shape

```text
crates/
  openreach-core/        portable protocol and data model
  openreach-identity/    key generation, signing, verification
  openreach-dns/         DNS records, DNS provider/update interfaces
  openreach-ice/         ICE/STUN/TURN adapter
  openreach-transport/   direct and relayed transport abstractions
  openreach-agent/       endpoint publication state machine
  openreach-client/      native client connection state machine
  openreach-relay/       relay/control edge implementation
  openreach-ingress/     ordinary-browser compatibility ingress
  openreach-cli/         desktop/server command line interface

bindings/
  apple/                 Swift/XCFramework integration
  android/               Kotlin/JNI integration

apps/
  macos/
  windows/
  linux/
  ios/
  android/

protocol/
  schemas/
  test-vectors/

docs/
```

The exact layout can change, but portable protocol logic should remain below platform-specific UI/service code.

## 4. Workstreams

### 4.1 Protocol and wire model

Implement first because all other components depend on it.

Required types include:

```text
ServiceIdentity
Publication
ProviderDescriptor
PresenceRegistration
ReachabilityDescriptor
CandidateSet
RelayAssignment
ConnectionOffer
ConnectionAnswer
AuthorizationPolicy
ProtocolVersion
```

Requirements:

- deterministic encoding where signatures are involved;
- explicit version negotiation;
- forward-compatible extension fields;
- replay protection for signed dynamic state;
- expiry timestamps for presence and transient connection data;
- test vectors shared by every platform.

Do not invent custom framing where HTTP, QUIC, ICE, STUN, TURN, or MASQUE already solve the problem.

### 4.2 Identity and key management

Implement stable service identity independently of provider accounts.

Required capabilities:

- create endpoint/service key pairs;
- derive stable service identifiers;
- sign current reachability/presence state;
- verify DNS-bound service identity;
- rotate keys without silently changing service ownership;
- revoke compromised endpoint keys;
- export/import or transfer a service identity to a replacement machine.

Platform storage:

- macOS/iOS: Keychain / Secure Enclave where appropriate;
- Windows: DPAPI/CNG-backed secure storage;
- Linux: file-based encrypted storage initially, with Secret Service/TPM integration later;
- Android: Android Keystore.

The protocol must not require hardware-backed keys because headless Linux systems may not provide them.

### 4.3 DNS identity and compatibility records

Implement DNS as the durable discovery root.

Initial prototype can use namespaced TXT records because they are deployable everywhere.

Example only:

```text
_openreach.photos.example.com TXT
  "v=or1 id=<service-key> provider=<provider-name>"
```

Also support normal compatibility records that route an unmodified browser to ingress.

Implementation should define a provider-neutral DNS update interface and initially support:

1. manual record instructions;
2. RFC 2136 dynamic update where available;
3. one or two common DNS-provider APIs as adapters;
4. delegated OpenReach subzones so endpoints do not need broad registrar credentials.

Investigate SVCB/HTTPS records as the intended standards-based evolution once the required OpenReach parameters are clear.

### 4.4 Endpoint agent

The endpoint agent is the first user-visible core product.

It must:

- publish a local TCP/HTTP endpoint;
- own publication configuration;
- register live presence with selected relay/control edges;
- maintain outbound sessions through NAT/CGNAT;
- gather ICE candidates;
- expose TURN relay candidates when configured;
- perform direct connectivity checks with native clients;
- forward relayed streams to the local application;
- recover after sleep, network changes, DHCP changes, ISP changes, or relay failures;
- expose clear health diagnostics.

Initial CLI:

```text
openreach publish localhost:8080 --as photos.example.com
openreach status
openreach unpublish photos.example.com
```

### 4.5 ICE/STUN/TURN integration

The design position is to use ICE unless a concrete requirement proves it insufficient.

Implementation tasks:

- evaluate a portable ICE implementation suitable for Rust/mobile embedding;
- gather host candidates;
- discover server-reflexive candidates with STUN;
- obtain relay candidates with TURN;
- exchange candidates through OpenReach rendezvous signaling;
- run connectivity checks;
- retain a relay path during direct-path migration;
- expose path metrics and selected candidate pair.

OpenReach should not fork ICE semantics casually.

### 4.6 Relay/control edge

Implement one daemon that can initially combine two low-level roles:

1. **live control/rendezvous** for endpoints and native clients;
2. **relay fallback** for traffic that cannot go direct.

This deliberately avoids requiring a separate hosted discovery product.

Required capabilities:

- authenticate a service identity;
- maintain live endpoint presence from outbound sessions;
- multiplex many publications over one endpoint connection;
- exchange ICE offers/answers/candidates;
- issue short-lived connection/relay credentials;
- relay streams when direct ICE connectivity fails;
- enforce quotas and abuse limits;
- expose health and provider metadata;
- support multiple relay instances/regions.

The first version may combine control and data plane. Later versions should permit separating them operationally without changing the protocol.

### 4.7 Ordinary-browser compatibility ingress

Required for:

```text
https://photos.example.com
```

from an unmodified browser.

Responsibilities:

- accept normal HTTP/HTTPS connections;
- route by hostname/SNI;
- find the currently registered service through the relay/control plane;
- bridge the connection to its outbound endpoint session;
- return a meaningful unavailable response when the endpoint is offline;
- support endpoint-terminated TLS if practical, otherwise clearly isolate edge-terminated TLS as a compatibility mode.

This component is logically distinct from native OpenReach transport even if the initial deployment runs them together.

### 4.8 Native OpenReach client

A native client is necessary only for clients that should benefit from direct ICE connectivity, private services, or service-key verification beyond normal browser semantics.

It must:

- resolve normal DNS name;
- discover and verify OpenReach metadata;
- locate a current provider/relay edge;
- request live rendezvous;
- exchange ICE candidates;
- choose direct or relay transport;
- verify service identity end to end;
- expose a stream/socket-like API to applications.

Desktop implementations may initially expose a local proxy so existing applications can use native OpenReach without linking a library.

### 4.9 Authorization and private services

After public connectivity works:

- service ACL model;
- client identities;
- invitation/bootstrap flow;
- passkey/OIDC gateway option for browser traffic;
- service-key-based authentication for native clients;
- revocation and expiry.

Do not turn OpenReach into a full-LAN VPN by default. Authorization should remain publication-oriented.

### 4.10 Observability and diagnostics

A consumer networking system must explain what it is doing.

Expose states such as:

```text
Published: photos.example.com
Identity: verified in DNS
Endpoint: online
Path: direct IPv6
Fallback relay: us-west-1
Browser ingress: healthy
Certificate: valid, renews in 41 days
```

Diagnostics should distinguish:

- DNS failure;
- identity mismatch;
- endpoint offline;
- STUN failure;
- TURN allocation failure;
- direct ICE failure;
- relay unavailable;
- ingress unavailable;
- local application unavailable.

## 5. Platform support

The protocol core must be portable from the first implementation.

### 5.1 macOS

Ship:

- arm64 binary;
- x86_64 binary while Intel macOS remains relevant;
- preferably a universal application/package artifact;
- launchd service support;
- CLI plus minimal menu/status UI later.

### 5.2 Linux

Avoid tying support to one distribution.

Target at minimum:

- x86_64-unknown-linux-gnu;
- aarch64-unknown-linux-gnu;
- x86_64-unknown-linux-musl static/mostly-static artifact where dependencies permit;
- aarch64-unknown-linux-musl where practical.

Provide:

- standalone tarballs;
- systemd unit example but do not require systemd;
- container image for relay/control infrastructure;
- no dependency on a desktop environment.

"Any Linux" means protocol/runtime portability rather than promising every historical libc/kernel combination. Publish documented minimum kernel/glibc requirements and provide musl builds to cover small/server distributions broadly.

### 5.3 Windows

Target:

- x86_64-pc-windows-msvc first;
- aarch64-pc-windows-msvc as a release target;
- Windows Service installation;
- CLI and later tray/status UI;
- no WSL dependency.

### 5.4 iOS

Do not assume iOS can behave like an always-running home server. Background execution restrictions make that an unsuitable baseline endpoint host.

Provide iOS support primarily as a **native OpenReach client** through a shared core packaged as an XCFramework and Swift API.

Support:

- arm64 device;
- simulator architectures required by current Xcode;
- DNS/service identity resolution;
- rendezvous;
- ICE/TURN transport;
- private-service authentication;
- foreground connection lifecycle.

Only add endpoint/publication mode if a specific iOS background-capable use case is validated.

### 5.5 Android

Provide the native client as an Android library/application using the shared core through JNI/Kotlin bindings.

Target initially:

- arm64-v8a;
- x86_64 emulator;
- consider armeabi-v7a only if demand justifies it.

Android can support longer-running services more flexibly than iOS, but endpoint publication should still be a separate product decision because mobile power/background policies are hostile to permanent service hosting.

## 6. Build and CI matrix

Every pull request should compile the shared core on:

```text
Linux x86_64
Linux aarch64 (cross build acceptable)
macOS arm64
macOS x86_64
Windows x86_64
Windows arm64 (cross/compile check initially)
Android arm64
Android x86_64
Apple mobile library / iOS simulator target
```

Release CI should produce signed/notarized artifacts where platform policy requires it.

Tests should be layered:

1. pure protocol/unit tests on every platform;
2. identity/DNS test vectors;
3. simulated NAT/ICE integration tests on Linux CI;
4. relay/endpoint end-to-end tests;
5. cross-platform endpoint/client interop matrix;
6. real-network periodic tests for CGNAT, IPv6, UDP-blocked, and TCP-only environments.

## 7. Implementation phases

### Phase 0: repository and protocol skeleton

- Rust workspace;
- protocol crate;
- identity crate;
- CLI skeleton;
- CI matrix for Linux/macOS/Windows;
- mobile core compile targets;
- deterministic protocol test vectors.

Success: the same protocol/identity core builds on every target platform.

### Phase 1: guaranteed relayed publication

- endpoint agent;
- combined relay/control daemon;
- generated OpenReach test hostname;
- persistent outbound endpoint connection;
- HTTP stream bridging;
- reconnect logic;
- basic metrics.

Success:

```text
openreach publish localhost:8080
```

works from a home network behind NAT without router changes.

### Phase 2: user-controlled DNS and HTTPS

- custom domains;
- service-key DNS binding;
- DNS update/delegation flow;
- ACME/certificate handling;
- ordinary browser ingress;
- endpoint-unavailable handling.

Success: an ordinary browser can visit a user-owned hostname served by a local machine behind CGNAT.

### Phase 3: native direct connectivity

- STUN;
- TURN;
- ICE candidate exchange;
- native desktop client;
- direct-path selection and relay fallback;
- path diagnostics.

Success: native clients automatically bypass the relay when a viable direct path exists.

### Phase 4: mobile clients and private services

- iOS library/app;
- Android library/app;
- private service identities/ACLs;
- invitation/bootstrap flow;
- mobile interoperability tests.

### Phase 5: provider independence and federation

- documented provider protocol;
- multiple relay/control providers;
- provider discovery through DNS;
- failover;
- self-hostable relay/control image;
- interoperability test suite.

## 8. First implementation milestone

The first code PR should be intentionally boring infrastructure:

1. choose Rust workspace and MSRV/toolchain policy;
2. create `openreach-core` and `openreach-identity`;
3. create endpoint and relay CLI skeletons;
4. establish GitHub Actions builds across Linux, macOS, and Windows;
5. add iOS and Android library compile checks;
6. define service identity and signed-presence test vectors;
7. add an end-to-end localhost relay smoke test.

Only after that baseline is green should NAT traversal and DNS-provider automation be layered in.
