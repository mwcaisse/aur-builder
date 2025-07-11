#!/bin/bash
set -ex

AUR_REPO_DIR="/repo"

## First let's update all of the packages, make sure everything is up to date
pacman -Syyu --noconfirm

if [[ -n "${AUR_BUILDER_GPG_KEYS}" ]]; then
  AUR_BUILDER_GPG_KEYS_LIST=($AUR_BUILDER_GPG_KEYS)
  # Import any keys for packages
  for keyid in "${AUR_BUILDER_GPG_KEYS_LIST[@]}"; do
    echo "Adding gpg key ${keyid}."
    sudo -u build gpg --recv-key "${keyid}"
  done
fi


## We aren't signing the packages yet, so allow packages without signatures
echo "
[${AUR_BUILDER_REPO_NAME}]
SigLevel = Optional TrustAll
Server = file://${AUR_REPO_DIR}
" >> /etc/pacman.conf

# Create the repo
# TODO: For now we will assume that the repo exists
#repo-add "${REPO_DIR}/${AUR_REPO_NAME}.db.tar.xz"

# Sync up the aur database
pacman -Syu --noconfirm

chown -R build:build "${AUR_REPO_DIR}"

# Create a working directory to use while creating the packages
mkdir working-dir
chown -R build:build ./working-dir
pushd working-dir

## Let Build install packages
export EDITOR="tee -a"
echo "build ALL=(ALL) NOPASSWD: ALL" | visudo

SYNC_ARGS=""

## Lets build yay and shutter and put them into aur-repo directory
if [[ -z "${AUR_BUILDER_NEW_PACKAGES}" ]]; then
  # If no new packages are provided we just want to update, so pass the -u command
  SYNC_ARGS=("-u")
else
  # If new packages are given, then we want to install them. Parse them from ';' seperated format into an array
  IFS=';' read -r -a SYNC_ARGS <<< "${AUR_BUILDER_NEW_PACKAGES}"

fi

sudo -u build aur sync --noconfirm --noview ${SYNC_ARGS[@]}
