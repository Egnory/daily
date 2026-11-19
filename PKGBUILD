pkgname=daily
pkgver=0.1.0
pkgrel=1
arch=('x86_64')
url="https://github.com/Egnory/daily"
license=('unknown')
options=('!lto' '!debug')

depends=(
    'gcc-libs'
    'glibc'
    'fontconfig'
    'libxcb'
    'libxkbcommon'
    'libxkbcommon-x11'
)

makedepends=(
    'git'
    'rust'
    'pkgconf'
)

source=("git+https://github.com/Egnory/daily.git#branch=master")
sha256sums=('SKIP')

build() {
    cd "$srcdir/daily"
    cargo build --release
}

package() {
    cd "$srcdir/daily"

    install -Dm755 target/release/daily \
        "$pkgdir/usr/bin/daily"

    install -Dm644 app.ico \
        "$pkgdir/usr/share/pixmaps/daily.ico"

    install -Dm644 /dev/stdin \
        "$pkgdir/usr/share/applications/daily.desktop" <<'EOF'
[Desktop Entry]
Name=Daily
Comment=Daily tracking application
Type=Application
Exec=daily
Icon=/usr/share/pixmaps/daily.ico
Terminal=false
Categories=Utility;
StartupNotify=true
EOF
}
