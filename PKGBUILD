# Maintainer: Aram Drevekenin <aram@poor.dev>
pkgname=what
pkgver=0.3.2
pkgrel=1
makedepends=('rust' 'cargo')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
pkgdesc="Display network utilization by process, connection and remote address"
url="https://github.com/imsnif/what"
source=("https://github.com/imsnif/$pkgname/archive/$pkgver.tar.gz")
license=('MIT')
sha256sums=("5097eb6e25c668c057444bf3be7bc3f98e4164a93bb17a0a025ebe1b0866455a")

check() {
  cd "$pkgname-$pkgver"
  cargo test --release --locked
}

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/what" "$pkgdir/usr/bin/what"
  install -Dm644 "docs/what.1" "$pkgdir/usr/share/man/man1/what.1"
  install -Dm644 LICENSE.md "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
