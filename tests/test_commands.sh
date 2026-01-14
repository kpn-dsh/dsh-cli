#!/bin/bash
set -e

# This script can be used as a run test for dsh.
# It will run a large number of commands from a file and print the output to the console.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev
export DSH_CLI_PASSWORD_FILE="$(dirname "$0")/../np-aws-lz-dsh.greenbox.pwd"

# For this test to run the following is expected:
#
# * Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM
# * Password file $DSH_CLI_PASSWORD_FILE exists and contains the password
#   for tenant $DSH_CLI_TENANT at platform $DSH_CLI_PLATFORM

export DSH_CLI_VERBOSITY="high"
export DSH_CLI_OUTPUT_FORMAT="table"
export DSH_CLI_SHOW_EXECUTION_TIME=""

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
