FROM rust:1.94-slim AS build

# install nbuild dependencies
RUN apt-get update && apt-get install -y --no-install-recommends clang llvm pkg-config nettle-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build/

COPY ./Cargo.lock /build/Cargo.lock
COPY ./Cargo.toml /build/Cargo.toml
# Cargo needs to have an entry point (main.rs or lib.rs) present before it will fetch
COPY ./src/main.rs /build/src/main.rs
# fetch the dependecies up here. this should only run if deps change
RUN cargo fetch --locked

# Copy the source and buuild the project down here
COPY ./src /build/src
RUN cargo build --release --locked --all-features

RUN stat /build/target/release/aur-builder

FROM archlinux:latest AS runtime

# Enable multilib
RUN echo -e "\
[multilib] \n\
Include = /etc/pacman.d/mirrorlist \n\
" >> /etc/pacman.conf

RUN pacman -Syyu --noconfirm
RUN pacman -S --noconfirm pacman-contrib

# Download and rank a mirror list by speed
# TODO: If we build this on github, we are optimizing the mirrors for github servers, not where this will be used / downloaded
#   Should address a way to get better mirrorlist to more localize it
RUN curl -s "https://archlinux.org/mirrorlist/?country=US&protocol=http&protocol=https&ip_version=4&use_mirror_status=on" \
 |  sed -e 's/^#Server/Server/' -e '/^#/d'  \
 | rankmirrors -n 5 - > /etc/pacman.d/mirrorlist

RUN pacman -Syyu --noconfirm

RUN pacman -S base-devel base sudo git signify --noconfirm

# Create a build user to build the package
RUN useradd -m build

# Add build to the sudoers file
RUN echo 'build ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/build
RUN chmod 0440 /etc/sudoers.d/build

# Install Aurutils dependencies
RUN pacman -S expect diffstat jq pacutils wget devtools bash-completion perl-json-xs --noconfirm

# Build and install aurutils
WORKDIR /tmp/pkg/aurutils
RUN git clone https://aur.archlinux.org/aurutils.git

WORKDIR /tmp/pkg/aurutils/aurutils

RUN chown -R build:build .
USER build
RUN makepkg

USER root
RUN pacman -U *.pkg.* --noconfirm

WORKDIR /opt/aur-builder/
COPY --from=build /build/target/release/aur-builder /opt/aur-builder/aur-builder
RUN chmod a+x /opt/aur-builder/aur-builder
RUN ln -s /opt/aur-builder/aur-builder /usr/bin/aur-builder

ENTRYPOINT ["/opt/aur-builder/aur-builder"]
CMD ["docker", "--help"]
