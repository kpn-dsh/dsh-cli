#!/bin/bash
set -e

# Run dsh setting commands
#
# This script will run a large number of setting commands.

PLATFORM=nplz
TENANT=greenbox-dev
HOME_DIRECTORY="$(dirname "$0")/.dsh_cli"
export DSH_CLI_HOME="$HOME_DIRECTORY"
rm "$HOME_DIRECTORY/settings.toml"

SETTING_COMMANDS=(
  "dsh setting set authentication robot"
  "dsh setting set authentication sso"
  "dsh setting set browser instruct"
  "dsh setting set browser open"
  "dsh setting set certificate-authority kpn-ca"
  "dsh setting set certificate-authority kpn-digic-rsdv"
  "dsh setting set certificate-authority self-signed"
  "dsh setting set csv-quote \"'\""
  "dsh setting set csv-separator \"|\""
  "dsh setting set default-platform $PLATFORM"
  "dsh setting set default-tenant $TENANT"
  "dsh setting set dry-run"
  "dsh setting unset dry-run"
  "dsh setting set error-color red"
  "dsh setting set error-style bold"
  "dsh setting set expiration 200"
  "dsh setting set label-color magenta"
  "dsh setting set label-style italic"
  "dsh setting set log-color red"
  "dsh setting set log-level info"
  "dsh setting set log-level-api info"
  "dsh setting set log-style bold"
  "dsh setting set matching-color cyan"
  "dsh setting set matching-style bold"
  "dsh setting set no-csv-headers"
  "dsh setting set no-escape"
  "dsh setting unset no-escape"
  "dsh setting set output-format table"
  "dsh setting set quiet"
  "dsh setting unset quiet"
  "dsh setting set show-execution-time"
  "dsh setting set stderr-color green"
  "dsh setting set stderr-style bold"
  "dsh setting set stdout-color black"
  "dsh setting set stdout-style normal"
  "dsh setting set suppress-exit-status"
  "dsh setting set target-color yellow"
  "dsh setting set target-style bold"
  "dsh setting set terminal-width 150"
  "dsh setting set verbosity high"
  "dsh setting set warning-color blue"
  "dsh setting set warning-style bold"
  "dsh -h"
  "dsh setting list"
)

UNSETTING_COMMANDS=(
  "dsh setting unset authentication"
  "dsh setting unset browser"
  "dsh setting unset certificate-authority"
  "dsh setting unset csv-quote"
  "dsh setting unset csv-separator"
  "dsh setting unset default-platform"
  "dsh setting unset default-tenant"
  "dsh setting unset dry-run"
  "dsh setting unset error-color"
  "dsh setting unset error-style"
  "dsh setting unset expiration"
  "dsh setting unset label-color"
  "dsh setting unset label-style"
  "dsh setting unset log-color"
  "dsh setting unset log-level"
  "dsh setting unset log-level-api"
  "dsh setting unset log-style"
  "dsh setting unset matching-color"
  "dsh setting unset matching-style"
  "dsh setting unset no-csv-headers"
  "dsh setting unset no-escape"
  "dsh setting unset output-format"
  "dsh setting unset quiet"
  "dsh setting unset show-execution-time"
  "dsh setting unset stderr-color"
  "dsh setting unset stderr-style"
  "dsh setting unset stdout-color"
  "dsh setting unset stdout-style"
  "dsh setting unset suppress-exit-status"
  "dsh setting unset target-color"
  "dsh setting unset target-style"
  "dsh setting unset terminal-width"
  "dsh setting unset verbosity"
  "dsh setting unset warning-color"
  "dsh setting unset warning-style"
  "dsh -h"
  "dsh setting list"
)

set -f
for SETTING_COMMAND in "${SETTING_COMMANDS[@]}"
do
  CMD=$(echo "$SETTING_COMMAND" | envsubst)
  echo "$CMD"
  echo "-------------------------------"
  eval "$CMD"
  echo "-------------------------------"
  echo
done

set -f
for UNSETTING_COMMAND in "${UNSETTING_COMMANDS[@]}"
do
  CMD=$(echo "$UNSETTING_COMMAND" | envsubst)
  echo "$CMD"
  echo "-------------------------------"
  eval "$CMD"
  echo "-------------------------------"
  echo
done

unset DSH_CLI_HOME
