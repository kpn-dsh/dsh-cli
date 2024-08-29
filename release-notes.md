# Release notes

## Version 0.0.7

#### New Features

* New naming schema in API.

#### Bugfixes and Improvements

## Version 0.0.6

#### New Features

* Added `replicator` and `regex-filter` processor configurations.
* Added buckets and topics to the dsh api.
* Added cli tool:
    * Supports dsh api and Trifonius engine.
    * List resources.
    * Show resource details.
    * List resource usage (reversed look-up).
* Strict validation on all identifiers.
* Added dry-run for deploy function.

#### Bugfixes and Improvements

* Reduced code duplication in tcli.
* Consistent naming convention on the dsh api.
* Added health check to processors.
* Added icon and tags fields to processor configuration.
* Improved generation of topic id from very long topic names.
* Moved generation of api code to this crate, for better control and one less dependency.
* Better separation of concerns between engine and dsh api.
* Introduced macro for identifiers.
* Improved documentation.
* Added favicon and logo to generated doc.
* Added some design documentation.
* Improved resource model.
* Added more and better example code.
* Improved error handling.
* Better readme files.

## Version 0.0.5

#### New Features

* Added validation to the config files.
* Added validation to the deploy method.
* Added methods to get lists of processor types and resource types.
* Descriptive error messages for configuration files errors.
* Added tags to dsh services and apps to be able to identify them as trifonius components.
* Added design documentation.
* Added dry-run version of deploy() method.
* Added icon to processor config.

#### Bugfixes and Improvements

* Updated some dependency versions.
* Improved and refactored registries.
* Renamed 'application' processor to 'dsh service'.
* More consistent naming of identifiers.
* Small changes to processor configuration file format.
* Seperated processor realizations and processor instances.

## Version 0.0.4

#### Bugfixes and Improvements

* Bugfix.

## Version 0.0.3

#### New Features

* Placeholders for platform dependent values.

#### Bugfixes and Improvements

* Improved model (explicit inbound and outbound junction).
* General code improvements.
* Get dsh topic data from rust sdk instead of api.
* Updated some dependency versions.

## Version 0.0.2

#### Bugfixes and Improvements

* Changed blocking functions to async.
* Updated some dependency versions.

## Version 0.0.1

Initial release.
