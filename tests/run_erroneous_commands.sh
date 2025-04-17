#!/bin/bash

# This script can be used as a run test for dsh.
# It will run a large number of erroneous commands from a file and print the output to the console.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev
export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.greenbox-dev.pwd

export DSH_CLI_LOG_LEVEL="error"
export DSH_CLI_OUTPUT_FORMAT="table"
export DSH_CLI_VERBOSITY="high"
export DSH_CLI_SHOW_EXECUTION_TIME=""

source erroneous_commands.sh

set -f
for COMMAND in "${ERRONEOUS_COMMANDS[@]}"
do
  CMD=`echo "dsh $COMMAND" | envsubst`
  echo "-------------------------------"
  echo "$CMD"
  echo "-------------------------------"
  eval "$CMD"
  if [ $? = "0" ]
  then
    echo "command did not fail: $CMD"
    exit 1
  fi
  echo "-------------------------------"
  echo
done
