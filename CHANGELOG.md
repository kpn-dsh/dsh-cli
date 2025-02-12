# Changelog

All notable changes to the `dsh` tool project will be documented in this file.

## [Unreleased]

## [0.5.0] - 2025-02-07

### Fixes

* Saving targets works properly again.

### Added

* Improved error handling for bad requests.
* Features (disabled by default):
    * `appcatalog` - Controls availability of app catalog operations.
    * `manage` - Controls availability of manage operations.
    * `robot` - Controls availability of robot operation.
* Set and unset default target.
* Set and unset all settings via the tool.
* Added csv output format.
* Added flag to hide headers.
* Added basic token capability.

### Removed

* Group and user id are not needed anymore.
* Feature `actual` is removed. Its capabilities are now all enabled.

### Changed

* Upgraded `dsh_api` dependency to 0.5.0.
* Autocomplete flag now hidden.

## [0.4.0] - 2025-01-31

### Added

* Support for Windows.
* Support DSH open api specification version 1.9.0.
* Added generic api calls.
* Added capabilities for piping and redirecting.
* Added logging.
* Secret update capability.
* Token capabilities.
* Capability to open platform web applications.

### Changed

* Improved user interface.
* Re-organized documentation.
* Embedded logo and favicon in generated docs.

## [0.3.0] - 2024-12-20

### Added

* User can specify the format of the output (csv, json, table, toml, yaml).
* Output will now be printed to stdout, while logging, error messages and metadata
  will be printed to stderr.
* More control over the generated output.
* Capability to do a dry-run, without actually changing anything on the DSH.
* Flag to enforce changes.
* Capability to print the open-api specification.
* Capability to list and use platform parameters.

### Changed

* Renamed the binary/executable from `dcli` to `dsh`.
* Changed environment variables.
* Changed dsp_api dependency from `git` to `crates.io`.

## [0.2.0] - 2024-11-29

### Added

* New subject for exposed metrics.
* Save settings and targets in application directory.
* Capability to manage settings and targets from within the tool.
* Capability to generate autocomplete files.

### Changed

* Credentials are now stored in the keyring (only supported for OsX).
* Added prompts for user input.
* System secrets no longer listed by default.

### Removed

* The dsh-api subproject is removed and is now in its own repo at
  [github.com/kpn-dsh/dsh-api](https://github.com/kpn-dsh/dsh-api).
* Removed redundant flags.

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
