#!/bin/bash
set -e

# This script can be used as a run test for dsh.
# It will run a number of commands that will open a DSH resource or web application.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev
export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.greenbox-dev.pwd

# For this test to run the following is expected:
#
# * Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM
# * Password file $DSH_CLI_PASSWORD_FILE exists and contains the password
#   for tenant $DSH_CLI_TENANT at platform $DSH_CLI_PLATFORM
# * The user is logged in to the console via SSO/Grip prior to running this test

export DSH_CLI_LOG_LEVEL="error"
export DSH_CLI_OUTPUT_FORMAT="table"
export DSH_CLI_VERBOSITY="low"

export APP_UNDER_TEST=cmd
export SERVICE_UNDER_TEST=keyring-dev

PLATFORM_OPEN_COMMANDS=(
  "app open $APP_UNDER_TEST"
  "platform open app $APP_UNDER_TEST"
  "platform open console"
  "platform open monitoring"
  "platform open service $SERVICE_UNDER_TEST"
  "platform open swagger"
  "platform open tenant"
  "platform open tracing"
  "service open $SERVICE_UNDER_TEST"
)

set -f
for COMMAND in "${PLATFORM_OPEN_COMMANDS[@]}"
do
  CMD=$(echo "dsh $COMMAND" | envsubst)
  echo "-------------------------------"
  echo "$CMD"
  echo "-------------------------------"
  eval "$CMD"
  echo "-------------------------------"
  echo
done
