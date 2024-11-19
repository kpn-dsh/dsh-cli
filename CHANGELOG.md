# Changelog

All notable changes to the dcli tool project will be documented in this file.

## [Unreleased]

### Security

* Credentials are now stored in the keyring (only supported for OsX).

## [0.2.0]

### Added

* New subject for exposed metrics.
* Save settings and targets in application directory.
* Capability to manage settings and targets from within the tool.
* Capability to generate autocomplete files.

### Changed

* Added prompts for user input.

### Removed

The dsh-api subproject is removed and is now in its own repo at
[github.com/kpn-dsh/dsh-api](https://github.com/kpn-dsh/dsh-api).

## [0.1.0] - 2024-10-29

### Added

* Capabilities for app catalog manifests.
* Capabilities for application tasks.
* Capabilities for certificates (including reversed look-up).
* Capabilities for Kafka proxies.
* Capabilities for secrets (including reversed lookup, creating and deleting).
* Capabilities for stream topics (disabled by default).
* Capabilities for volumes (including reversed lookup).
* Capabilities for images (including reversed lookup).
* Support for regular expressions for (reversed) lookup functions.

### Changed

* Renamed tool to `dcli` (**D**SH **C**ommand **L**ine **I**nterface).
* New naming schema in API.
* Added number of instances to usage tables.
* Improved support for reversed lookup of certificates, secrets and volumes.
* Improved support for manifests.

### Fixes

* Support DSH open api specification version 1.8.0.
* Added [tabled](https://github.com/zhiburt/tabled) crate for better presentation of the output.
* Improved internal application design.

### Removed

All capabilities, code and dependencies for Trifonius are removed.
