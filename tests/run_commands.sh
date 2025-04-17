#!/bin/bash
set -e

# This script can be used as a run test for dsh.
# It will run a large number of commands from a file and print the output to the console.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev
export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.greenbox-dev.pwd

export DSH_CLI_LOG_LEVEL="error"
export DSH_CLI_OUTPUT_FORMAT="table"
export DSH_CLI_VERBOSITY="high"
export DSH_CLI_SHOW_EXECUTION_TIME=""

source safe_commands.sh

set -f
for COMMAND in "${SAFE_COMMANDS[@]}"
do
  CMD=`echo "dsh $COMMAND" | envsubst`
  echo "$CMD"
  echo "-------------------------------"
  eval "$CMD"
  echo "-------------------------------"
  echo
done
