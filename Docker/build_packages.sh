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

if [[ -n "${AUR_BUILDER_SIGN_PACKAGES}" ]]; then
  echo "Signing packages enabled. Configuring package signing"

  # Import the private key
  sudo -u build gpg --import "/aur-builder-keys/signing.key"

  # Tell pacman about the signing key
  pacman-key --add "/aur-builder-keys/signing.pub"

  # Set the key id in makepkg.conf
  echo "
  GPGKEY=\"${AUR_BUILDER_GPG_KEY_ID}\"
  " >> /etc/makepkg.conf
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

SYNC_ARGS=()

if [[ -n "${AUR_BUILDER_SIGN_PACKAGES}" ]]; then
  SYNC_ARGS+=("--sign")
fi

if [[ -z "${AUR_BUILDER_NEW_PACKAGES}" ]]; then
  # If no new packages are provided we just want to update, so pass the -u command
  SYNC_ARGS+=("-u")
elif [[ "${AUR_BUILDER_NEW_PACKAGES}" == "rebuild" ]]; then
  SYNC_ARGS+=("--rebuild-all")
else
  # If new packages are given, then we want to install them. Parse them from ';' seperated format into an array
  SYNC_PACKAGES=()
  IFS=';' read -r -a SYNC_PACKAGES <<< "${AUR_BUILDER_NEW_PACKAGES}"
  SYNC_ARGS+=("${SYNC_PACKAGES[@]}")
fi

sudo -u build aur sync --noconfirm --noview ${SYNC_ARGS[@]}
