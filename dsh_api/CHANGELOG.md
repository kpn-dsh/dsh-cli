# Changelog

All notable changes to the DSH Api Client project will be documented in this file.

## [Unreleased]

## [0.1.0] - 2024-10-29

### Added

* Functions for app catalog manifests.
* Functions for application tasks.
* Functions for certificates.
* Functions for kafka proxies.
* Functions for stream topics.
* Functions for volumes.

### Changed

* New naming schema in API.

### Fixes

* Support DSH open api specification version 1.8.0.

### Removed

All code and dependencies for the Trifonius engine are moved to their own project.

## [0.0.6] - 2024-08-20

### Added

* Functions for buckets.
* Functions for topics.

### Fixes

* Consistent naming convention on the dsh api.
* Moved generation of api code to this crate, for better control and one less dependency.
* Better separation of concerns between Trifonius engine and dsh api.
