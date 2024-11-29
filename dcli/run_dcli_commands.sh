#!/bin/bash
set -e

# This script can be used as a run test for dcli.
# It will run a large number of commands from a file and print the output to the console.
# Note that you have to set the DSH_API_SECRET_NPLZ_GREENBOX_DEV environment variable
# prior to starting this script.
#
# export DSH_API_SECRET_NPLZ_GREENBOX_DEV=...

export DSH_API_PLATFORM=nplz
export DSH_API_TENANT=greenbox-dev
export DSH_API_GUID_GREENBOX_DEV=1903

export APP_UNDER_TEST=cmd
export APPLICATION_UNDER_TEST=cmd
export BUCKET_UNDER_TEST=cpr
export CERTIFICATE_UNDER_TEST=broker-kafka-proxy-certificate
export ENV_VALUE_UNDER_TEST=info
export ENV_VALUE_UNDER_TEST_REGEX="^info$"
export IMAGE_UNDER_TEST=registry:eavesdropper:0.9.3
export IMAGE_UNDER_TEST_REGEX=registry
export MANIFEST_UNDER_TEST="kpn/eavesdropper"
export PROXY_UNDER_TEST=broker
export SECRET_UNDER_TEST=boss-account-ids
export TOPIC_UNDER_TEST=reference-implementation-compliant
export VHOST_UNDER_TEST=greenbox-dev
export VOLUME_UNDER_TEST=github-action-runner-home

export RUST_LOG=dcli=info,dsh_api=info

export SEPARATOR="-------------------------------"
export VERBOSITY="-v medium"
#export HIDE_BORDER="--hide-border"
export SHOW_EXECUTION_TIME="--show-execution-time"

IFS=$'\n'
set -f
for i in $(cat < "$1"); do
  CMD=`echo "dcli $VERBOSITY $HIDE_BORDER $SHOW_EXECUTION_TIME $i" | envsubst`
  echo "$SEPARATOR"
  echo "$CMD"
  echo "$SEPARATOR"
  eval "$CMD"
  echo "$SEPARATOR"
  echo
done
