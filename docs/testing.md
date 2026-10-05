# Testy i granice odbioru

Wydania pochodzą z odebranych prototypów QEMU q35/KVM x86_64, BIOS i UEFI. 0.1.0: 20 startów każdego firmware i godzinny test procesów/IPC/pamięci; 0.2.0: prawdziwe PS/2 i framebuffer, RAMFS/edytor, 20 startów i pięć minut mieszanych operacji; 0.3.0: trwałość, EIO/flush/RO/brak dysku, uszkodzenia/przerwania checkpointów, 10 startów każdego firmware i pięć minut mieszanych operacji. Późniejsze poprawki dodają rzeczywiste resety i piksele — oraz ×, z niezmienionymi starszymi glifami.

Publiczne pakiety są osobno sprawdzane po rozpakowaniu. Raport acceptance.json podaje dokładne sumy obrazu, wyniki i ograniczenia. Żaden wynik QEMU nie jest testem fizycznego laptopa ani gwarancją pracy produkcyjnej. Przykład regresji:

```sh
python3 tools/boot/test-console.py --out /tmp/haios-build
python3 tools/boot/test-console.py --out /tmp/haios-build --uefi
```

W 0.3.0 dodatkowo test-help.py, test-font-symbols.py oraz test-reboot.py. Testy używają własnych obrazów tymczasowych; nie używaj danych produkcyjnych.
