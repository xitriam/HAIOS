# Architektura

Własny kernel Rust no_std, Limine BIOS/UEFI, jeden CPU x86_64. Bitmapowy PMM stron 4 KiB, VMM i osobne przestrzenie ring 3, wyjątki, timer, scheduler, ograniczone IPC i syscall int 0x80. Programy są wbudowanymi małymi obrazami, nie dynamicznie ładowanym ELF. Kernel jest deterministyczny; przyszła AI działa poza jego granicą zaufania. Warstwa AI nie jest zaimplementowana.

0.1.0: COM1 i konsola procesów. 0.2.0: framebuffer RGB, legalny atlas DejaVu, PS/2, AltGr, edycja/historia/Tab, RAMFS i narzędzia echo/calc/uptime/wc. 0.3.0: VirtIO-blk transitional PCI i dedykowany obraz danych 64 MiB. Dwa checkpointy CRC/generacji; sync unieważnia nieaktywny nagłówek, zapisuje payload i zatwierdza nagłówek, z flush między etapami. Niekompletny checkpoint jest odrzucany; nieznany dysk nie jest automatycznie formatowany. Shutdown/reboot wymagają udanego zapisu dirty cache. Restart fizyczny pozostaje przyszłym etapem.

Brak sieci gościa, AI, bare metal, SMP, pełnego POSIX, pełnego W^X kernela i wielu użytkowników. [Zasady pracy](../CONTRIBUTING.md), [testy i granice](testing.md).
