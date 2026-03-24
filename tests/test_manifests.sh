#!/bin/bash

# Run dsh manifest commands
#
# This script will run a large number of manifest commands and print the output to the console.
#
# To run this test the following is expected:
# * User is logged in via the single sign on authentication pattern at platform $DSH_CLI_PLATFORM
# * Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM
# * User is authenticated for tenant $DSH_CLI_TENANT at platform $DSH_CLI_PLATFORM

export DSH_CLI_PLATFORM=np-aws-lz-dsh
export DSH_CLI_TENANT=greenbox-dev

export MANIFEST_SUBCOMMAND=show

MANIFEST_EXPORT_COMMANDS=(
  "manifest $MANIFEST_SUBCOMMAND kpn/aep-sink-connect"
  "manifest $MANIFEST_SUBCOMMAND kpn/airflow-ephemeral"
  "manifest $MANIFEST_SUBCOMMAND kpn/airflow-persistent"
  "manifest $MANIFEST_SUBCOMMAND kpn/cmdline"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-database-ingester"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-labs"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-labs-kernel"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-ollama"
  "manifest $MANIFEST_SUBCOMMAND kpn/eavesdropper"
  "manifest $MANIFEST_SUBCOMMAND kpn/explorer"
  "manifest $MANIFEST_SUBCOMMAND kpn/greenbox"
  "manifest $MANIFEST_SUBCOMMAND kpn/http-source-connector"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafdrop"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka-data-archiver"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka2kafka"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-kafka-database-extractor"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-service"
  "manifest $MANIFEST_SUBCOMMAND kpn/metrics-proxy"
  "manifest $MANIFEST_SUBCOMMAND kpn/oidc-fwd-auth"
  "manifest $MANIFEST_SUBCOMMAND kpn/prometheus-scraper"
  "manifest $MANIFEST_SUBCOMMAND kpn/schema-store-ui"
  "manifest $MANIFEST_SUBCOMMAND kpn/secor"
  "manifest $MANIFEST_SUBCOMMAND kpn/sql-database"
  "manifest $MANIFEST_SUBCOMMAND kpn/sql-database-viewer"
  "manifest $MANIFEST_SUBCOMMAND kpn/topic-metrics-exporter"
  "manifest $MANIFEST_SUBCOMMAND kpn/whoami"
  "manifest $MANIFEST_SUBCOMMAND kpn/zookeeper-proxy"
)

set -f
for COMMAND in "${MANIFEST_EXPORT_COMMANDS[@]}"
do
  CMD=$(echo "dsh $COMMAND" | envsubst)
  echo $CMD
  eval "$CMD"
  echo
done
