# Składniki zewnętrzne

Własny kod HAIOS jest objęty Apache 2.0; [LICENSE](LICENSE) i [NOTICE](NOTICE) pozostają częścią dystrybucji. Licencji zależności nie zastępuje licencja projektu.

Limine 12.9.2: bootloader, główny kod BSD-2-Clause, protokół 0BSD; pełne LICENSE i 3RDPARTY.md w licenses/limine w paczce i ISO. Rust 1.99.0 / bibliotekę core opisują COPYRIGHT-library.html i teksty licencji w licenses/rust. Zależności mają także wyjątki LLVM oraz własne notices. Font DejaVu Sans Mono 2.37 (jeśli obecny w danym wydaniu): pochodne dane atlasu zachowują Bitstream Vera/DejaVu; licenses/dejavu/LICENSE, font-source.json i generator dokumentują pochodzenie. Font nie jest relabelowany na Apache 2.0.

Pliki zależności oraz notices są dołączone do każdej paczki i ISO; weryfikuje je tools/boot/verify-notices.py. [Budowanie](docs/build.md).
