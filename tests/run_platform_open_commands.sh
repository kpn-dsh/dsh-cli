#!/bin/bash

# This script can be used as a run test for dsh.
# It will run a number of commands that will open a DSH resource or web application.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev
export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.greenbox-dev.pwd

export DSH_CLI_LOG_LEVEL="error"
export DSH_CLI_OUTPUT_FORMAT="table"
export DSH_CLI_VERBOSITY="low"

export APP_UNDER_TEST=kafdrop
export SERVICE_UNDER_TEST=keyring-dev

PLATFORM_OPEN_COMMANDS=(
  "platform open app $APP_UNDER_TEST"
  "platform open console"
  "platform open monitoring"
  "platform open service $SERVICE_UNDER_TEST"
  "platform open swagger"
  "platform open tenant"
  "platform open tracing"
)

set -f
for COMMAND in "${PLATFORM_OPEN_COMMANDS[@]}"
do
  CMD=`echo "dsh $COMMAND" | envsubst`
  echo "-------------------------------"
  echo "$CMD"
  echo "-------------------------------"
  eval "$CMD"
  echo "-------------------------------"
  echo
done
