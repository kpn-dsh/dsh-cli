set -e

#export DSH_API_PLATFORM=nplz
#export DSH_API_SECRET_NPLZ_GREENBOX_DEV=55hMQcrS7xDw4GxnjptgYWagpNaGOmPp
#export DSH_API_TENANT=greenbox-dev
#export DSH_API_USER_GREENBOX_DEV=1903:1903
#export RUST_LOG=trifonius_engine=info,dsh_api=info

export APP_UNDER_TEST=cmd
export APPLICATION_UNDER_TEST=cmd
export BUCKET_UNDER_TEST=cpr
export CERTIFICATE_UNDER_TEST=broker-kafka-proxy-certificate
export ENV_VALUE_UNDER_TEST=info
export PROXY_UNDER_TEST=broker
export SECRET_UNDER_TEST=boss-account-ids
export TOPIC_UNDER_TEST=reference-implementation-compliant
export VHOST_UNDER_TEST=greenbox-dev
export VOLUME_UNDER_TEST=github-action-runner-home

dcli -vvv app list
dcli -vvv app list --all
dcli -vvv app list --status
dcli -vvv app list --configuration
dcli -vvv app list --ids
dcli -vvv app l
dcli -vvv apps
dcli -vvv app show $APP_UNDER_TEST
dcli -vvv app show $APP_UNDER_TEST --all

dcli -vvv application diff cmd
dcli -vvv application list
dcli -vvv application list --all
dcli -vvv application list --status
dcli -vvv application list --configuration
dcli -vvv application list --ids
dcli -vvv application list --tasks
dcli -vvv application list --started
dcli -vvv application list --stopped
dcli -vvv application l
dcli -vvv a l
dcli -vvv applications
dcli -vvv as
dcli -vvv application show $APPLICATION_UNDER_TEST
dcli -vvv application show $APPLICATION_UNDER_TEST --all
dcli -vvv application show $APPLICATION_UNDER_TEST --status
dcli -vvv application show $APPLICATION_UNDER_TEST --configuration
dcli -vvv application show $APPLICATION_UNDER_TEST --tasks

dcli -vvv bucket list
dcli -vvv bucket list --all
dcli -vvv bucket list --status
dcli -vvv bucket list --configuration
dcli -vvv bucket list --ids
dcli -vvv bucket l
dcli -vvv b l
dcli -vvv buckets
dcli -vvv bs
dcli -vvv bucket show $BUCKET_UNDER_TEST
dcli -vvv bucket show $BUCKET_UNDER_TEST --all
dcli -vvv bucket show $BUCKET_UNDER_TEST --status

dcli -vvv certificate list
dcli -vvv certificate list --all
dcli -vvv certificate list --status
dcli -vvv certificate list --configuration
dcli -vvv certificate list --ids
dcli -vvv certificate list --usage
dcli -vvv certificate l
dcli -vvv c l
dcli -vvv certificates
dcli -vvv cs
dcli -vvv certificate show $CERTIFICATE_UNDER_TEST
dcli -vvv certificate show $CERTIFICATE_UNDER_TEST --all
dcli -vvv certificate show $CERTIFICATE_UNDER_TEST --status
dcli -vvv certificate show $CERTIFICATE_UNDER_TEST --usage

dcli -vvv env find $ENV_VALUE_UNDER_TEST
dcli -vvv env find $ENV_VALUE_UNDER_TEST --app
dcli -vvv env find $ENV_VALUE_UNDER_TEST --application

dcli -vvv manifest list
dcli -vvv manifest list --all
dcli -vvv manifest list --configuration
dcli -vvv manifest list --ids

dcli -vvv proxy list
dcli -vvv proxy list --all
dcli -vvv proxy list --ids
dcli -vvv proxy l
dcli -vvv proxys
dcli -vvv proxy show $PROXY_UNDER_TEST
dcli -vvv proxy show $PROXY_UNDER_TEST --configuration

dcli -vvv secret list
dcli -vvv secret list --all
dcli -vvv secret list --status
dcli -vvv secret list --ids
dcli -vvv secret list --usage
dcli -vvv secret l
dcli -vvv s l
dcli -vvv secrets
dcli -vvv ss
dcli -vvv secret show $SECRET_UNDER_TEST
dcli -vvv secret show $SECRET_UNDER_TEST --status
dcli -vvv secret show $SECRET_UNDER_TEST --usage
dcli -vvv secret show $SECRET_UNDER_TEST --value

dcli -vvv topic list
dcli -vvv topic list --status
dcli -vvv topic list --configuration
dcli -vvv topic list --ids
dcli -vvv topic list --usage
dcli -vvv topic show $TOPIC_UNDER_TEST
dcli -vvv topic show $TOPIC_UNDER_TEST --status
dcli -vvv topic show $TOPIC_UNDER_TEST --configuration
dcli -vvv topic show $TOPIC_UNDER_TEST --usage
dcli -vvv topic show $TOPIC_UNDER_TEST --properties

dcli -vvv vhost list
dcli -vvv vhost list --usage
dcli -vvv vhost l
dcli -vvv v l
dcli -vvv vhosts
dcli -vvv vs
dcli -vvv vhost show $VHOST_UNDER_TEST
dcli -vvv vhost show $VHOST_UNDER_TEST --usage

dcli -vvv volume list
dcli -vvv volume list --all
dcli -vvv volume list --status
dcli -vvv volume list --configuration
dcli -vvv volume list --ids
dcli -vvv volume list --usage
dcli -vvv volume list --usage --app
dcli -vvv volume list --usage --application
dcli -vvv volume show $VOLUME_UNDER_TEST
dcli -vvv volume show $VOLUME_UNDER_TEST --all
dcli -vvv volume show $VOLUME_UNDER_TEST --status
dcli -vvv volume show $VOLUME_UNDER_TEST --configuration
dcli -vvv volume show $VOLUME_UNDER_TEST --usage

##dcli -vvv proxy delete
##dcli -vvv proxy update
##dcli -vvv secret create
##dcli -vvv secret delete
##dcli -vvv volume create
##dcli -vvv volume delete
