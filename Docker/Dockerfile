FROM archlinux:latest

# Enable multilib
RUN echo -e "\
[multilib] \n\
Include = /etc/pacman.d/mirrorlist \n\
" >> /etc/pacman.conf

RUN pacman -Syyu --noconfirm
RUN pacman -S --noconfirm pacman-contrib

# Download and rank a mirror list by speed
RUN curl -s "https://archlinux.org/mirrorlist/?country=US&protocol=http&protocol=https&ip_version=4&use_mirror_status=on" \
 |  sed -e 's/^#Server/Server/' -e '/^#/d'  \
 | rankmirrors -n 5 - > /etc/pacman.d/mirrorlist

RUN pacman -Syyu --noconfirm

RUN pacman -S base-devel base sudo git signify --noconfirm

# Install Aurutils dependencies
RUN pacman -S expect diffstat jq pacutils wget devtools bash-completion perl-json-xs --noconfirm
# Build and install aurutils

WORKDIR /tmp/pkg/aurutils
RUN git clone https://aur.archlinux.org/aurutils.git

WORKDIR /tmp/pkg/aurutils/aurutils

# Create a build user to build the package
RUN useradd -m build
RUN chown -R build:build .
USER build
RUN makepkg

USER root
RUN pacman -U *.pkg.* --noconfirm

WORKDIR /app
COPY build_packages.sh .

ENTRYPOINT ["bash", "build_packages.sh"]
