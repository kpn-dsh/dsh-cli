#!/bin/bash
set -e

# This script can be used as a run test for dsh.
# It will run a large number of commands from a file and print the output to the console.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=greenbox-dev
export DSH_CLI_PASSWORD_FILE=np-aws-lz-dsh.greenbox-dev.pwd

export APP_UNDER_TEST=kafdrop
export APPLICATION_UNDER_TEST=kafdrop
export BUCKET_UNDER_TEST=cpr
export CERTIFICATE_UNDER_TEST=broker
export ENV_VALUE_UNDER_TEST=info
export ENV_VALUE_UNDER_TEST_REGEX="^info$"
export IMAGE_UNDER_TEST=registry:eavesdropper:0.9.3
export IMAGE_UNDER_TEST_REGEX=registry
export MANIFEST_UNDER_TEST="kpn/eavesdropper"
export PLATFORM_UNDER_TEST=prodlz
export PROXY_UNDER_TEST=broker
export SECRET_UNDER_TEST=boss-account-ids
export TENANT_UNDER_TEST=greenbox-dev
export TOPIC_UNDER_TEST=reference-implementation-compliant
export VENDOR_UNDER_TEST=kpn
export VHOST_UNDER_TEST=greenbox-dev
export VOLUME_UNDER_TEST=github-action-runner-home

export RUST_LOG=dsh=info,dsh_api=info

export SEPARATOR="-------------------------------"

#export MATCHING_STYLE="--matching-style bold"
#export OUTPUT_FORMAT="--output-format json"
#export SHOW_EXECUTION_TIME="--show-execution-time"
#export VERBOSITY="-v high"

export MATCHING_STYLE=""
#export OUTPUT_FORMAT=""
export SHOW_EXECUTION_TIME=""
export VERBOSITY=""

IFS=$'\n'
set -f
for i in $(cat < "$1"); do
  CMD=`echo "dsh --dry-run $MATCHING_STYLE $OUTPUT_FORMAT $SHOW_EXECUTION_TIME $VERBOSITY $i" | envsubst`
  echo "$SEPARATOR"
  echo "$CMD"
  echo "$SEPARATOR"
  eval "$CMD"
  echo "$SEPARATOR"
  echo
done
