# mac-suite

App macOS native scritte in Rust (AppKit via `objc2`, niente WebView). Gratis e open source.

## Installazione dalla release
1. Scarica lo zip da [Releases](../../releases) ed estrailo.
2. Sposta l'app in **Applicazioni** e aprila.
3. Al primo avvio macOS la blocca: Impostazioni di Sistema › Privacy e sicurezza › **Apri comunque**. Serve una volta sola.

Richiede macOS 13+ e Apple Silicon.

## Compilazione dal sorgente (nessun avviso)
```bash
scripts/bundle.sh hello Hello && cp -R target/release/Hello.app /Applications/
```
