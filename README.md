# mac-suite

App macOS native scritte in Rust (AppKit via `objc2`, niente WebView). Gratis e open source.

> **Stato: in sviluppo iniziale.** Nessuna app è ancora alla pari con quella Apple che sostituisce. Qui sotto c'è cosa funziona davvero e cosa manca.

## App

| App | Sostituisce | Fase | Stato |
|---|---|---|---|
| Hello | — | 0 | ✅ Solo verifica del toolchain (finestra, menu, bundle, firma) |
| Monitor | Monitoraggio Attività | 1 | 🟡 v0.1 — vedi sotto |
| Terminale | Terminal | 1 | ⏳ Da fare |
| Editor | TextEdit | 2 | ⏳ Da fare |
| Note | Note | 2 | ⏳ Da fare |
| Launcher | Spotlight (avvio app) | 2 | ⏳ Da fare |
| Password | Password / Portachiavi | 3 | ⏳ Da fare |
| Backup | Time Machine (copie file) | 3 | ⏳ Da fare |
| Mail | Mail | 4 | ⏳ Da fare |
| Musica | Musica (file locali) | 4 | ⏳ Da fare |
| Browser | Safari (motore Servo) | 5 | ⏳ Da fare, sperimentale |

### Monitor v0.1
Funziona:
- elenco processi con nome, % CPU, memoria, PID, aggiornato ogni 2 s;
- ordinamento per colonna, larghezze e ordine colonne ricordati;
- **Processo › Termina processo** (⌥⌘Q) e **Forza uscita** (⌥⇧⌘Q) con conferma;
- ricerca per nome o PID, con conteggio "M di N processi" nel sottotitolo;
- totale processi e CPU nel sottotitolo; Dark Mode e VoiceOver da AppKit.

Manca rispetto a Monitoraggio Attività: schede Energia, Disco, Rete; campionamento processo; icona del Dock con grafico; storico e avvisi soglia; albero processi.
Nota: la ricerca per nome vede il nome dell'eseguibile (un processo avviato con `exec -a` può non comparire con il nome dichiarato).

## Limiti noti (di tutta la suite)
- Le app non diventano "di sistema": si affiancano a quelle Apple, non le rimpiazzano.
- Niente Developer ID: al primo avvio serve **Apri comunque** (vedi sotto).
- Testato solo su Apple Silicon, macOS 13+.
- Interfaccia per ora solo in italiano.

## Installazione dalla release
1. Scarica lo zip da [Releases](../../releases) ed estrailo.
2. Sposta l'app in **Applicazioni** e aprila.
3. Al primo avvio macOS la blocca: Impostazioni di Sistema › Privacy e sicurezza › **Apri comunque**. Serve una volta sola.

## Compilazione dal sorgente (nessun avviso)
Serve Rust stabile (`rustup`).
```bash
scripts/bundle.sh monitor Monitor && cp -R target/release/Monitor.app /Applications/
scripts/bundle.sh hello Hello && cp -R target/release/Hello.app /Applications/
```
Test: `cargo test --workspace`.

## Struttura
- `crates/appkit-shell` — avvio app, menu standard, finestra principale.
- `apps/<nome>` — una cartella per app.
- `scripts/bundle.sh` — crea il `.app` firmato ad-hoc.
