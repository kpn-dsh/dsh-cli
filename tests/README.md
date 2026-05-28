# Integration tests

The `tests` directory contains some shell scripts that will run a large number of commands
in sequence. This is not a full test, but it will catch many bugs which have to do with the
command line part of the program.

You can run the test as you would normally run a shell script, for example from the project
root you can run:

```shell
> tests/test_environment_variables.sh
```

Note that some tests require that you are logged in (using single-sign-on) on platform
`np-aws-lz-dsh` with authorization for tenant `greenbox-dev`, prior to running the tests.
Other scripts might require you to enter the platform and/or tenant manually, or to provide
credentials via you computer's keyring. Also, many tests act on the existing/deployed resources
for this platform and tenant, and this can change over time. Because of this the tests can fail.
If this is the case, please check the test scripts carefully and change the resources in the
tests files to the new situation.

## `test_commands_sso.sh`

This script will run many commands that all must succeed and will print output to `stdout`
(and possibly `stderr`). Commands that would result in changes on the DSH platform
all have the `--dry-run` argument included, so that no actual changes will be made.

For these tests to run successfully you need to be logged in via single-sign-on on the
`np-aws-lz-dsh`platform and you need to be authorized for the `greenbox-dev` tenant. The
tests also assume the existence of a number of subjects and resources to be available/deployed
for this tenant on the platform.

You can limit the tests to only a single subject by providing the subject as an
argument for the script. For example, if you only want to run the tests for the `secret`
subject, use:

```shell
> tests/test_commands_sso.sh secret
```

## `test_environment_variables.sh`

This script will run commands related to environment variables.

## `test_erroneous_commands.sh`

This script will run erroneous commands and print the error message to `stderr`.
All commands must produce a controlled error message and never panic.

## `test_erroneous_manage_commands.sh`

This script will run erroneous manage commands and print the error message to `stderr`.
All commands must produce a controlled error message and never terminate in panic.

## `test_manage_commands.sh`

This script will run commands that are only available when the `manage` feature
was enabled when the tool was built or installed.
All commands must succeed and print output to `stdout` (and possibly `stderr`).
Commands that would result in changes on the DSH platform
all have the `--dry-run` argument included, so that no actual changes will be made.

## `test_manifests.sh`

This script will run commands related to manifests.

## `test_manifests_all_versions.sh`

This script will run commands related to all versions of the manifests.

## `test_platform_open_commands.sh`

This script will run some `dsh platform open` commands which will try to open DSH
resources and web applications. If you run this script, make sure that you are
already logged in to the Console for the proper platform and tenant.
If successful, you will have some open tabs in your browser.

## `test_robot.sh`

This script will run commands related to the robot authentication method.

## `test_styling.sh`

This script will run some commands with different color and styling settings.
This allows for a visual check of the proper rendering of tables and textual console output
to `stdout` and `stderr`.

## `test_robot_authentication.sh`

This script will run the `dsh service list` command with different ways of providing the
target platform, tenant and password.

[Publish &#x2192;](publish.md)
