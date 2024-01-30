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
  -s, --strategy <STRATEGY>          Strategia przeszukiwania [default: bfs]
  -u, --url <URL>                    Adres URL do przeszukania
  -j, --jobs <JOBS>                  Maksymalna liczba wątków
  -c, --csv                          Czy zapisywać wyniki do pliku CSV
  -t, --timeout-secs <TIMEOUT_SECS>  Timeout przeszukiwania w sekundach
  -d, --max-depth <MAX_DEPTH>        Maksymalna głębokość przeszukiwania
  -D, --dot                          Generowanie wizualizacji grafu
  -h, --help                         Print help
  -V, --version                      Print version
```

## Podgląd wygenerowanego grafu
- https://edotor.net/
