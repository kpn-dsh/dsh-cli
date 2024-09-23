# Release notes

## Version 0.0.7

#### New Features

* Added support for:
    * app catalog manifests,
    * application tasks,
    * certificates,
    * kafka proxies,
    * stream topics and
    * volumes.

* New naming schema in API.

#### Bugfixes and Improvements

* Support DSH open api specification version 1.8.0.

## Version 0.0.6

#### New Features

* Added buckets and topics to the dsh api.

#### Bugfixes and Improvements

* Consistent naming convention on the dsh api.
* Moved generation of api code to this crate, for better control and one less dependency.
* Better separation of concerns between engine and dsh api.

## Version 0.0.2

#### Bugfixes and Improvements

* Changed blocking functions to async.

## Version 0.0.1

Initial release.
