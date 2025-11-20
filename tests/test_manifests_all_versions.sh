#!/bin/bash

export DSH_CLI_PLATFORM=np-aws-lz-dsh
export DSH_CLI_TENANT=greenbox-dev

export DSH_CLI_PASSWORD_FILE=../${DSH_CLI_PLATFORM}.${DSH_CLI_TENANT}.pwd
#export DSH_CLI_PASSWORD_FILE=../np-aws-lz-dsh.greenbox-dev.pwd

# For this test to run the following is expected:
#
# * Tenant $DSH_CLI_TENANT exists at platform $DSH_CLI_PLATFORM
# * Password file $DSH_CLI_PASSWORD_FILE exists and contains the password
#   for tenant $DSH_CLI_TENANT at platform $DSH_CLI_PLATFORM

#export MANIFEST_SUBCOMMAND=export
#export MANIFEST_SUBCOMMAND=explain
export MANIFEST_SUBCOMMAND=show

MANIFEST_EXPORT_COMMANDS=(
  "manifest $MANIFEST_SUBCOMMAND kpn/aep-sink-connect 0.1.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/aep-sink-connect 0.1.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/airflow-ephemeral 3.0.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/airflow-ephemeral 0.9.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/airflow-persistent 3.0.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/airflow-persistent 1.0.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/cmdline 1.1.6"
  "manifest $MANIFEST_SUBCOMMAND kpn/cmdline 1.1.5"
  "manifest $MANIFEST_SUBCOMMAND kpn/cmdline 1.1.3"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-database-ingester 0.4.6"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-database-ingester 0.4.4"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-database-ingester 0.4.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-database-ingester 0.4.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-database-ingester 0.3.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-database-ingester 0.1.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-ollama 0.5.0-phi"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-ollama 0.5.0-mistral"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-ollama 0.5.0-gemma"
  "manifest $MANIFEST_SUBCOMMAND kpn/dsh-ollama 0.5.0-all"
  "manifest $MANIFEST_SUBCOMMAND kpn/eavesdropper 0.9.3"
  "manifest $MANIFEST_SUBCOMMAND kpn/eavesdropper 0.9.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/eavesdropper 0.9.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/eavesdropper 0.8.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/eavesdropper 0.8.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/eavesdropper 0.7.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/explorer 0.0.7"
  "manifest $MANIFEST_SUBCOMMAND kpn/greenbox 0.0.7"
  "manifest $MANIFEST_SUBCOMMAND kpn/greenbox 0.0.6"
  "manifest $MANIFEST_SUBCOMMAND kpn/http-source-connector 0.6.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/http-source-connector 0.5.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/http-source-connector 0.5.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafdrop 4.1.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafdrop 4.0.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka-data-archiver 1.6.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka-data-archiver 1.5.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka-data-archiver 1.4.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka-data-archiver 1.3.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka-data-archiver 1.0.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka2kafka 1.1.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/kafka2kafka 1.0.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-kafka-database-extractor 0.4.4"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-kafka-database-extractor 0.4.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-kafka-database-extractor 0.4.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-kafka-database-extractor 0.4.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-kafka-database-extractor 0.3.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-kafka-database-extractor 0.2.4"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-service 0.6.3"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-service 0.5.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-service 0.5.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-service 0.4.3"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-service 0.4.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/keyring-service 0.4.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/metrics-proxy 0.1.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/metrics-proxy 0.1.1"
  "manifest $MANIFEST_SUBCOMMAND kpn/oidc-fwd-auth 1.1.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/oidc-fwd-auth 1.0.0"
  "manifest $MANIFEST_SUBCOMMAND kpn/prometheus-scraper 0.1.7"
  "manifest $MANIFEST_SUBCOMMAND kpn/prometheus-scraper 0.1.6"
  "manifest $MANIFEST_SUBCOMMAND kpn/prometheus-scraper 0.1.5"
  "manifest $MANIFEST_SUBCOMMAND kpn/prometheus-scraper 0.1.4"
  "manifest $MANIFEST_SUBCOMMAND kpn/schema-store-ui 0.0.15"
  "manifest $MANIFEST_SUBCOMMAND kpn/schema-store-ui 0.0.14"
  "manifest $MANIFEST_SUBCOMMAND kpn/schema-store-ui 0.0.13"
  "manifest $MANIFEST_SUBCOMMAND kpn/schema-store-ui 0.0.12"
  "manifest $MANIFEST_SUBCOMMAND kpn/schema-store-ui 0.0.11-beta"
  "manifest $MANIFEST_SUBCOMMAND kpn/schema-store-ui 0.0.10-beta"
  "manifest $MANIFEST_SUBCOMMAND kpn/secor 0.30.3"
  "manifest $MANIFEST_SUBCOMMAND kpn/secor 0.30.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/sql-database 1.1.3"
  "manifest $MANIFEST_SUBCOMMAND kpn/sql-database 1.1.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/sql-database-viewer 1.1.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/topic-metrics-exporter 0.1.5"
  "manifest $MANIFEST_SUBCOMMAND kpn/whoami 0.0.7"
  "manifest $MANIFEST_SUBCOMMAND kpn/whoami 0.0.4"
  "manifest $MANIFEST_SUBCOMMAND kpn/zookeeper-proxy 1.2.2"
  "manifest $MANIFEST_SUBCOMMAND kpn/zookeeper-proxy 1.2.1"
)

set -f
for COMMAND in "${MANIFEST_EXPORT_COMMANDS[@]}"
do
  CMD=$(echo "dsh $COMMAND" | envsubst)
  echo $CMD
  eval "$CMD"
  echo
done
