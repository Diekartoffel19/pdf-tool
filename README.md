# PDF Toolkit (Rust Native Edition)

Ein extrem leichtgewichtiges (nur 1.8 MB Einzeldatei), schnelles und 100% lokales PDF-Werkzeug nach dem Vorbild von PDF24.

## Vorteile der Rust-Version
- **Single-Binary**: Nur eine einzige ausführbare Datei `pdf-toolkit-rust.exe` (1.8 MB), HTML/CSS/JS sind direkt im Binary eingebettet.
- **Keine Abhängigkeiten**: Kein Python, kein Node.js, keine externen Runtimes nötig.
- **Minimale Ressourcennutzung**: Benötigt im Betrieb weniger als 15 MB Arbeitsspeicher.
- **Dark Mode**: Standardmäßig aktives dunkles Theme mit schnellem Umschalter.
- **100% Lokal**: Sämtliche PDF-Verarbeitung geschieht nativ auf deinem PC.

## Enthaltene Funktionen
1. PDF zusammenfügen (Merge)
2. PDF teilen & extrahieren (Split in Bereiche oder alle Seiten als ZIP)
3. Seiten verwalten & drehen (Visuelle Thumbnails, 90°-Drehung, Verschieben, Löschen)
4. PDF komprimieren (Stream-Deflation & Objekt-Pruning)
5. Bilder zu PDF (JPEG zu A4-PDF)
6. Wasserzeichen einfügen
7. Seitenzahlen nummerieren
8. Text extrahieren (.txt)

## Starten
Einfach die Datei `start.bat` doppelklicken oder direkt die Binärdatei ausführen:
```powershell
.\target\release\pdf-toolkit-rust.exe
```
Der Standardbrowser öffnet sich automatisch unter `http://127.0.0.1:8080`.
