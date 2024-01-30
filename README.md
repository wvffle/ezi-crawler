## Wymagania
- Chromium

## Budowanie 
```sh
cargo build
```

## Uruchomienie
```
Usage: ezi [OPTIONS] --url <URL>

Options:
  -u, --url <URL>                    Adres URL do przeszukania
  -S, --strategy <STRATEGY>          Strategia przeszukiwania [default: bfs]
  -t, --timeout-secs <TIMEOUT_SECS>  Timeout przeszukiwania w sekundach
  -d, --max-depth <MAX_DEPTH>        Maksymalna głębokość przeszukiwania
  -c, --csv                          Czy zapisywać wyniki do pliku out.csv
  -D, --dot                          Czy zapisywać wizualizację grafu do pliku out.dot
  -U, --user-agent <USER_AGENT>      Własny User-Agent
  -j, --jobs <JOBS>                  Maksymalna liczba wątków
  -H, --headful                      Czy uruchomić przeglądarkę w trybie headful
  -s, --silent                       Czy wyświetlać logi
  -h, --help                         Print help
  -V, --version                      Print version
```

## Podgląd wygenerowanego grafu
- https://edotor.net/
