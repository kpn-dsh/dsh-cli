#!/bin/bash

# This script can be used as a run test for dsh.
# It will run a large number of erroneous manage commands and print the output to the console.
# This tests must be run from within the 'tests' directory.

export DSH_CLI_PLATFORM=nplz
export DSH_CLI_TENANT=ajuc
export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.ajuc.pwd

# For this test to run the following is expected:
#
# *  Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM and has manage rights
# *  Password file $DSH_CLI_PASSWORD_FILE exists and contains the password
#    for tenant $DSH_CLI_TENANT at platform $DSH_CLI_PLATFORM
# *  Managed internal stream $INTERNAL_STREAM exists at platform $DSH_CLI_PLATFORM and is managed by $DSH_CLI_TENANT
# *  Managed public stream $PUBLIC_STREAM exists at platform $DSH_CLI_PLATFORM and is managed by $DSH_CLI_TENANT
# *  Managed stream $STREAM_NON_EXISTING does not exist at platform $DSH_CLI_PLATFORM
# *  Managed tenant $TENANT exists at platform $DSH_CLI_PLATFORM and is managed by $DSH_CLI_TENANT
# *  Managed tenant $TENANT_NON_EXISTING does not exist at platform $DSH_CLI_PLATFORM

export DSH_CLI_VERBOSITY="high"
export DSH_CLI_SHOW_EXECUTION_TIME=""

MANAGING_TENANT="$DSH_CLI_TENANT"

INTERNAL_STREAM="$MANAGING_TENANT---internal"
PUBLIC_STREAM="$MANAGING_TENANT---public"
STREAM_NON_EXISTING="$MANAGING_TENANT---non-existing"
TENANT="$MANAGING_TENANT-test"
TENANT_NON_EXISTING="$MANAGING_TENANT---non-existing"

ERRONEOUS_MANAGE_COMMANDS=(
  "stream --wrong"

  "stream create --wrong"
  "stream create --internal $INTERNAL_STREAM --dry-run"
  "stream create --internal $PUBLIC_STREAM --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --cleanup-policy wrong --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --compression-type wrong --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --delete-retention-ms=-1 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --max-message-bytes 1023 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --max-message-bytes 1048577 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --message-timestamp-type wrong --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --partitions 0 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --partitions 129 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --retention-bytes=-2 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --retention-ms 3599999 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --retention-ms 31536000001 --dry-run"
  "stream create --internal $STREAM_NON_EXISTING --segment-bytes 52428799 --dry-run"
  "stream create --public $INTERNAL_STREAM --dry-run"
  "stream create --public $PUBLIC_STREAM --dry-run"
  "stream create --public $STREAM_NON_EXISTING --can-be-retained wrong --dry-run"
  "stream create --public $STREAM_NON_EXISTING --cleanup-policy wrong --dry-run"
  "stream create --public $STREAM_NON_EXISTING --compression-type wrong --dry-run"
  "stream create --public $STREAM_NON_EXISTING --delete-retention-ms=-1 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --kafka-default-partitioner wrong --dry-run"
  "stream create --public $STREAM_NON_EXISTING --max-message-bytes 1023 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --max-message-bytes 1048577 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --message-timestamp-type wrong --dry-run"
  "stream create --public $STREAM_NON_EXISTING --partitions 0 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --partitions 129 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --retention-bytes=-2 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --retention-ms 3599999 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --retention-ms 31536000001 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --segment-bytes 52428799 --dry-run"
  "stream create --public $STREAM_NON_EXISTING --topic-level-partitioner 0 --dry-run"

  "stream delete --wrong"
  "stream delete $STREAM_NON_EXISTING --force --dry-run"

  "stream list --wrong"
  "streams --wrong"

  "tenant --wrong"

  "tenant create --wrong"
  "tenant create $TENANT --dry-run"
  "tenant create $TENANT_NON_EXISTING --wrong --dry-run"

  "tenant delete --wrong"
  "tenant delete $TENANT_NON_EXISTING --wrong --dry-run"
  "tenant delete $TENANT_NON_EXISTING --force --dry-run"

  "tenant grant --wrong"
  "tenant grant $TENANT --stream-read $STREAM_NON_EXISTING --dry-run"
  "tenant grant $TENANT_TENANT_NON_EXISTING --stream-read $INTERNAL_STREAM --dry-run"
  "tenant grant $TENANT_TENANT_NON_EXISTING --stream-read $STREAM_NON_EXISTING --dry-run"
  "tenant grant $TENANT --stream-write $STREAM_NON_EXISTING --dry-run"
  "tenant grant $TENANT_TENANT_NON_EXISTING --stream-write $INTERNAL_STREAM --dry-run"
  "tenant grant $TENANT_TENANT_NON_EXISTING --stream-write $STREAM_NON_EXISTING --dry-run"
  "tenant grant $TENANT --stream-rw $STREAM_NON_EXISTING --dry-run"
  "tenant grant $TENANT_TENANT_NON_EXISTING --stream-rw $INTERNAL_STREAM --dry-run"
  "tenant grant $TENANT_TENANT_NON_EXISTING --stream-rw $STREAM_NON_EXISTING --dry-run"

  "tenant list --wrong"
  "tenants --wrong"

  "tenant revoke --wrong"
  "tenant revoke $TENANT --stream-read $STREAM_NON_EXISTING --dry-run"
  "tenant revoke $TENANT_TENANT_NON_EXISTING --stream-read $INTERNAL_STREAM --dry-run"
  "tenant revoke $TENANT_TENANT_NON_EXISTING --stream-read $STREAM_NON_EXISTING --dry-run"
  "tenant revoke $TENANT --stream-write $STREAM_NON_EXISTING --dry-run"
  "tenant revoke $TENANT_TENANT_NON_EXISTING --stream-write $INTERNAL_STREAM --dry-run"
  "tenant revoke $TENANT_TENANT_NON_EXISTING --stream-write $STREAM_NON_EXISTING --dry-run"
  "tenant revoke $TENANT --stream-rw $STREAM_NON_EXISTING --dry-run"
  "tenant revoke $TENANT_TENANT_NON_EXISTING --stream-rw $INTERNAL_STREAM --dry-run"
  "tenant revoke $TENANT_TENANT_NON_EXISTING --stream-rw $STREAM_NON_EXISTING --dry-run"

  "stream show --wrong"
  "stream show $STREAM_NON_EXISTING"

  "tenant update --wrong"
  "tenant update $TENANT_TENANT_NON_EXISTING --certificate-count 1 --dry-run"
  "tenant update $TENANT --certificate-count 0 --dry-run"
  "tenant update $TENANT --certificate-count 41 --dry-run"
  "tenant update $TENANT --certificate-count 3.1415 --dry-run"
  "tenant update $TENANT --certificate-count nan --dry-run"
  "tenant update $TENANT --consumer-rate 1048575 --dry-run"
  "tenant update $TENANT --consumer-rate 1250000001 --dry-run"
  "tenant update $TENANT --consumer-rate 3.1415 --dry-run"
  "tenant update $TENANT --consumer-rate nan --dry-run"
  "tenant update $TENANT --cpu 0.0099 --dry-run"
  "tenant update $TENANT --cpu 16.001 --dry-run"
  "tenant update $TENANT --cpu nan --dry-run"
  "tenant update $TENANT --kafka-acl-group-count=-1 --dry-run"
  "tenant update $TENANT --kafka-acl-group-count 101 --dry-run"
  "tenant update $TENANT --kafka-acl-group-count 3.1415 --dry-run"
  "tenant update $TENANT --kafka-acl-group-count nan --dry-run"
  "tenant update $TENANT --mem 0 --dry-run"
  "tenant update $TENANT --mem 131073 --dry-run"
  "tenant update $TENANT --mem 3.1415 --dry-run"
  "tenant update $TENANT --mem nan --dry-run"
  "tenant update $TENANT --partition-count 0 --dry-run"
  "tenant update $TENANT --partition-count 41 --dry-run"
  "tenant update $TENANT --partition-count 3.1415 --dry-run"
  "tenant update $TENANT --partition-count nan --dry-run"
  "tenant update $TENANT --producer-rate 1048575 --dry-run"
  "tenant update $TENANT --producer-rate 1250000001 --dry-run"
  "tenant update $TENANT --producer-rate 3.1415 --dry-run"
  "tenant update $TENANT --producer-rate nan --dry-run"
  "tenant update $TENANT --request-rate 0 --dry-run"
  "tenant update $TENANT --request-rate 101 --dry-run"
  "tenant update $TENANT --request-rate 3.1415 --dry-run"
  "tenant update $TENANT --request-rate nan --dry-run"
  "tenant update $TENANT --secret-count 0 --dry-run"
  "tenant update $TENANT --secret-count 41 --dry-run"
  "tenant update $TENANT --secret-count 3.1415 --dry-run"
  "tenant update $TENANT --secret-count nan --dry-run"
  "tenant update $TENANT --topic-count 0 --dry-run"
  "tenant update $TENANT --topic-count 41 --dry-run"
  "tenant update $TENANT --topic-count 3.1415 --dry-run"
  "tenant update $TENANT --topic-count nan --dry-run"
  "tenant update $TENANT --tracing 1 --dry-run"
  "tenant update $TENANT --tracing wrong --dry-run"
  "tenant update $TENANT --vpn 1 --dry-run"
  "tenant update $TENANT --vpn wrong --dry-run"
)


set -f
for COMMAND in "${ERRONEOUS_MANAGE_COMMANDS[@]}"
do
  CMD=$(echo "dsh $COMMAND" | envsubst)
  echo "-------------------------------"
  echo "$CMD"
  echo "-------------------------------"
  if eval "$CMD"
  then
    echo "command did not fail: $CMD"
    exit 1
  fi
  echo "-------------------------------"
  echo
done
