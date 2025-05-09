#!/bin/bash
set -e

# This script can be used as a run test for dsh.
# It will run a large number of commands from a file and print the output to the console.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev
export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.greenbox-dev.pwd

ENV_VAR_QUERY=level
IMAGE_QUERY=keyring-service
NON_EXISTING_SERVICE=non-existing-service
NON_EXISTING_TOPIC=non-existing-topic

export DSH_CLI_LOG_LEVEL="error"
export DSH_CLI_OUTPUT_FORMAT="table"
export DSH_CLI_VERBOSITY="high"
export DSH_CLI_SHOW_EXECUTION_TIME=""

function unset_all {
  unset DSH_CLI_ERROR_COLOR    DSH_CLI_ERROR_STYLE
  unset DSH_CLI_LABEL_COLOR    DSH_CLI_LABEL_STYLE
  unset DSH_CLI_MATCHING_COLOR DSH_CLI_MATCHING_STYLE
  unset DSH_CLI_STDERR_COLOR   DSH_CLI_STDERR_STYLE
  unset DSH_CLI_STDOUT_COLOR   DSH_CLI_STDOUT_STYLE
  unset DSH_CLI_WARNING_COLOR  DSH_CLI_WARNING_STYLE
}

ERROR_STYLES=(
  "DSH_CLI_ERROR_COLOR            DSH_CLI_ERROR_STYLE     "
  "DSH_CLI_ERROR_COLOR=normal     DSH_CLI_ERROR_STYLE=normal"
  "DSH_CLI_ERROR_COLOR=yellow     DSH_CLI_ERROR_STYLE=underline"
)

LABEL_STYLES=(
  "DSH_CLI_LABEL_COLOR            DSH_CLI_LABEL_STYLE     "
  "DSH_CLI_LABEL_COLOR=normal     DSH_CLI_LABEL_STYLE=normal"
  "DSH_CLI_LABEL_COLOR=cyan       DSH_CLI_LABEL_STYLE=italic"
)

MATCHING_STYLES=(
  "DSH_CLI_MATCHING_COLOR         DSH_CLI_MATCHING_STYLE     "
  "DSH_CLI_MATCHING_COLOR=normal  DSH_CLI_MATCHING_STYLE=normal"
  "DSH_CLI_MATCHING_COLOR=yellow  DSH_CLI_MATCHING_STYLE=underline"
)

STDERR_STYLES=(
  "DSH_CLI_STDERR_COLOR           DSH_CLI_STDERR_STYLE     "
  "DSH_CLI_STDERR_COLOR=normal    DSH_CLI_STDERR_STYLE=normal"
  "DSH_CLI_STDERR_COLOR=cyan      DSH_CLI_STDERR_STYLE=underline"
)

STDOUT_STYLES=(
  "DSH_CLI_STDOUT_COLOR           DSH_CLI_STDOUT_STYLE     "
  "DSH_CLI_STDOUT_COLOR=normal    DSH_CLI_STDOUT_STYLE=normal"
  "DSH_CLI_STDOUT_COLOR=magenta   DSH_CLI_STDOUT_STYLE=italic"
)

WARNING_STYLES=(
  "DSH_CLI_WARNING_COLOR          DSH_CLI_WARNING_STYLE     "
  "DSH_CLI_WARNING_COLOR=normal   DSH_CLI_WARNING_STYLE=normal"
  "DSH_CLI_WARNING_COLOR=yellow   DSH_CLI_WARNING_STYLE=italic"
)

LABEL_COMMANDS=(
  "--env-var $ENV_VAR_QUERY"
)

set -f
for COMMAND in "${LABEL_COMMANDS[@]}"
do
  for STDOUT_STYLE in "${STDOUT_STYLES[@]}"
  do
    for LABEL_STYLE in "${LABEL_STYLES[@]}"
    do
      unset_all
      CMD=$(echo "dsh $COMMAND" | envsubst)
      echo "$CMD"
      echo "$STDOUT_STYLE"
      echo "$LABEL_STYLE"
      echo "-------------------------------"
      eval "export $STDOUT_STYLE"
      eval "export $LABEL_STYLE"
      eval "$CMD"
      echo "-------------------------------"
      echo
    done
  done
done

MATCHING_COMMANDS=(
  "image find $IMAGE_QUERY --regex"
)

set -f
for COMMAND in "${MATCHING_COMMANDS[@]}"
do
  for STDOUT_STYLE in "${STDOUT_STYLES[@]}"
  do
    for LABEL_STYLE in "${MATCHING_STYLES[@]}"
    do
      unset_all
      CMD=`echo "dsh $COMMAND" | envsubst`
      echo "$CMD"
      echo "$STDOUT_STYLE"
      echo "$LABEL_STYLE"
      echo "-------------------------------"
      eval "export $STDOUT_STYLE"
      eval "export $LABEL_STYLE"
      eval "$CMD"
      echo "-------------------------------"
      echo
    done
  done
done

ERROR_COMMANDS=(
  "service show $NON_EXISTING_SERVICE --suppress-exit-status"
)

set -f
for COMMAND in "${ERROR_COMMANDS[@]}"
do
  for STDERR_STYLE in "${STDERR_STYLES[@]}"
  do
    for ERROR_STYLE in "${ERROR_STYLES[@]}"
    do
      unset_all
      CMD=`echo "dsh $COMMAND" | envsubst`
      echo "$CMD"
      echo "$STDERR_STYLE"
      echo "$ERROR_STYLE"
      echo "-------------------------------"
      eval "export $STDERR_STYLE"
      eval "export $ERROR_STYLE"
      eval "$CMD"
      echo "-------------------------------"
      echo
    done
  done
done

WARNING_COMMANDS=(
  "topic create $NON_EXISTING_TOPIC --dry-run"
)

set -f
for COMMAND in "${WARNING_COMMANDS[@]}"
do
  for STDERR_STYLE in "${STDERR_STYLES[@]}"
  do
    for WARNING_STYLE in "${WARNING_STYLES[@]}"
    do
      unset_all
      CMD=`echo "dsh $COMMAND" | envsubst`
      echo "$CMD"
      echo "$STDERR_STYLE"
      echo "$WARNING_STYLE"
      echo "-------------------------------"
      eval "export $STDERR_STYLE"
      eval "export $WARNING_STYLE"
      eval "$CMD"
      echo "-------------------------------"
      echo
    done
  done
done
