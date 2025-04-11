#!/bin/bash
set -e

# This script will test various methods for getting the target platform, tenant and password.

PUT=np-aws-lz-dsh
TUT=greenbox-dev
PFUT="../$PUT.$TUT.pwd"
DUT=.dsh_cli_under_test

PUT_NA=prod-azure-dsh
TUT_NE=nope
PFUT_NAE="../$PUT_NA.nope.pwd"
PWUT_NE=nope

export DSH_CLI_OUTPUT_FORMAT=quiet
export DSH_CLI_LOG_LEVEL=info
export DSH_CLI_VERBOSITY=high

function set_environment {
  rm -rf $DUT
  mkdir $DUT
  mkdir "$DUT/targets"
  export DSH_CLI_HOME=$DUT
  unset DSH_CLI_PLATFORM
  unset DSH_CLI_TENANT
  unset DSH_CLI_PASSWORD
  unset DSH_CLI_PASSWORD_FILE
}

function set_default_platform {
  touch "$DUT/settings.toml"
  echo "default-platform = \"$1\"" >> "$DUT/settings.toml"
}

function set_default_tenant {
  touch "$DUT/settings.toml"
  echo "default-tenant = \"$1\"" >> "$DUT/settings.toml"
}

function create_target {
  dsh --suppress-exit-status target create $1 $2 < "../$1.$2.pwd"
}

echo "target platform from argument"
set_environment
set_default_platform $PUT_NA
export DSH_CLI_PLATFORM=$PUT_NA
dsh service list --platform $PUT --tenant $TUT --password-file $PFUT
export DSH_CLI_HOME=
dsh service list --platform $PUT --tenant $TUT --password-file $PFUT

echo "target platform from environment variable"
set_environment
set_default_platform $PUT_NA
export DSH_CLI_PLATFORM=$PUT
dsh service list --tenant $TUT --password-file $PFUT
export DSH_CLI_HOME=
dsh service list --tenant $TUT --password-file $PFUT

echo "target platform from settings"
set_environment
set_default_platform $PUT
dsh service list --tenant $TUT --password-file $PFUT

echo "target tenant from argument"
set_environment
set_default_tenant $TUT_NE
export DSH_CLI_TENANT=$TUT_NE
dsh service list --platform $PUT --tenant $TUT --password-file $PFUT
export DSH_CLI_HOME=
dsh service list --platform $PUT --tenant $TUT --password-file $PFUT

echo "target tenant from environment variable"
set_environment
set_default_tenant $TUT_NE
export DSH_CLI_TENANT=$TUT
dsh service list --platform $PUT --password-file $PFUT
export DSH_CLI_HOME=
dsh service list --platform $PUT --password-file $PFUT

echo "target tenant from settings"
set_environment
set_default_tenant $TUT
dsh service list --platform $PUT --password-file $PFUT

echo "password from password file argument"
set_environment
export DSH_CLI_PASSWORD=$PWUT_NE
export DSH_CLI_PASSWORD_FILE=$PFUT_NAE
create_target $PUT $TUT $PFUT_NAE
dsh service list --platform $PUT --tenant $TUT --password-file $PFUT
export DSH_CLI_HOME=
dsh service list --platform $PUT --tenant $TUT --password-file $PFUT

echo "password from password file environment variable"
set_environment
export DSH_CLI_PASSWORD=$PWUT_NE
export DSH_CLI_PASSWORD_FILE=$PFUT
create_target $PUT $TUT $PFUT_NAE
dsh service list --platform $PUT --tenant $TUT
export DSH_CLI_HOME=
dsh service list --platform $PUT --tenant $TUT

echo "password from password environment variable"
set_environment
PASSWORD="$(cat $PFUT)"
export DSH_CLI_PASSWORD=$PASSWORD
create_target $PUT $TUT $PFUT_NAE
dsh service list --platform $PUT --tenant $TUT
export DSH_CLI_HOME=
dsh service list --platform $PUT --tenant $TUT

echo "password from keyring"
set_environment
create_target $PUT $TUT $PFUT
dsh service list --platform $PUT --tenant $TUT
