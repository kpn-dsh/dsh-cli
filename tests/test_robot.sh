#!/bin/bash
set -e

# Run commands using the robot authentication method
#
# This script tests the various robot authentication method for dsh.
# You will be prompted for some values during the process and
# you will be asked to authenticate with your local keyring app.
# Enter platform 'nplz' and tenant 'greenbox-dev' when requested.
#
# For this test to run successfully the following is expected:
#
# * User is not logged in via single-sign-on authentication method.
# * Tenant 'greenbox-dev' exists at platform 'np-aws-lz-dsh'.
# * Secret 'test' exists.
# * Password file 'np-aws-lz-dsh.greenbox-dev.pwd' exists and contains.
#   the robot password for tenant 'greenbox-dev' at platform 'np-aws-lz-dsh'.
# * The keyring contains an entry for tenant 'greenbox-dev' at platform
#   'np-aws-lz-dsh'.

PLATFORM_UNDER_TEST="np-aws-lz-dsh"
TENANT_UNDER_TEST="greenbox-dev"
PASSWORD_DIRECTORY="$(dirname "$0")"
SECRET_UNDER_TEST="cli-test"

PLATFORM="$PLATFORM_UNDER_TEST"
TENANT="$TENANT_UNDER_TEST"
PASSWORD_FILE="$PASSWORD_DIRECTORY/$PLATFORM.$TENANT.pwd"
PASSWORD="$(cat "$PASSWORD_FILE")"

function unset_all {
  unset DSH_CLI_PLATFORM
  unset DSH_CLI_TENANT
  unset DSH_CLI_PASSWORD
  unset DSH_CLI_PASSWORD_FILE
}

export DSH_CLI_LOG_LEVEL=info
export DSH_CLI_LOG_LEVEL_API=info

BASE_CMD="dsh secret show $SECRET_UNDER_TEST --value --authentication robot"

echo "-------------------------------"
echo "password file from command line argument"
unset_all
export DSH_CLI_PLATFORM="$PLATFORM"
export DSH_CLI_TENANT="$TENANT"
export DSH_CLI_PASSWORD_FILE="non-existing-file"
export DSH_CLI_PASSWORD="invalid-password"
CMD="$BASE_CMD --password-file $PASSWORD_FILE"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "password from password file in environment variable"
unset_all
export DSH_CLI_PLATFORM="$PLATFORM"
export DSH_CLI_TENANT="$TENANT"
export DSH_CLI_PASSWORD_FILE="$PASSWORD_FILE"
export DSH_CLI_PASSWORD="invalid-password"
CMD="$BASE_CMD"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "password from environment variable"
unset_all
export DSH_CLI_PLATFORM="$PLATFORM"
export DSH_CLI_TENANT="$TENANT"
export DSH_CLI_PASSWORD="$PASSWORD"
CMD="$BASE_CMD"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "password from keyring or prompt"
unset_all
export DSH_CLI_PLATFORM="$PLATFORM"
export DSH_CLI_TENANT="$TENANT"
CMD="$BASE_CMD"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "platform from command line argument"
unset_all
export DSH_CLI_TENANT="$TENANT"
export DSH_CLI_PASSWORD="$PASSWORD"
CMD="$BASE_CMD --platform $PLATFORM"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "platform from environment variable"
unset_all
export DSH_CLI_PLATFORM="$PLATFORM"
export DSH_CLI_TENANT="$TENANT"
export DSH_CLI_PASSWORD="$PASSWORD"
CMD="$BASE_CMD"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "platform from prompt"
echo "enter '$PLATFORM' at the prompt"
unset_all
export DSH_CLI_TENANT="$TENANT"
export DSH_CLI_PASSWORD="$PASSWORD"
CMD="$BASE_CMD"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "tenant from command line argument"
unset_all
export DSH_CLI_PLATFORM="$PLATFORM"
export DSH_CLI_PASSWORD="$PASSWORD"
CMD="$BASE_CMD --tenant $TENANT"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "tenant from environment variable"
unset_all
export DSH_CLI_PLATFORM="$PLATFORM"
export DSH_CLI_TENANT="$TENANT"
export DSH_CLI_PASSWORD="$PASSWORD"
CMD="$BASE_CMD"
echo "> $CMD"
eval "$CMD"

echo "-------------------------------"
echo "tenant from prompt"
echo "enter '$TENANT' at the prompt"
unset_all
export DSH_CLI_PLATFORM="$PLATFORM"
export DSH_CLI_PASSWORD="$PASSWORD"
CMD="$BASE_CMD"
echo "> $CMD"
eval "$CMD"
