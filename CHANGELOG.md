# Changelog

All notable changes to the `dsh` tool project will be documented in this file.

## [Unreleased]

## [0.7.3] - YYYY-MM-DD

### Added

* Capability to create, delete, list and show managed tenants.
* Capability to set limits and enable services for managed tenants.
* Capability to create, delete, list and show managed streams.
* Additional properties when creating topics/streams (`compression.type`,
  `delete.retention.ms`, `message.timestamp.type`, `retention.bytes`,
  `retention.ms` and `segment.bytes`).
* Capability to export service configuration file.
* Capability to update service from configuration file.
* Capability to fetch a token and copy it directly to the clipboard.
* Capability to list and explain the used environment variables.
* Some more colors.

### Changed

* Capability to list vhosts improved.
* Change to `service update` capability so it can also update from a configuration file.
* Added styling for error messages.
* Upgraded dependency `dsh_api` to version `0.7.1`,
  for patch on error in open api specification and smaller package size.
* Added kafka properties to show topic command.

### Fixes

* Fixed bug when printing lists with some output formats.

### Removed

* Removed `token show` command.
* Short flag (`-V`) to show the tools version number.
* Settings for `dsh_sdk` logging level.

## [0.7.2] - 2025-04-11

### Fixes

* Fixed bug where target configuration was checked while it shouldn't.
* Fixed bug with verbosity level.

## [0.7.1] - 2025-04-07

### Added

* Export app catalog manifest.
* Show settings and environment variables in help text.
* Configurable coloring and styling.
* Flag to suppress exit status for subject functions.

### Changed

* Ask for confirmation on service duplicate and edit capabilities.
* Code simplifications.

### Fixes

* Fixed bug with default output format.

## [0.7.0] - 2025-03-06

### Added

* Capability to edit service configurations.
* Capability to duplicate service configurations.
* Capability to create topics.

### Changed

* Renamed new and deploy subcommands to create.
* Display help when no commands or arguments are provided.
* Api selector commands sorted alphabetically.
* Improved help text for api commands.
* Dependency to `dsh_api` crate to version `0.6.0`.
* Deleted feature `appcatalog`, which is now always enabled.

## [0.6.0] - 2025-02-28

### Added

* Dedicated functions to delete, deploy, restart, start, stop and update DSH services.
* Single command to set default platform and tenant.

### Changed

* Renamed subject `application` to `service`. This results in changed commands and options.
* Simplified json output.
* Better rendering of empty results.
* Dependency to `dsh_api` crate to version `0.5.2`.

### Fixes

* Fixed bug with duplicate ids in json output.
* Lists of identifiers and lists derived from them are sorted again.

## [0.5.0] - 2025-02-20

### Fixes

* Saving targets works properly again.

### Added

* Improved error handling for bad requests.
* Improved documentation.
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

* Upgraded `dsh_api` dependency to 0.5.1.
* Autocomplete flag now hidden.
* Improved logging.
* Renamed create capabilities to new.

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
