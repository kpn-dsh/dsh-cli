# Security Policy

This document outlines security procedures and general policies for the `dsh-cli` project.

- [Security Policy](#security-policy)
    - [Supported Versions](#supported-versions)
    - [Known dependency vulnerabilities](#known-dependency-vulnerabilities)
    - [Reporting a Vulnerability](#reporting-a-vulnerability)
        - [Reporting Process](#reporting-process)
        - [Handling and Resolution](#handling-and-resolution)

## Supported Versions

The following versions of this project are currently being supported with security updates.

| Version | Supported          |
|---------|--------------------|
| 0.4.0   | :white_check_mark: |
| 0.3.0   | :x:                |
| 0.2.0   | :x:                |
| 0.1.0   | :x:                |

## Known dependency vulnerabilities

[![dependency status](https://deps.rs/repo/github/kpn-dsh/dsh-cli/status.svg)](https://deps.rs/repo/github/kpn-dsh/dsh-cli).

## Reporting a Vulnerability

If you have found a vulnerability or bug, you can report it to wilbertschelvis@kpn.com or
unibox@kpn.com.

### Reporting Process

When reporting a vulnerability, please include the following information:

- Description of the vulnerability
- Steps to reproduce (if applicable)
- Affected versions
- Potential impact of the vulnerability
- Any additional information of context

### Handling and Resolution

When a vulnerability is reported, the following process will be followed:

- The vulnerability will be evaluated by the maintainers and aknowledged within 3 business days
- The maintainers will determine the severity of the vulnerability and the impact on the project
- The maintainers will update the issue with the above information
- The maintainers will create a fix for the vulnerability
- The maintainers will release a new version with the fix and post a security advisory on
  the [ GitHub repo ]( https://github.com/kpn-dsh/dsh-cli ) with the following
  information:
    - Description of the vulnerability
    - Affected versions
    - Fixed versions
    - Severity of the vulnerability
    - Potential impact of the vulnerability
    - Any additional information of context

We appreciate contributions to our security and, where appropriate, will offer credit in release
notes. 
