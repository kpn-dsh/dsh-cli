#!/bin/bash
set -e

# Run dsh manage commands
#
# This script will run a large number of commands and print the output to the console.
#
# To run this test the following is expected:
# * User is logged in via the single sign on authentication pattern at platform $DSH_CLI_PLATFORM
# * Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM and has manage rights
# * User is authenticated for tenant $DSH_CLI_TENANT at platform $DSH_CLI_PLATFORM
# *  Managed internal stream $INTERNAL_STREAM exists at platform $DSH_CLI_PLATFORM and is managed by $DSH_CLI_TENANT
# *  Managed public stream $PUBLIC_STREAM exists at platform $DSH_CLI_PLATFORM and is managed by $DSH_CLI_TENANT
# *  Managed stream $STREAM_NON_EXISTING does not exist at platform $DSH_CLI_PLATFORM
# *  Managed tenant $TENANT exists at platform $DSH_CLI_PLATFORM and is managed by $DSH_CLI_TENANT
# *  Managed tenant $TENANT_NON_EXISTING does not exist at platform $DSH_CLI_PLATFORM

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=ajuc

export DSH_CLI_VERBOSITY="high"
export DSH_CLI_SHOW_EXECUTION_TIME=""

MANAGING_TENANT="$DSH_CLI_TENANT"

INTERNAL_STREAM="$MANAGING_TENANT---internal"
PUBLIC_STREAM="$MANAGING_TENANT---internal"
STREAM_NON_EXISTING="$MANAGING_TENANT---non-existing"
TENANT="$MANAGING_TENANT-test"
TENANT_NON_EXISTING="$MANAGING_TENANT---non-existing"

MANAGE_COMMANDS=(
  "stream create --internal $STREAM_NON_EXISTING --cleanup-policy compact --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --compression-type gzip --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --delete-retention-ms 0 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --delete-retention-ms 6000 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --max-message-bytes 1024 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --max-message-bytes 1048576 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --message-timestamp-type create-time --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --partitions 1 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --partitions 128 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --retention-bytes=-1 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --retention-ms 3600000 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --retention-ms 31536000000 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --segment-bytes 52428800 --dry-run"

  "stream create --public $STREAM_NON_EXISTING --can-be-retained --dry-run"
  "stream create --public $STREAM_NON_EXISTING --cleanup-policy compact --dry-run"
  "stream create --public $STREAM_NON_EXISTING --compression-type lz4 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --delete-retention-ms 6000 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --kafka-default-partitioner --dry-run"
  "stream create --public $STREAM_NON_EXISTING --max-message-bytes 1024 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --max-message-bytes 1048576 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --message-timestamp-type log-append-time --dry-run"
  "stream create --public $STREAM_NON_EXISTING --partitions 1 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --partitions 128 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --retention-bytes 1000 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --retention-ms 3600000 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --retention-ms 31536000000 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --segment-bytes 52428800 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --topic-level-partitioner 1 --dry-run"

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

  "tenant update $TENANT --certificate-count 1 --dry-run"
  "tenant update $TENANT --certificate-count 40 --dry-run"
  "tenant update $TENANT --consumer-rate 1048576 --dry-run"
  "tenant update $TENANT --consumer-rate 1250000000 --dry-run"
  "tenant update $TENANT --cpu 0.01 --dry-run"
  "tenant update $TENANT --cpu 16.0 --dry-run"
  "tenant update $TENANT --cpu 16 --dry-run"
  "tenant update $TENANT --kafka-acl-group-count 0 --dry-run"
  "tenant update $TENANT --kafka-acl-group-count 100 --dry-run"
  "tenant update $TENANT --mem 1 --dry-run"
  "tenant update $TENANT --mem 131072 --dry-run"
  "tenant update $TENANT --partition-count 1 --dry-run"
  "tenant update $TENANT --partition-count 40 --dry-run"
  "tenant update $TENANT --producer-rate 1048576 --dry-run"
  "tenant update $TENANT --producer-rate 1250000000 --dry-run"
  "tenant update $TENANT --request-rate 1 --dry-run"
  "tenant update $TENANT --request-rate 100 --dry-run"
  "tenant update $TENANT --secret-count 1 --dry-run"
  "tenant update $TENANT --secret-count 40 --dry-run"
  "tenant update $TENANT --topic-count 1 --dry-run"
  "tenant update $TENANT --topic-count 40 --dry-run"
  "tenant update $TENANT --tracing false --dry-run"
  "tenant update $TENANT --tracing true --dry-run"
  "tenant update $TENANT --vpn false --dry-run"
  "tenant update $TENANT --vpn true --dry-run"

  "tenant update $TENANT --certificate-count 4 --consumer-rate 1048576 --cpu 1 --kafka-acl-group-count 4 --mem 131072 --partition-count 4 --producer-rate 1250000000 --request-rate 4 --secret-count 4 --topic-count 4 --dry-run"
  "tenant update $TENANT --tracing false --vpn false --dry-run"
)

set -f
for COMMAND in "${MANAGE_COMMANDS[@]}"
do
  CMD=$(echo "dsh $COMMAND" | envsubst)
  echo "$CMD"
  echo "-------------------------------"
  eval "$CMD"
  echo "-------------------------------"
  echo
done
