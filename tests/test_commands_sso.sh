#!/bin/bash
set -e

# Run dsh commands
#
# This script will run a large number of commands from a file and print the output to the console.
#
# To run this test the following is expected:
# * User is logged in via the single-sign-on authentication pattern at platform $DSH_CLI_PLATFORM
# * Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM
# * User is authenticated for tenant $DSH_CLI_TENANT at platform $DSH_CLI_PLATFORM

export DSH_CLI_AUTHENTICATION=sso
export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev

export DSH_CLI_OUTPUT_FORMAT="table"
export DSH_CLI_SHOW_EXECUTION_TIME=""
export DSH_CLI_VERBOSITY="high"

source "$(dirname "$0")/commands.sh"

set -f
for COMMAND in "${SAFE_COMMANDS[@]}"
do
  if [[ $COMMAND == "$1"* ]]
  then
    CMD=$(echo "dsh $COMMAND" | envsubst)
    echo "$CMD"
    echo "-------------------------------"
    eval "$CMD"
    echo "-------------------------------"
    echo
  fi
done
