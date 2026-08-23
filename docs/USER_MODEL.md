# OpenReach User Model

Status: product and setup model

## 1. Principle

OpenReach is local software first.

The normal user should not be asked to deploy a VPS, create an Azure/AWS project, run Kubernetes, or create an OpenReach cloud account.

The user already supplies the two things the system fundamentally needs:

1. a machine running useful local applications;
2. authority over a public DNS name.

OpenReach connects those two.

## 2. User-owned pieces

### 2.1 Local applications

The user creates, installs, or finds applications that already run locally.

Examples:

```text
http://127.0.0.1:3000
http://192.168.1.20:8123
localhost:25565
```

Applications do not need to implement OpenReach.

OpenReach treats them as local publication targets.

### 2.2 Domain name

The user owns or controls a normal Internet domain or delegated subdomain.

Examples:

```text
example.com
home.example.com
mybox.example.net
```

OpenReach does not create a mandatory proprietary namespace.

### 2.3 DNS hosting

The user chooses a normal DNS hoster.

OpenReach receives only enough authority to manage the records required for selected publications.

The DNS provider is therefore the globally available durable infrastructure already present in the user's setup.

### 2.4 OpenReach endpoint

The user launches OpenReach on the same machine as the applications, or on another machine that can reach them over the local network.

OpenReach owns:

- service identity keys;
- publication configuration;
- DNS synchronization;
- address and NAT capability detection;
- direct reachability checks;
- optional ICE/STUN behavior;
- TLS/certificate automation where applicable;
- local reverse proxying;
- optional integration with third-party rendezvous/relay/ingress providers.

## 3. First-run setup

The intended setup should read like product configuration, not network administration.

### Step 1: connect DNS

The user chooses their DNS provider and authorizes OpenReach.

Preferred mechanisms are scoped OAuth, zone-limited API tokens, delegated sub-zone credentials, or scoped dynamic-DNS credentials.

OpenReach validates that it can create and update records without receiving broader account access than necessary.

### Step 2: discover or add local applications

OpenReach may discover likely local listeners and applications, while also allowing explicit configuration.

Example:

```text
Detected locally:

[ ] http://localhost:3000   Photos
[ ] http://localhost:8123   Home Assistant
[ ] tcp://localhost:25565   Game server

[ Add manually ]
```

Nothing is exposed until the user selects it.

### Step 3: assign names and policy

For each selected application:

```text
Local:       http://localhost:3000
Public name: photos.example.com
Access:      public
```

or:

```text
Local:       http://localhost:8123
Public name: home.example.com
Access:      private / approved OpenReach clients
```

### Step 4: establish service identity

OpenReach creates a cryptographic service identity and binds it to the DNS name.

The user should not need to manage keys manually.

### Step 5: probe reachability

OpenReach determines what the current network allows:

- public IPv6;
- public IPv4;
- explicit mapping through PCP/NAT-PMP/UPnP;
- STUN server-reflexive reachability;
- direct OpenReach-native connectivity;
- whether ordinary browser access is directly possible.

### Step 6: publish DNS

OpenReach creates or updates the records needed for the best currently available path.

### Step 7: report actual capability

The UI reports the resulting state rather than claiming universal reachability when the network cannot provide it.

Example:

```text
photos.example.com

Reachability: Direct
Browser access: Yes
Path: IPv6
Third-party relay: None
```

or:

```text
home.example.com

Reachability: OpenReach clients only
Browser access: No
Reason: inbound traffic blocked by current NAT/firewall
Third-party relay: None

Optional: configure rendezvous/ingress provider
```

## 4. No mandatory OpenReach account

The local service identity plus DNS control should be sufficient to establish ownership.

An OpenReach software installation should not require registration with an OpenReach-operated service.

Optional external providers may have their own commercial/account models, just as DNS hosters and registrars do today.

OpenReach should store provider configuration locally and describe it in portable configuration where safe.

## 5. OpenReach should be useful before any provider is configured

The product must attempt direct publication first.

A user with a workable direct path should be finished after:

```text
application + domain + DNS authorization + OpenReach
```

No fourth infrastructure purchase should be introduced merely because the implementation prefers a centralized design.

## 6. Optional capabilities are selected by requirement

If direct connectivity is insufficient, OpenReach can identify exactly what is missing.

### Rendezvous only

Needed when both OpenReach endpoints could communicate directly after exchanging transient candidate information, but cannot establish that exchange through static DNS alone.

The provider carries signaling only, not application traffic.

### Relay

Needed when the network topology cannot establish a direct path.

The provider carries application traffic and therefore incurs bandwidth cost.

### Browser compatibility ingress

Needed when the endpoint is not conventionally reachable but arbitrary unmodified browsers must still open the service URL.

The provider exposes a normal public Internet endpoint and bridges to the local service.

These are separate services and should remain independently replaceable.

## 7. Example zero-hosting lifecycle

A user has a local photo application and buys `familyphotos.net`.

They install OpenReach and authorize DNS updates for that zone.

OpenReach discovers that the home has globally routable IPv6 and can accept the publication safely.

It creates:

```text
photos.familyphotos.net
```

with the required DNS identity and connectivity records, obtains/maintains TLS, and forwards incoming traffic to the local photo app.

The resulting system is:

```text
browser/client
      |
      | Internet
      v
home OpenReach endpoint
      |
      v
local photo application

DNS hosting is the only pre-existing hosted service.
```

OpenReach operates no cloud component in the path.

## 8. Example constrained-network lifecycle

The same user later moves the machine to a network behind restrictive CGNAT.

OpenReach detects that previously direct reachability is gone.

It does not silently upload the application or enroll the user into an OpenReach cloud.

Instead it reports:

```text
Direct browser reachability unavailable.

Possible capabilities:
- native OpenReach connection: test/available depending on traversal
- rendezvous provider: optional
- relay provider: required if direct traversal fails
- browser ingress provider: required for ordinary browser compatibility
```

The user chooses whether the additional capability is worth acquiring.

## 9. Configuration model

Conceptually, local configuration is:

```text
OpenReachConfig {
    dns_authority
    service_identities
    publications[]
    network_policy
    optional_stun_services[]
    optional_rendezvous_providers[]
    optional_relay_providers[]
    optional_ingress_providers[]
}

Publication {
    local_target
    dns_name
    service_identity
    access_policy
    desired_compatibility
}
```

This configuration belongs to the user and should be exportable without depending on a central account database.

## 10. Product promise

The user-facing promise should be precise:

> Give OpenReach a local application and control of a DNS name. OpenReach will publish the strongest secure Internet reachability your existing connection can provide, without requiring OpenReach-hosted infrastructure. If the topology makes additional public infrastructure unavoidable, OpenReach will identify that requirement and let you choose a compatible provider.
