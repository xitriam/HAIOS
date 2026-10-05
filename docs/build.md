# Budowanie źródeł tagu

Linux x86_64; Python 3.13, rustup z Rust 1.99.0 i target x86_64-unknown-none, Limine 12.9.2 binary, GnuPG, make, gcc, xorriso, QEMU/KVM i OVMF. Na Debianie pakiety: build-essential xorriso qemu-system-x86 ovmf python3 gnupg ca-certificates curl git. Wymagany podpis i sumy przypiętego Limine; narzędzie build ponownie porównuje wejścia z przypiętym archiwum.

```sh
python3 tools/provisioning/prepare-toolchain-user.py
python3 tools/boot/build.py --out /tmp/haios-build
python3 tools/boot/verify-notices.py --out /tmp/haios-build
```

Skrypt toolchain instaluje przypięte narzędzia w katalogu aktualnego użytkownika (~/.cargo, ~/.rustup, ~/.local/share/haios/toolchain). Przeczytaj go przed użyciem; nie wymaga sudo. Kompilacja musi mieć output poza drzewem źródeł. Narzędzia i notices pochodzą z oficjalnych wydań upstream. Build zapisuje wejścia i sumy ISO/ELF w build.json. ISO zawiera znaczniki czasu; identyczność ISO nie jest obiecana.
