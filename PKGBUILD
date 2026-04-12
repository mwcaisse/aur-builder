# Maintainer: Mitchell Caisse

pkgname=aur-builder
# TODO: Should paramaterize this to a degree and use timestamp versioning for now
pkgver=0.1.0
pkgrel=1
pkgdesc='AUR Wrapper to easily create, manage, and update a local repository of AUR packages'
url='https://github.com/mwcaisse/aur-builder'
license=("MIT")
makedepends=('cargo' 'nettle')
depends=()
arch=('x86_64')
source=("git+file://${PWD}")
b2sums=()

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target host-tuple
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features
}

package() {
    install -Dm0755 -t "${pkgdir}/usr/bin/" "target/release/${pkgname}"
    # for custom license, e.g. MIT
    install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"

    # Config file
    install -Dm644 sample-config.toml "${pkgdir}/etc/${pkgname}/config.toml"
}