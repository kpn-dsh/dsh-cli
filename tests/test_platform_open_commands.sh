#!/bin/bash
set -e

# Run dsh platform open commands
#
# This script will run platform open commands which results in a number of webpages that will be opened in
# your default browser.
#
# To run this test the following is expected:
# * User is logged in via the single-sign-on authentication method at platform $DSH_CLI_PLATFORM
# * Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM
# * User is authenticated for tenant $DSH_CLI_TENANT at platform $DSH_CLI_PLATFORM

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev

export DSH_CLI_LOG_LEVEL="error"
export DSH_CLI_OUTPUT_FORMAT="table"
export DSH_CLI_VERBOSITY="low"

export APP_UNDER_TEST=cmd
export SERVICE_UNDER_TEST=keyring-dev
export SERVICE_TASK_UNDER_TEST=58b9fc6c48-z9t46-00000000
export VHOST_UNDER_TEST=eavesdropper

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
  "task open $SERVICE_UNDER_TEST"
  "task open $SERVICE_UNDER_TEST $SERVICE_TASK_UNDER_TEST"
  "vhost open $VHOST_UNDER_TEST"
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
