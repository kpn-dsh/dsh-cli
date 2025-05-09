#!/bin/bash
set -e

# This script can be used as a run test for dsh.
# It will run a large number of commands from a file and print the output to the console.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=ajuc
export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.ajuc.pwd

export DSH_CLI_VERBOSITY="high"
export DSH_CLI_SHOW_EXECUTION_TIME=""

MANAGING_TENANT=ajuc

INTERNAL_STREAM="$MANAGING_TENANT---internal"
PUBLIC_STREAM="$MANAGING_TENANT---internal"
STREAM_NON_EXISTING="$MANAGING_TENANT---non-existing"
TENANT="$MANAGING_TENANT-test"
TENANT_NON_EXISTING="$MANAGING_TENANT---non-existing"

MANAGE_COMMANDS=(
  "stream create --internal $STREAM_NON_EXISTING --cleanup-policy compact --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --compression-type gzip --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --delete-retention-ms 6000 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --max-message-bytes 1024 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --message-timestamp-type producer --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --partitions 3 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --retention-bytes 1000 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --retention-ms 31536000000 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --segment-bytes 52428800 --dry-run"

  "stream create --public $STREAM_NON_EXISTING --can-be-retained --dry-run"
  "stream create --public $STREAM_NON_EXISTING --cleanup-policy compact --dry-run"
  "stream create --public $STREAM_NON_EXISTING --compression-type gzip --dry-run"
  "stream create --public $STREAM_NON_EXISTING --delete-retention-ms 6000 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --kafka-default-partitioner --dry-run"
  "stream create --public $STREAM_NON_EXISTING --max-message-bytes 1024 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --message-timestamp-type producer --dry-run"
  "stream create --public $STREAM_NON_EXISTING --partitions 3 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --retention-bytes 1000 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --retention-ms 31536000000 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --segment-bytes 52428800 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --topic-level-partitioner 2 --dry-run"

  "stream delete $INTERNAL_STREAM --force --dry-run"

  "stream list"
  "stream list --public"
  "stream list --internal"
  "stream list --internal --public"
  "stream list --ids"
  "stream list --ids --public"
  "stream list --ids --internal"
  "stream list --ids --internal --public"
  "streams"

  "stream show $INTERNAL_STREAM"
  "stream show $PUBLIC_STREAM"

  "tenant create $TENANT_NON_EXISTING --dry-run"
  "tenant create $TENANT_NON_EXISTING --tracing true --dry-run"
  "tenant create $TENANT_NON_EXISTING --vpn true --dry-run"

  "tenant delete $TENANT --force --dry-run"

  "tenant grant $TENANT --stream-read $INTERNAL_STREAM --dry-run"
  "tenant grant $TENANT --stream-write $INTERNAL_STREAM --dry-run"
  "tenant grant $TENANT --stream-rw $INTERNAL_STREAM --dry-run"

  "tenant list"
  "tenant list --ids"
  "tenant list --stream"
  "tenant list"

  "tenant revoke $TENANT --stream-read $INTERNAL_STREAM --dry-run"
  "tenant revoke $TENANT --stream-write $INTERNAL_STREAM --dry-run"
  "tenant revoke $TENANT --stream-rw $INTERNAL_STREAM --dry-run"

  "tenant show $TENANT"
  "tenant show $TENANT --stream"

  "tenant update $TENANT --certificate-count 5 --dry-run"
  "tenant update $TENANT --consumer-rate 1048576 --dry-run"
  "tenant update $TENANT --cpu 0.2 --dry-run"
  "tenant update $TENANT --kafka-acl-group-count 5 --dry-run"
  "tenant update $TENANT --mem 2048 --dry-run"
  "tenant update $TENANT --partition-count 5 --dry-run"
  "tenant update $TENANT --producer-rate 1048576 --dry-run"
  "tenant update $TENANT --request-rate 5 --dry-run"
  "tenant update $TENANT --secret-count 5 --dry-run"
  "tenant update $TENANT --topic-count 5 --dry-run"
  "tenant update $TENANT --tracing false --dry-run"
  "tenant update $TENANT --vpn false --dry-run"
)

set -f
for COMMAND in "${MANAGE_COMMANDS[@]}"
do
  CMD=`echo "dsh $COMMAND" | envsubst`
  echo "$CMD"
  echo "-------------------------------"
  eval "$CMD"
  echo "-------------------------------"
  echo
done
