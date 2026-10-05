//! Own Polish command reference; overview and detailed help share one catalog.
use crate::serial;
struct Topic {name: &'static str, description: &'static str, example: &'static str, syntax: &'static str, details: &'static str}
static TOPICS: &[Topic] = &[
    Topic {name:"help", description:"Pomoc dla poleceń", example:"help calc", syntax:"help [KOMENDA]", details:"Bez argumentu: spis poleceń z przykładami. Z nazwą: szczegóły i ograniczenia."},
    Topic {name:"version", description:"Wersja systemu", example:"version", syntax:"version", details:"Pokazuje wersję kernela i status eksperymentalny."},
    Topic {name:"credits", description:"Twórcy HAIOS", example:"credits", syntax:"credits", details:"Pokazuje autora projektu, udział narzędzi AI i składniki zewnętrzne."},
    Topic {name:"pwd", description:"Bieżący katalog", example:"pwd", syntax:"pwd", details:"Wyświetla pełną ścieżkę bieżącego katalogu."},
    Topic {name:"ls", description:"Zawartość katalogu", example:"ls /home", syntax:"ls [ŚCIEŻKA]", details:"Bez ścieżki pokazuje bieżący katalog. / oznacza katalog główny; [ro] oznacza tylko do odczytu."},
    Topic {name:"cd", description:"Zmiana katalogu", example:"cd /home", syntax:"cd ŚCIEŻKA", details:"Przykład względny: cd .. . Nazwy rozróżniają wielkie i małe litery: Proba i proba to różne katalogi."},
    Topic {name:"mkdir", description:"Nowy katalog", example:"mkdir /home/Proba", syntax:"mkdir ŚCIEŻKA", details:"Katalog nadrzędny musi istnieć. Zmiana trafia do RAM; sync utrwala ją na dysku."},
    Topic {name:"touch", description:"Utworzenie pustego pliku", example:"touch /home/notatka", syntax:"touch ŚCIEŻKA", details:"Istniejący plik zachowuje treść. Katalog nadrzędny musi istnieć. Wykonaj sync, aby utrwalić zmianę."},
    Topic {name:"cat", description:"Odczyt tekstu", example:"cat /home/notatka", syntax:"cat ŚCIEŻKA", details:"Pokazuje treść pliku UTF-8. cat about opisuje aktualny zakres systemu."},
    Topic {name:"write", description:"Zastąpienie treści pliku", example:"write /home/notatka Witaj", syntax:"write ŚCIEŻKA [TEKST]", details:"Tworzy plik lub zastępuje całą jego treść. Tekst może zawierać spacje; bez tekstu plik zostaje pusty. Potem sync."},
    Topic {name:"append", description:"Dopisanie tekstu", example:"append /home/notatka !", syntax:"append ŚCIEŻKA [TEKST]", details:"Dopisuje bez automatycznego nowego wiersza; tworzy plik, jeśli nie istnieje. Potem sync."},
    Topic {name:"edit", description:"Edytor tekstu", example:"edit /home/notatka", syntax:"edit ŚCIEŻKA", details:"Edytuje istniejący lub nowy plik. Limit 4096 bajtów. .show pokazuje treść; .clear czyści bufor.\n.line N TEKST zastępuje istniejący wiersz, np. .line 1 Witaj.\n.save zapisuje bufor do RAM; .cancel anuluje edycję. Po .save wykonaj sync.\nW edytorze help/reboot są tekstem pliku, nie poleceniami systemu."},
    Topic {name:"rm", description:"Usunięcie pliku lub pustego katalogu", example:"rm /home/notatka", syntax:"rm ŚCIEŻKA", details:"Nie usuwa niepustych katalogów ani /bin. Zmiana wymaga sync. Brak kosza i cofania."},
    Topic {name:"stat", description:"Wykorzystanie pamięci plików", example:"stat", syntax:"stat", details:"Pokazuje łączną liczbę węzłów (limit 32) i bajty treści. Nie przyjmuje ścieżki."},
    Topic {name:"disk", description:"Stan dysku VM", example:"disk", syntax:"disk", details:"Pokazuje dostępność dysku, generację checkpointu i dirty. dirty=1: zmiany czekają na sync."},
    Topic {name:"sync", description:"Zapis zmian na dysk", example:"sync", syntax:"sync", details:"sync utrwala pliki z RAM na dedykowanym dysku VM. Przy błędzie zmiany zostają w RAM.\nBrak automatycznego formatowania; /bin pozostaje tylko do odczytu."},
    Topic {name:"echo", description:"Wyświetlenie tekstu", example:"echo Witaj w HAIOS", syntax:"echo [TEKST]", details:"Uruchamia program użytkownika; wypisuje tekst i kończy się. Cudzysłowy są dosłowne; brak interpretacji powłoki."},
    Topic {name:"calc", description:"Obliczenia całkowite", example:"calc 12 + 3", syntax:"calc LICZBA OPERATOR LICZBA", details:"Operatory: + - * / . Przykład: calc -12 / 3 . Dwie liczby i jeden operator, bez nawiasów.\nDzielenie całkowite; błąd przy dzieleniu przez zero lub przekroczeniu zakresu i64."},
    Topic {name:"wc", description:"Liczba wierszy, słów i bajtów", example:"wc /home/notatka", syntax:"wc ŚCIEŻKA", details:"Pokazuje trzy liczby: wiersze (znaki LF), słowa i bajty. Polskie znaki mogą zajmować więcej niż jeden bajt."},
    Topic {name:"uptime", description:"Czas od uruchomienia", example:"uptime", syntax:"uptime", details:"Pokazuje czas pracy gościa; po reboot licznik zaczyna od nowa."},
    Topic {name:"mem", description:"Wolna pamięć systemu", example:"mem", syntax:"mem", details:"Pokazuje stan alokatora ramek pamięci. Nie jest to rozmiar dysku."},
    Topic {name:"ps", description:"Uruchomione procesy", example:"ps", syntax:"ps", details:"Pokazuje PID, nazwę i przełączenia procesów użytkownika. PID służy do kill."},
    Topic {name:"run", description:"Uruchomienie programu", example:"run hello", syntax:"run NAZWA [ARGUMENTY]", details:"Programs: hello count ipc fault writefault badptr spin fpu\nTakże: echo calc uptime wc. run ipc uruchamia parę sender/receiver.\nProgramy fault/writefault/badptr/fpu sprawdzają izolację błędów.\nrun spin działa do kill PID. Argumenty zależą od programu; np. run calc 2 + 3."},
    Topic {name:"kill", description:"Zatrzymanie procesu", example:"kill 1", syntax:"kill PID", details:"Najpierw ps, potem podaj rzeczywisty PID. Przykład 1 nie gwarantuje istniejącego procesu."},
    Topic {name:"clear", description:"Wyczyszczenie ekranu", example:"clear", syntax:"clear", details:"Czyści widok konsoli. Nie usuwa plików ani historii poleceń."},
    Topic {name:"shutdown", description:"Zapis i wyłączenie VM", example:"shutdown", syntax:"shutdown", details:"Zapisuje zmiany przed wyłączeniem gościa. Błąd zapisu pozostawia system uruchomiony.\nObecnie obsługuje VM QEMU q35; fizyczny komputer będzie wymagał osobnej obsługi i testów."},
    Topic {name:"reboot", description:"Zapis i restart VM", example:"reboot", syntax:"reboot", details:"Zapisuje zmiany, restartuje gościa i ponownie otwiera konsolę w tym samym oknie VM.\nBłąd zapisu zatrzymuje restart i zachowuje zmiany w RAM. Procesy i bieżący katalog są resetowane.\nObecnie QEMU q35; restart fizycznego komputera pozostaje przyszłym etapem."},
];
fn text(s:&str){serial::write(s.as_bytes());}
pub fn names(mut visit:impl FnMut(&str)){for t in TOPICS{visit(t.name);}}
pub fn show(name:Option<&str>){
 if let Some(name)=name {
  if let Some(t)=TOPICS.iter().find(|t|t.name==name){text(t.name);text(" — ");text(t.description);text("\nSkładnia: ");text(t.syntax);text("\nPrzykład: ");text(t.example);text("\n");text(t.details);text("\n");}
  else{text("ERROR help: nieznane polecenie; wpisz help\n");}
 }else{
  text("Pomoc HAIOS — opis i przykład każdego polecenia\nSzczegóły: help KOMENDA, np. help calc lub help edit.\nW składni [ARGUMENT] jest opcjonalny; nawiasów nie wpisuj.\n");
  for t in TOPICS{text(t.example);text(" — ");text(t.description);text("\n");}
  text("Ścieżki względne odnoszą się do pwd; wielkość liter ma znaczenie.\nStrzałki: edycja/historia; Tab: uzupełnianie. AltGr: polskie znaki.\nPo zmianie plików wykonaj sync; shutdown/reboot zapisują przed końcem.\n");
 }
}
