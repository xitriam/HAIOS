# Uruchomienie

Środowisko sprawdzone: Linux x86_64, QEMU/KVM (wersja testowa 10.0.13), Python 3.13. Zainstaluj qemu-system-x86, ovmf i python3. Pobierz paczkę z Releases, rozpakuj i w jej katalogu wykonaj:

```sh
sha256sum -c SHA256SUMS
python3 source/tools/boot/run-console.py --out bin
# BIOS jest domyślny; UEFI:
python3 source/tools/boot/run-console.py --out bin --uefi
```

Wymaga uprawnień do /dev/kvm; launchery nie używają sudo. UEFI korzysta z plików /usr/share/OVMF/OVMF_CODE_4M.fd i OVMF_VARS_4M.fd. Nie deklarujemy testów na innych platformach.

0.1.0 ma konsolę szeregową. W 0.2.0/0.3.0 dostępny jest też run-window.py: obraz QEMU przez uwierzytelnione VNC związane z 127.0.0.1; hasło sesji pokazuje launcher. Podłącz lokalnego klienta VNC, nie wystawiaj portu do Internetu. VNC jest transportem gospodarza, nie siecią HAIOS.

Wpisz help. W 0.3.0: help calc, help edit, credits; write /home/proba Witaj, sync, reboot, cat /home/proba. Shutdown zapisuje przed zamknięciem; reboot zapisuje przed resetem. Błąd zapisu zachowuje działający system. Dysk bin/haios-data.img jest czystym obrazem 64 MiB; używaj tego samego obrazu między startami, NIE twórz go ponownie. Jego suma zmienia się po zapisach. Kopię zapasową rób przy wyłączonej VM. /bin jest tylko do odczytu, plik ma limit 4096 bajtów, RAMFS 32 węzły.

0.2.0/0.1.0 nie mają trwałego dysku. Ctrl+A, potem X zamyka QEMU bez synchronizacji; w 0.3.0 preferuj shutdown. Reboot/shutdown obecnie obsługują VM q35, nie fizyczny komputer. Format eksperymentalny bez szyfrowania, ACL lub gwarancji dla danych produkcyjnych.
