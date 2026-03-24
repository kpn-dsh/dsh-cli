#!/bin/bash
set -e

# Run commands using the robot access pattern
#
# This script test the various robot authentication patterns for dsh.
# You will be prompted for some values during the process.
# Enter $DSH_CLI_PLATFORM and $DSH_CLI_TENANT when requested.
#
# For this test to run successfully the following is expected:
#
# * User is not logged in via single sign on access pattern
# * Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM
# * Secret 'test' exsists
# * Password file $DSH_CLI_PASSWORD_FILE exists and contains the password for tenant $DSH_CLI_TENANT
#   at platform $DSH_CLI_PLATFORM

DSH_CLI_PLATFORM="np-aws-lz-dsh"
DSH_CLI_TENANT="greenbox-dev"
SECRET_NAME="test"

export DSH_CLI_PLATFORM
export DSH_CLI_TENANT

ROBOT_PLATFORM="$DSH_CLI_PLATFORM"
ROBOT_TENANT="$DSH_CLI_TENANT"
ROBOT_PASSWORD_FILE="$(dirname "$0")/../dirty/$DSH_CLI_PLATFORM.$DSH_CLI_TENANT.pwd"
ROBOT_PASSWORD="$(cat "$ROBOT_PASSWORD_FILE")"

export DSH_CLI_ROBOT_PLATFORM="$ROBOT_PLATFORM"
export DSH_CLI_ROBOT_TENANT="$ROBOT_TENANT"
export DSH_CLI_ROBOT_PASSWORD_FILE="$ROBOT_PASSWORD_FILE"
export DSH_CLI_ROBOT_PASSWORD="$ROBOT_PASSWORD"

function unset_all {
  unset DSH_CLI_ROBOT_PLATFORM
  unset DSH_CLI_ROBOT_TENANT
  unset DSH_CLI_ROBOT_PASSWORD
  unset DSH_CLI_ROBOT_PASSWORD_FILE
}

BASE_CMD="dsh secret show $SECRET_NAME --value --authentication robot"

unset_all
export DSH_CLI_ROBOT_PLATFORM="$ROBOT_PLATFORM"
export DSH_CLI_ROBOT_TENANT="$ROBOT_TENANT"
export DSH_CLI_ROBOT_PASSWORD_FILE="$ROBOT_PASSWORD_FILE"
echo "$BASE_CMD"
eval "$BASE_CMD"

printf "\n-------------------------------\n\n"

unset_all
export DSH_CLI_ROBOT_PLATFORM="$ROBOT_PLATFORM"
export DSH_CLI_ROBOT_TENANT="$ROBOT_TENANT"
export DSH_CLI_ROBOT_PASSWORD="$ROBOT_PASSWORD"
echo "$BASE_CMD"
eval "$BASE_CMD"

printf "\n-------------------------------\n\n"

unset_all
export DSH_CLI_ROBOT_PASSWORD_FILE="$ROBOT_PASSWORD_FILE"
CMD="$BASE_CMD --robot-platform $ROBOT_PLATFORM --robot-tenant $ROBOT_TENANT"
echo "$CMD"
eval "$CMD"

printf "\n-------------------------------\n\n"

unset_all
export DSH_CLI_ROBOT_PASSWORD="$ROBOT_PASSWORD"
CMD="$BASE_CMD --robot-platform $ROBOT_PLATFORM --robot-tenant $ROBOT_TENANT"
echo "$CMD"
eval "$CMD"







