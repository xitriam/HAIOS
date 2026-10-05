# HAIOS — Human × Artificial Intelligence Operating System

Eksperymentalny system operacyjny x86_64 napisany od podstaw w Rust, rozwijany przez Mateusza (xitriam) z udziałem narzędzi ChatGPT/Codex. Kernel nie bazuje na Linux/Windows/macOS. Warstwa AI jest planowana i jeszcze nie istnieje.

| Rola | Wersja | Zakres |
| --- | --- | --- |
| candidate | [0.3.0](https://github.com/xitriam/HAIOS/releases/tag/v0.3.0) | Trwałe pliki, sync, shutdown/reboot, credits, szczegółowy help |
| stable | [0.2.0](https://github.com/xitriam/HAIOS/releases/tag/v0.2.0) | Konsola graficzna tekstowa, PS/2, polskie znaki, edytor i pliki w RAM |
| old | [0.1.0](https://github.com/xitriam/HAIOS/releases/tag/v0.1.0) | Konsola szeregowa, pamięć, procesy i IPC |

Stable oznacza odebraną poprzednią wersję eksperymentalną, nie przydatność produkcyjną. 0.2.0 i 0.1.0 nie zachowują plików po wyłączeniu. Aktualne trzy role przesuwają się dopiero po odbiorze kolejnego wydania; archiwa i tagi są niezmienne.

[Wydania i pobieranie](https://github.com/xitriam/HAIOS/releases), [uruchomienie](docs/run.md), [budowanie](docs/build.md), [architektura](docs/architecture.md), [testy](docs/testing.md), [zasady dokumentacji](CONTRIBUTING.md), [licencje](THIRD_PARTY.md).

Własny kod: Apache 2.0. Limine, Rust i font DejaVu mają odrębne licencje/notices. Paczki zawierają ISO, ELF, pełne publiczne źródła, licencje i sumy SHA256. Wbudowany komunikat wersji może zawierać -dev: wszystkie trzy wydania są eksperymentalne.

Obsługiwane środowisko: QEMU q35/KVM x86_64, jeden CPU, 256 MiB, BIOS/UEFI. Bez bare metal, instalatora na fizyczny dysk, sieci gościa, GUI okienkowego, SMP, pełnego POSIX, FPU i AI. Nigdy nie przekazuj fizycznego dysku do gościa.

Źródła tego tagu: **0.2.0 (stable)**.
