# Bombowe roboty

Aktualna treść zadania dostępna jest [tutaj](https://docs.google.com/document/d/1t99N_8okRU4Dlyd9b8HL6TIW6wLmYICTwwwCBxLSitI/edit).

Pytania proszę wysyłać na adres agluszak@mimuw.edu.pl.

## Uruchamienie

Do uruchomienia programów potrzeba [kompilatora Rusta](https://rustup.rs/).

Po zainstalowaniu kompilatora należy wykonać komendę:
`cargo run --bin <gui/verifier>` i uzupełnić parametry.

## GUI

Interfejs graficzny dla gry Bombowe Roboty.
GUI prawdopodobnie będzie jeszcze aktualizowane.

### Sterowanie

```
W, strzałka w górę -  porusza robotem w górę.
S, strzałka w dół - porusza robotem w dół.
A, strzałka w lewo - porusza robotem w lewo.
D, strzałka w prawo - porusza robotem w prawo.
Spacja, J, Z - kładzie bombę.
K, X - blokuje pole.
```

## Weryfikator

Ten program pozwala sprawdzić, czy wiadomości są poprawnie serializowane.
Innymi słowy, jest to wzorcowy deserializator. Można łączyć się z nim zarówno po TCP, jak i UDP (z parametrem `-u`).
Przy uruchamianiu należy podać, jakiego rodzaju wiadomości mają być sprawdzane.