#!/bin/bash

export APP_UNDER_TEST=kafdrop
export BUCKET_UNDER_TEST=cpr
export CERTIFICATE_UNDER_TEST=broker
export ENV_VALUE_UNDER_TEST=info
export ENV_VALUE_UNDER_TEST_REGEX="^info$"
export IMAGE_UNDER_TEST=registry:eavesdropper:0.9.2
export IMAGE_UNDER_TEST_REGEX=registry
export MANIFEST_UNDER_TEST=dsh-ollama
export MANIFEST_UNDER_TEST_VERSION=0.5.0-all
export PLATFORM_UNDER_TEST=nplz
export PROXY_UNDER_TEST=broker
export SECRET_NON_EXISTING=non-existing-secret
export SECRET_UNDER_TEST=boss-account-ids
export SERVICE_NON_EXISTING=non-existing-service
export SERVICE_UNDER_TEST=keyring-dev
export TENANT_UNDER_TEST=greenbox-dev
export TOPIC_NON_EXISTING=non-existing-topic
export TOPIC_UNDER_TEST=reference-implementation-compliant
export VENDOR_UNDER_TEST=kpn
export VHOST_UNDER_TEST=greenbox-dev
export VOLUME_NON_EXISTING=non-existing-volume
export VOLUME_UNDER_TEST=github-action-runner-home

export SAFE_COMMANDS=(
  "-h"
  "--help"
  "--version"
  "--generate-autocomplete-file zsh"

  "api delete secret-configuration $SECRET_UNDER_TEST --force --dry-run"
  "api get secret $SECRET_UNDER_TEST"
  "api post secret --dry-run < /dev/null"
  "api put secret $SECRET_UNDER_TEST --dry-run < /dev/null"
  "api show > /dev/null"

  "app list --ids"
  "app list"
  "app show $APP_UNDER_TEST"

  "bucket list --ids"
  "bucket list"
  "bucket show $BUCKET_UNDER_TEST"

  "certificate list --configuration"
  "certificate list --ids"
  "certificate list --status"
  "certificate list --usage"
  "certificate list"
  "certificate show $CERTIFICATE_UNDER_TEST --status"
  "certificate show $CERTIFICATE_UNDER_TEST --usage"
  "certificate show $CERTIFICATE_UNDER_TEST"

  "env find $ENV_VALUE_UNDER_TEST --started"
  "env find $ENV_VALUE_UNDER_TEST --stopped"
  "env find $ENV_VALUE_UNDER_TEST"
  "env find $ENV_VALUE_UNDER_TEST_REGEX --regex --started"
  "env find $ENV_VALUE_UNDER_TEST_REGEX --regex --stopped"
  "env find $ENV_VALUE_UNDER_TEST_REGEX --regex"

  "image find $IMAGE_UNDER_TEST --started"
  "image find $IMAGE_UNDER_TEST --stopped"
  "image find $IMAGE_UNDER_TEST"
  "image find $IMAGE_UNDER_TEST_REGEX --regex --started"
  "image find $IMAGE_UNDER_TEST_REGEX --regex --stopped"
  "image find $IMAGE_UNDER_TEST_REGEX --regex"
  "image list --started"
  "image list --stopped"
  "image list"

  "manifest export $MANIFEST_UNDER_TEST $MANIFEST_UNDER_TEST_VERSION"
  "manifest list"
  "manifest list --ids"
  "manifest show $MANIFEST_UNDER_TEST"
  "manifest show $MANIFEST_UNDER_TEST $MANIFEST_UNDER_TEST_VERSION"
  "manifest show $MANIFEST_UNDER_TEST --complete"
  "manifest show $MANIFEST_UNDER_TEST $MANIFEST_UNDER_TEST_VERSION --complete"

  "metric list --started"
  "metric list --stopped"
  "metric list"

  "platform export"
  "platform list"
  "platform open app $APP_UNDER_TEST --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST --dry-run"
  "platform open app $APP_UNDER_TEST --dry-run"
  "platform open console --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST --dry-run"
  "platform open console --dry-run"
  "platform open monitoring --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST --dry-run"
  "platform open monitoring --dry-run"
  "platform open service $SERVICE_UNDER_TEST --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST --dry-run"
  "platform open service $SERVICE_UNDER_TEST --dry-run"
  "platform open swagger --platform $PLATFORM_UNDER_TEST --dry-run"
  "platform open swagger --dry-run"
  "platform open tenant --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST --dry-run"
  "platform open tenant --dry-run"
  "platform open tracing --platform $PLATFORM_UNDER_TEST --dry-run"
  "platform open tracing --dry-run"
  "platform show --app $APP_UNDER_TEST --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST"
  "platform show --app $APP_UNDER_TEST --vendor $VENDOR_UNDER_TEST --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST"
  "platform show --app $APP_UNDER_TEST --vendor $VENDOR_UNDER_TEST"
  "platform show --app $APP_UNDER_TEST"
  "platform show --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST"
  "platform show --service $SERVICE_UNDER_TEST --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST"
  "platform show --service $SERVICE_UNDER_TEST"
  "platform show --vhost $VHOST_UNDER_TEST --platform $PLATFORM_UNDER_TEST --tenant $TENANT_UNDER_TEST"
  "platform show --vhost $VHOST_UNDER_TEST"
  "platform show"

  "proxy list --ids"
  "proxy list"
  "proxy show $PROXY_UNDER_TEST"

  "secret create $SECRET_NON_EXISTING --dry-run < /dev/null"
  "secret delete $SECRET_UNDER_TEST --force --dry-run"
  "secret list --app"
  "secret list --service"
  "secret list --status"
  "secret list --system"
  "secret list --usage"
  "secret list"
  "secret show $SECRET_UNDER_TEST --usage"
  "secret show $SECRET_UNDER_TEST --value"
  "secret show $SECRET_UNDER_TEST"

  "service delete $SERVICE_UNDER_TEST --force --dry-run"
  "service list --ids"
  "service list --started"
  "service list --status"
  "service list --stopped"
  "service list --tasks"
  "service list"
  "service restart $SERVICE_UNDER_TEST --force --dry-run"
  "service show $SERVICE_UNDER_TEST --status"
  "service show $SERVICE_UNDER_TEST --tasks"
  "service show $SERVICE_UNDER_TEST"
  "service start $SERVICE_UNDER_TEST --force --dry-run"
  "service start $SERVICE_UNDER_TEST --force --instances 2 --dry-run"
  "service stop $SERVICE_UNDER_TEST --force --dry-run"
  "service update $SERVICE_UNDER_TEST --cpus 1 --instances 1 --mem 32 --force --dry-run"

  "setting list"

  "target list"

  "topic create $TOPIC_NON_EXISTING --dry-run"
  "topic create $TOPIC_NON_EXISTING --cleanup-policy compact --dry-run"
  "topic create $TOPIC_NON_EXISTING --max-message-size 2048 --dry-run"
  "topic create $TOPIC_NON_EXISTING --partitions 2 --dry-run"
  "topic create $TOPIC_NON_EXISTING --segment-size 52428801 --dry-run"
  "topic list --ids"
  "topic list --status"
  "topic list --usage"
  "topic list"
  "topic show $TOPIC_UNDER_TEST --properties"
  "topic show $TOPIC_UNDER_TEST --status"
  "topic show $TOPIC_UNDER_TEST --usage"
  "topic show $TOPIC_UNDER_TEST"

  "vhost list"

  "volume create $VOLUME_NON_EXISTING --size 2 --dry-run"
  "volume delete $VOLUME_UNDER_TEST --force --dry-run"
  "volume list --app"
  "volume list --service"
  "volume list --configuration"
  "volume list --ids"
  "volume list --status"
  "volume list --usage"
  "volume list"
  "volume show $VOLUME_UNDER_TEST --status"
  "volume show $VOLUME_UNDER_TEST --usage"
  "volume show $VOLUME_UNDER_TEST"
)
