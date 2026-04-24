# Maintainer: Mitchell Caisse

pkgname=aur-builder
# TODO: Should paramaterize this to a degree and use timestamp versioning for now
pkgver=0.1.0
pkgrel=1
pkgdesc='AUR Wrapper to easily create, manage, and update a local repository of AUR packages'
url='https://github.com/mwcaisse/aur-builder'
license=("MIT")
makedepends=('cargo' 'nettle' 'clang')
depends=()
arch=('x86_64')
source=("aur-builder::git+file://${PWD}")
b2sums=()
backup=("etc/${pkgname}/config.toml")

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target host-tuple
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
    ls
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    # Only run unit tests
    cargo test --bin aur-builder --frozen --all-features
}

package() {

    echo "SRCDIR=${srcdir}"
    echo "PWD=$(pwd)"
    ls -l
    ls -l "${srcdir}"

    install -Dm0755 -t "${pkgdir}/usr/bin/" "target/release/${pkgname}"
    # for custom license, e.g. MIT
    install -Dm644 "${pkgname}/LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"

    # Config file
    install -Dm644 "${pkgname}/sample-config.toml" "${pkgdir}/etc/${pkgname}/config.toml"
}