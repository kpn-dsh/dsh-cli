#!/bin/bash
set -e

# This script can be used as a run test for dsh.
# It will run a large number of commands from a file and print the output to the console.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_LOG_LEVEL="debug"

echo "DSH_CLI_STDOUT_COLOR=blue" > _test-environment-variables.env

ARGUMENTS=(
  ""
  "--environment-variable DSH_CLI_STDOUT_COLOR=blue"
  "-e DSH_CLI_STDOUT_COLOR=blue"
  "--env-var-file _test-environment-variables.env"
)

COMMANDS=(
  ""
  "echo DSH_CLI_STDOUT_COLOR=blue > .dsh_cli.env"
  "export DSH_CLI_STDOUT_COLOR=blue"
)

function clear_all {
  unset DSH_CLI_ENV_FILE
  unset DSH_CLI_STDOUT_COLOR
  rm .dsh_cli.env 2> /dev/null || true
}

set -f
for COMMAND in "${COMMANDS[@]}"
do
  for ARGUMENT in "${ARGUMENTS[@]}"
    do
      clear_all
      CMD=$(echo "dsh platforms $ARGUMENT" | envsubst)
      echo "$COMMAND"
      echo "$CMD"
      echo "-------------------------------"
      eval "$COMMAND"
      eval "$CMD"
      echo "-------------------------------"
      echo
    done
done

rm _test-environment-variables.env 2> /dev/null || true
