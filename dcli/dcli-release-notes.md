# Release notes

## Version 0.0.7

#### New Features

* Renamed tool to `dcli` (**D**SH **C**ommand **L**ine **I**nterface).
* Added support for:
    * Display of app catalog manifests.
    * Listing application tasks.
    * Listing certificates with reversed look-up.
    * Listing Kafka proxies.
    * Creating and deleting secrets.
    * Functions for stream topics (disabled by default).
    * Listing volumes with reversed look-up.
* Improved support for:
    * Reversed look-up of certificates, secrets and volumes.
* Features to enable/disable support for streams and Trifonius.

#### Bugfixes and Improvements

* Support DSH open api specification version 1.8.0.
* Added [tabled](https://github.com/zhiburt/tabled) crate for better presentation of the output.
* Improved internal application design.

## Version 0.0.6

Added `tcli` tool (**T**rifonius **C**ommand **L**ine **I**nterface).
