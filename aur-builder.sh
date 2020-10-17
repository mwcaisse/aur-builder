#!/bin/bash
if [[ -z "${AUR_BUILDER_CONFIG_FILE}" ]]; then
  AUR_BUILDER_CONFIG_FILE="/opt/aur-builder/aur-builder.conf"
fi
source ${AUR_BUILDER_CONFIG_FILE}

echo "${AUR_BUILDER_IMAGE_TAG}"
if [[ -z "${AUR_BUILDER_IMAGE_TAG}" ]]; then
  echo "AUR_BUILDER_IMAGE_TAG not set. Please set it in the configuration file '${AUR_BUILDER_CONFIG_FILE}'"
  exit 1
fi

if [[ -z "${AUR_BUILDER_REPO_DIR}" ]] ; then
  echo "AUR_BUILDER_REPO_DIR not set. Please set it in the configuration file '${AUR_BUILDER_CONFIG_FILE}'"
  exit 1
fi

if [[ -z "${AUR_BUILDER_REPO_NAME}" ]] ; then
  echo "AUR_BUILDER_REPO_NAME not set. Please set it in the configuration file '${AUR_BUILDER_CONFIG_FILE}'"
  exit 1
fi


PACKAGES_TO_INSTALL=""
for arg in ${BASH_ARGV[*]} ; do
  if [[ ! -z "${PACKAGES_TO_INSTALL}" ]] ; then
    PACKAGES_TO_INSTALL="${PACKAGES_TO_INSTALL};"
  fi
  PACKAGES_TO_INSTALL="${PACKAGES_TO_INSTALL}${arg}"
done

docker run \
  --mount type=bind,source=${AUR_BUILDER_REPO_DIR},destination="/repo" \
  --env AUR_BUILDER_REPO_NAME="${AUR_BUILDER_REPO_NAME}" \
  --env AUR_BUILDER_NEW_PACKAGES="${PACKAGES_TO_INSTALL}" \
  "${AUR_BUILDER_IMAGE_TAG}"