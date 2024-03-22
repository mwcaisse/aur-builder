#!/bin/bash

if [[ -z "${AUR_BUILDER_CONFIG_FILE}" ]]; then
  AUR_BUILDER_CONFIG_FILE="/opt/aur-builder/aur-builder.conf"
fi
source ${AUR_BUILDER_CONFIG_FILE}

if [[ -z "${AUR_BUILDER_REPO_DIR}" ]] ; then
  echo "AUR_BUILDER_REPO_DIR not set. Please set it in the configuration file '${AUR_BUILDER_CONFIG_FILE}'"
  exit 1
fi

paccache -rv -c "${AUR_BUILDER_REPO_DIR}" -k 2
