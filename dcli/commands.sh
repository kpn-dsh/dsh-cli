set -e

#export DSH_API_PLATFORM=nplz
#export DSH_API_SECRET_NPLZ_GREENBOX_DEV=...
#export DSH_API_TENANT=greenbox-dev
#export DSH_API_USER_GREENBOX_DEV=1903:1903
#export RUST_LOG=trifonius_engine=info,dsh_api=info

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

export VERBOSITY="-vvv"

dcli $VERBOSITY app list
echo
dcli $VERBOSITY app list --all
echo
dcli $VERBOSITY app list --status
echo
dcli $VERBOSITY app list --configuration
echo
dcli $VERBOSITY app list --ids
echo
dcli $VERBOSITY app l
echo
dcli $VERBOSITY apps
echo
dcli $VERBOSITY app show $APP_UNDER_TEST
echo
dcli $VERBOSITY app show $APP_UNDER_TEST --all
echo

dcli $VERBOSITY application diff cmd
echo
dcli $VERBOSITY application list
echo
dcli $VERBOSITY application list --all
echo
dcli $VERBOSITY application list --status
echo
dcli $VERBOSITY application list --configuration
echo
dcli $VERBOSITY application list --ids
echo
dcli $VERBOSITY application list --tasks
echo
dcli $VERBOSITY application list --started
echo
dcli $VERBOSITY application list --stopped
echo
dcli $VERBOSITY application l
echo
dcli $VERBOSITY a l
echo
dcli $VERBOSITY applications
echo
dcli $VERBOSITY as
echo
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST
echo
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --all
echo
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --status
echo
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --configuration
echo
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --tasks
echo

dcli $VERBOSITY bucket list
echo
dcli $VERBOSITY bucket list --all
echo
dcli $VERBOSITY bucket list --status
echo
dcli $VERBOSITY bucket list --configuration
echo
dcli $VERBOSITY bucket list --ids
echo
dcli $VERBOSITY bucket l
echo
dcli $VERBOSITY b l
echo
dcli $VERBOSITY buckets
echo
dcli $VERBOSITY bs
echo
dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST
echo
dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST --all
echo
dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST --status
echo

dcli $VERBOSITY certificate list
echo
dcli $VERBOSITY certificate list --all
echo
dcli $VERBOSITY certificate list --status
echo
dcli $VERBOSITY certificate list --configuration
echo
dcli $VERBOSITY certificate list --ids
echo
dcli $VERBOSITY certificate list --usage
echo
dcli $VERBOSITY certificate l
echo
dcli $VERBOSITY c l
echo
dcli $VERBOSITY certificates
echo
dcli $VERBOSITY cs
echo
dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST
echo
dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --all
echo
dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --status
echo
dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --usage
echo

dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST
echo
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --regex
echo
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST --app
echo
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --app --regex
echo
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST --application
echo
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --application --regex
echo

dcli $VERBOSITY image find $IMAGE_UNDER_TEST
echo
dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --regex
echo
dcli $VERBOSITY image find $IMAGE_UNDER_TEST --app
echo
dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --app --regex
echo
dcli $VERBOSITY image find $IMAGE_UNDER_TEST --application
echo
dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --application --regex
echo

dcli $VERBOSITY manifest list
echo
dcli $VERBOSITY manifest list --all
echo
dcli $VERBOSITY manifest list --ids
echo
dcli $VERBOSITY manifest show $MANIFEST_UNDER_TEST
echo
dcli $VERBOSITY manifest show $MANIFEST_UNDER_TEST --all
echo

dcli $VERBOSITY proxy list
echo
dcli $VERBOSITY proxy list --all
echo
dcli $VERBOSITY proxy list --ids
echo
dcli $VERBOSITY proxy l
echo
dcli $VERBOSITY proxys
echo
dcli $VERBOSITY proxy show $PROXY_UNDER_TEST
echo
dcli $VERBOSITY proxy show $PROXY_UNDER_TEST --configuration
echo

dcli $VERBOSITY secret list
echo
dcli $VERBOSITY secret list --all
echo
dcli $VERBOSITY secret list --status
echo
dcli $VERBOSITY secret list --ids
echo
dcli $VERBOSITY secret list --usage
echo
dcli $VERBOSITY secret l
echo
dcli $VERBOSITY s l
echo
dcli $VERBOSITY secrets
echo
dcli $VERBOSITY ss
echo
dcli $VERBOSITY secret show $SECRET_UNDER_TEST
echo
dcli $VERBOSITY secret show $SECRET_UNDER_TEST --status
echo
dcli $VERBOSITY secret show $SECRET_UNDER_TEST --usage
echo
dcli $VERBOSITY secret show $SECRET_UNDER_TEST --value
echo

dcli $VERBOSITY topic list
echo
dcli $VERBOSITY topic list --status
echo
dcli $VERBOSITY topic list --configuration
echo
dcli $VERBOSITY topic list --ids
echo
dcli $VERBOSITY topic list --usage
echo
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST
echo
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --status
echo
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --configuration
echo
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --usage
echo
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --properties
echo

dcli $VERBOSITY vhost list
echo
dcli $VERBOSITY vhost list --usage
echo
dcli $VERBOSITY vhost l
echo
dcli $VERBOSITY v l
echo
dcli $VERBOSITY vhosts
echo
dcli $VERBOSITY vs
echo
dcli $VERBOSITY vhost show $VHOST_UNDER_TEST
echo
dcli $VERBOSITY vhost show $VHOST_UNDER_TEST --usage
echo

dcli $VERBOSITY volume list
echo
dcli $VERBOSITY volume list --all
echo
dcli $VERBOSITY volume list --status
echo
dcli $VERBOSITY volume list --configuration
echo
dcli $VERBOSITY volume list --ids
echo
dcli $VERBOSITY volume list --usage
echo
dcli $VERBOSITY volume list --usage --app
echo
dcli $VERBOSITY volume list --usage --application
echo
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST
echo
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --all
echo
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --status
echo
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --configuration
echo
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --usage
echo

##dcli $VERBOSITY proxy delete
##dcli $VERBOSITY proxy update
##dcli $VERBOSITY secret create
##dcli $VERBOSITY secret delete
##dcli $VERBOSITY volume create
##dcli $VERBOSITY volume delete
