# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 2.x.x   | :white_check_mark: |
| < 2.x.x | :x:                |

## Reporting a Vulnerability

Submit vulnerabilities at [Security Tab](https://github.com/TxtDot/txtdot/security)

## Proxy Isolation

The application shares a network namespace with the `txtdot-firewall` service.
That service installs the egress policy before `txtdot` starts, then keeps the
namespace alive.

Outbound TCP is allowed only after private, loopback, link-local, multicast,
documentation, benchmark, shared, and reserved networks are rejected in the
namespace firewall. This keeps Drova protocols working while blocking access to
local infrastructure and metadata networks.
