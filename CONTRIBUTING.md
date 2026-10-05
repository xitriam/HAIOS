# Współpraca i dokumentacja

Każda zmiana: cel i zakres w Issue → kod → odpowiedni test → aktualna dokumentacja → commit/PR. Dokumentacja jest częścią produktu. PR podaje zachowanie, testy, ograniczenia i aktualizowane dokumenty. Decyzje architektury wymagają zapisu ADR. Nie zapisuj sekretów, prywatnych danych ani audytów sieci w publicznym repo. Nie przedstawiaj niewykonanych testów lub planów jako działających funkcji.

Wydanie wymaga źródeł, odtwarzalnej instrukcji builda, własnego zakresu, wyników testów, notices i sum kontrolnych. Candidate/stable/old to role trzech aktualnych wersji; promocja wymaga odbioru. Tagi i artefakty wydanych wersji pozostają niezmienne.

## Rozwój prywatny, publikacja publiczna

Właściciel przyjął strategię: rozwój, testy i audyty w repo prywatnym. Następnie kontrola informacji, czysty zestaw produktu i publikacja w publicznym xitriam/HAIOS. Nie kopiujemy historii audytów, danych osobistych, adresów sieci, kluczy, prywatnych launcherów ani dysków użytkownika. Każde wydanie ma dokładne publiczne źródła tagu, nowe buildy, raport oraz sumy. Oryginały prywatnych paczek zachowujemy.
