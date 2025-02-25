#!/bin/bash
set -e

# This script can be used as a run test for dsh.
# It will run a large number of commands from a file and print the output to the console.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev
export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.greenbox-dev.pwd

export SEPARATOR="-------------------------------"

#export MATCHING_STYLE="--matching-style bold"
#export OUTPUT_FORMAT="--output-format json"
#export SHOW_EXECUTION_TIME="--show-execution-time"
#export VERBOSITY="-v high"
#export LOG_LEVEL="--log-level debug"

export DRY_RUN="--dry-run"

source export_safe_commands.sh

#IFS=$'\n'
set -f
for COMMAND in "${SAFE_COMMANDS[@]}"
do
  CMD=`echo "dsh $DRY_RUN $MATCHING_STYLE $OUTPUT_FORMAT $SHOW_EXECUTION_TIME $VERBOSITY $LOG_LEVEL $COMMAND" | envsubst`
  echo "$SEPARATOR"
  echo "$CMD"
  echo "$SEPARATOR"
  eval "$CMD"
  echo "$SEPARATOR"
  echo
done
