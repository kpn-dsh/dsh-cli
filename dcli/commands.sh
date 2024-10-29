set -e

# This file can be used as a run test for dcli.
# It will run a large number of commands and print the output to the console.
# Note that you have to set the DSH_API_SECRET_NPLZ_GREENBOX_DEV environment variable
# prior to starting this script.
#
# export DSH_API_SECRET_NPLZ_GREENBOX_DEV=...

export DSH_API_PLATFORM=nplz
export DSH_API_TENANT=greenbox-dev
export DSH_API_USER_GREENBOX_DEV=1903:1903

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
export VERBOSITY="-vvv"

echo $SEPARATOR
echo app
echo $SEPARATOR

echo dcli $VERBOSITY app list
dcli $VERBOSITY app list
echo $SEPARATOR

echo dcli $VERBOSITY app list --all
dcli $VERBOSITY app list --all
echo $SEPARATOR

echo dcli $VERBOSITY app list --status
dcli $VERBOSITY app list --status
echo $SEPARATOR

echo dcli $VERBOSITY app list --configuration
dcli $VERBOSITY app list --configuration
echo $SEPARATOR

echo dcli $VERBOSITY app list --ids
dcli $VERBOSITY app list --ids
echo $SEPARATOR

echo dcli $VERBOSITY app l
dcli $VERBOSITY app l
echo $SEPARATOR

echo dcli $VERBOSITY apps
dcli $VERBOSITY apps
echo $SEPARATOR

echo dcli $VERBOSITY app show $APP_UNDER_TEST
dcli $VERBOSITY app show $APP_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY app show $APP_UNDER_TEST --all
dcli $VERBOSITY app show $APP_UNDER_TEST --all
echo $SEPARATOR


echo $SEPARATOR
echo application
echo $SEPARATOR

echo dcli $VERBOSITY application diff cmd
dcli $VERBOSITY application diff cmd
echo $SEPARATOR

echo dcli $VERBOSITY application list
dcli $VERBOSITY application list
echo $SEPARATOR

echo dcli $VERBOSITY application list --all
dcli $VERBOSITY application list --all
echo $SEPARATOR

echo dcli $VERBOSITY application list --status
dcli $VERBOSITY application list --status
echo $SEPARATOR

echo dcli $VERBOSITY application list --configuration
dcli $VERBOSITY application list --configuration
echo $SEPARATOR

echo dcli $VERBOSITY application list --ids
dcli $VERBOSITY application list --ids
echo $SEPARATOR

echo dcli $VERBOSITY application list --tasks
dcli $VERBOSITY application list --tasks
echo $SEPARATOR

echo dcli $VERBOSITY application list --started
dcli $VERBOSITY application list --started
echo $SEPARATOR

echo dcli $VERBOSITY application list --stopped
dcli $VERBOSITY application list --stopped
echo $SEPARATOR

echo dcli $VERBOSITY application l
dcli $VERBOSITY application l
echo $SEPARATOR

echo dcli $VERBOSITY a l
dcli $VERBOSITY a l
echo $SEPARATOR

echo dcli $VERBOSITY applications
dcli $VERBOSITY applications
echo $SEPARATOR

echo dcli $VERBOSITY as
dcli $VERBOSITY as
echo $SEPARATOR

echo dcli $VERBOSITY application show $APPLICATION_UNDER_TEST
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --all
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --all
echo $SEPARATOR

echo dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --status
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --status
echo $SEPARATOR

echo dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --configuration
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --configuration
echo $SEPARATOR

echo dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --tasks
dcli $VERBOSITY application show $APPLICATION_UNDER_TEST --tasks
echo $SEPARATOR


echo $SEPARATOR
echo bucket
echo $SEPARATOR

echo dcli $VERBOSITY bucket list
dcli $VERBOSITY bucket list
echo $SEPARATOR

echo dcli $VERBOSITY bucket list --all
dcli $VERBOSITY bucket list --all
echo $SEPARATOR

echo dcli $VERBOSITY bucket list --status
dcli $VERBOSITY bucket list --status
echo $SEPARATOR

echo dcli $VERBOSITY bucket list --configuration
dcli $VERBOSITY bucket list --configuration
echo $SEPARATOR

echo dcli $VERBOSITY bucket list --ids
dcli $VERBOSITY bucket list --ids
echo $SEPARATOR

echo dcli $VERBOSITY bucket l
dcli $VERBOSITY bucket l
echo $SEPARATOR

echo dcli $VERBOSITY b l
dcli $VERBOSITY b l
echo $SEPARATOR

echo dcli $VERBOSITY buckets
dcli $VERBOSITY buckets
echo $SEPARATOR

echo dcli $VERBOSITY bs
dcli $VERBOSITY bs
echo $SEPARATOR

echo dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST
dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST --all
dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST --all
echo $SEPARATOR

echo dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST --status
dcli $VERBOSITY bucket show $BUCKET_UNDER_TEST --status
echo $SEPARATOR


echo $SEPARATOR
echo certificate
echo $SEPARATOR

echo dcli $VERBOSITY certificate list
dcli $VERBOSITY certificate list
echo $SEPARATOR

echo dcli $VERBOSITY certificate list --all
dcli $VERBOSITY certificate list --all
echo $SEPARATOR

echo dcli $VERBOSITY certificate list --status
dcli $VERBOSITY certificate list --status
echo $SEPARATOR

echo dcli $VERBOSITY certificate list --configuration
dcli $VERBOSITY certificate list --configuration
echo $SEPARATOR

echo dcli $VERBOSITY certificate list --ids
dcli $VERBOSITY certificate list --ids
echo $SEPARATOR

echo dcli $VERBOSITY certificate list --usage
dcli $VERBOSITY certificate list --usage
echo $SEPARATOR

echo dcli $VERBOSITY certificate l
dcli $VERBOSITY certificate l
echo $SEPARATOR

echo dcli $VERBOSITY c l
dcli $VERBOSITY c l
echo $SEPARATOR

echo dcli $VERBOSITY certificates
dcli $VERBOSITY certificates
echo $SEPARATOR

echo dcli $VERBOSITY cs
dcli $VERBOSITY cs
echo $SEPARATOR

echo dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST
dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --all
dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --all
echo $SEPARATOR

echo dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --status
dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --status
echo $SEPARATOR

echo dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --usage
dcli $VERBOSITY certificate show $CERTIFICATE_UNDER_TEST --usage
echo $SEPARATOR


echo $SEPARATOR
echo env
echo $SEPARATOR

echo dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --regex
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --regex
echo $SEPARATOR

echo dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST --app
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST --app
echo $SEPARATOR

echo dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --app --regex
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --app --regex
echo $SEPARATOR

echo dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST --application
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST --application
echo $SEPARATOR

echo dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --application --regex
dcli $VERBOSITY env find $ENV_VALUE_UNDER_TEST_REGEX --application --regex
echo $SEPARATOR


echo $SEPARATOR
echo image
echo $SEPARATOR

echo dcli $VERBOSITY image find $IMAGE_UNDER_TEST
dcli $VERBOSITY image find $IMAGE_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --regex
dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --regex
echo $SEPARATOR

echo dcli $VERBOSITY image find $IMAGE_UNDER_TEST --app
dcli $VERBOSITY image find $IMAGE_UNDER_TEST --app
echo $SEPARATOR

echo dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --app --regex
dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --app --regex
echo $SEPARATOR

echo dcli $VERBOSITY image find $IMAGE_UNDER_TEST --application
dcli $VERBOSITY image find $IMAGE_UNDER_TEST --application
echo $SEPARATOR

echo dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --application --regex
dcli $VERBOSITY image find $IMAGE_UNDER_TEST_REGEX --application --regex
echo $SEPARATOR


echo $SEPARATOR
echo manifest
echo $SEPARATOR

echo dcli $VERBOSITY manifest list
dcli $VERBOSITY manifest list
echo $SEPARATOR

echo dcli $VERBOSITY manifest list --all
dcli $VERBOSITY manifest list --all
echo $SEPARATOR

echo dcli $VERBOSITY manifest list --ids
dcli $VERBOSITY manifest list --ids
echo $SEPARATOR

echo dcli $VERBOSITY manifest show $MANIFEST_UNDER_TEST
dcli $VERBOSITY manifest show $MANIFEST_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY manifest show $MANIFEST_UNDER_TEST --all
dcli $VERBOSITY manifest show $MANIFEST_UNDER_TEST --all
echo $SEPARATOR


echo $SEPARATOR
echo proxy
echo $SEPARATOR

echo dcli $VERBOSITY proxy list
dcli $VERBOSITY proxy list
echo $SEPARATOR

echo dcli $VERBOSITY proxy list --all
dcli $VERBOSITY proxy list --all
echo $SEPARATOR

echo dcli $VERBOSITY proxy list --ids
dcli $VERBOSITY proxy list --ids
echo $SEPARATOR

echo dcli $VERBOSITY proxy l
dcli $VERBOSITY proxy l
echo $SEPARATOR

echo dcli $VERBOSITY proxys
dcli $VERBOSITY proxys
echo $SEPARATOR

echo dcli $VERBOSITY proxy show $PROXY_UNDER_TEST
dcli $VERBOSITY proxy show $PROXY_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY proxy show $PROXY_UNDER_TEST --configuration
dcli $VERBOSITY proxy show $PROXY_UNDER_TEST --configuration
echo $SEPARATOR


echo $SEPARATOR
echo secret
echo $SEPARATOR

echo dcli $VERBOSITY secret list
dcli $VERBOSITY secret list
echo $SEPARATOR

echo dcli $VERBOSITY secret list --all
dcli $VERBOSITY secret list --all
echo $SEPARATOR

echo dcli $VERBOSITY secret list --status
dcli $VERBOSITY secret list --status
echo $SEPARATOR

echo dcli $VERBOSITY secret list --ids
dcli $VERBOSITY secret list --ids
echo $SEPARATOR

echo dcli $VERBOSITY secret list --usage
dcli $VERBOSITY secret list --usage
echo $SEPARATOR

echo dcli $VERBOSITY secret l
dcli $VERBOSITY secret l
echo $SEPARATOR

echo dcli $VERBOSITY s l
dcli $VERBOSITY s l
echo $SEPARATOR

echo dcli $VERBOSITY secrets
dcli $VERBOSITY secrets
echo $SEPARATOR

echo dcli $VERBOSITY ss
dcli $VERBOSITY ss
echo $SEPARATOR

echo dcli $VERBOSITY secret show $SECRET_UNDER_TEST
dcli $VERBOSITY secret show $SECRET_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY secret show $SECRET_UNDER_TEST --status
dcli $VERBOSITY secret show $SECRET_UNDER_TEST --status
echo $SEPARATOR

echo dcli $VERBOSITY secret show $SECRET_UNDER_TEST --usage
dcli $VERBOSITY secret show $SECRET_UNDER_TEST --usage
echo $SEPARATOR

echo dcli $VERBOSITY secret show $SECRET_UNDER_TEST --value
dcli $VERBOSITY secret show $SECRET_UNDER_TEST --value
echo $SEPARATOR


echo $SEPARATOR
echo topic
echo $SEPARATOR

echo dcli $VERBOSITY topic list
dcli $VERBOSITY topic list
echo $SEPARATOR

echo dcli $VERBOSITY topic list --status
dcli $VERBOSITY topic list --status
echo $SEPARATOR

echo dcli $VERBOSITY topic list --configuration
dcli $VERBOSITY topic list --configuration
echo $SEPARATOR

echo dcli $VERBOSITY topic list --ids
dcli $VERBOSITY topic list --ids
echo $SEPARATOR

echo dcli $VERBOSITY topic list --usage
dcli $VERBOSITY topic list --usage
echo $SEPARATOR

echo dcli $VERBOSITY topic show $TOPIC_UNDER_TEST
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --status
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --status
echo $SEPARATOR

echo dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --configuration
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --configuration
echo $SEPARATOR

echo dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --usage
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --usage
echo $SEPARATOR

echo dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --properties
dcli $VERBOSITY topic show $TOPIC_UNDER_TEST --properties
echo $SEPARATOR


echo $SEPARATOR
echo vhost
echo $SEPARATOR

echo dcli $VERBOSITY vhost list
dcli $VERBOSITY vhost list
echo $SEPARATOR

echo dcli $VERBOSITY vhost list --usage
dcli $VERBOSITY vhost list --usage
echo $SEPARATOR

echo dcli $VERBOSITY vhost l
dcli $VERBOSITY vhost l
echo $SEPARATOR

echo dcli $VERBOSITY v l
dcli $VERBOSITY v l
echo $SEPARATOR

echo dcli $VERBOSITY vhosts
dcli $VERBOSITY vhosts
echo $SEPARATOR

echo dcli $VERBOSITY vs
dcli $VERBOSITY vs
echo $SEPARATOR

echo dcli $VERBOSITY vhost show $VHOST_UNDER_TEST
dcli $VERBOSITY vhost show $VHOST_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY vhost show $VHOST_UNDER_TEST --usage
dcli $VERBOSITY vhost show $VHOST_UNDER_TEST --usage
echo $SEPARATOR


echo $SEPARATOR
echo volume
echo $SEPARATOR

echo dcli $VERBOSITY volume list
dcli $VERBOSITY volume list
echo $SEPARATOR

echo dcli $VERBOSITY volume list --all
dcli $VERBOSITY volume list --all
echo $SEPARATOR

echo dcli $VERBOSITY volume list --status
dcli $VERBOSITY volume list --status
echo $SEPARATOR

echo dcli $VERBOSITY volume list --configuration
dcli $VERBOSITY volume list --configuration
echo $SEPARATOR

echo dcli $VERBOSITY volume list --ids
dcli $VERBOSITY volume list --ids
echo $SEPARATOR

echo dcli $VERBOSITY volume list --usage
dcli $VERBOSITY volume list --usage
echo $SEPARATOR

echo dcli $VERBOSITY volume list --usage --app
dcli $VERBOSITY volume list --usage --app
echo $SEPARATOR

echo dcli $VERBOSITY volume list --usage --application
dcli $VERBOSITY volume list --usage --application
echo $SEPARATOR

echo dcli $VERBOSITY volume show $VOLUME_UNDER_TEST
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST
echo $SEPARATOR

echo dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --all
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --all
echo $SEPARATOR

echo dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --status
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --status
echo $SEPARATOR

echo dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --configuration
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --configuration
echo $SEPARATOR

echo dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --usage
dcli $VERBOSITY volume show $VOLUME_UNDER_TEST --usage
echo $SEPARATOR


##dcli $VERBOSITY proxy delete
##dcli $VERBOSITY proxy update
##dcli $VERBOSITY secret create
##dcli $VERBOSITY secret delete
##dcli $VERBOSITY volume create
##dcli $VERBOSITY volume delete
