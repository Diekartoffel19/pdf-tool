# Master-Prompt: Leichtgewichtiges PDF-Editor Programm in Rust (PDF24 Alternative)

Dieser Prompt kann in jede moderne KI eingegeben werden, um dieses native, extrem leichtgewichtige PDF-Toolkit in Rust jederzeit vollständig und reproduzierbar zu bauen:

---

```markdown
Erstelle ein natives, ultra-leichtgewichtiges PDF-Editor-Programm in Rust (nach dem Vorbild von PDF24) mit vollem Dark-Mode-Support, eingebetteten Assets (Single Binary) und ohne Cloud-Zwang.

### 1. Technische Architektur & Vorgaben:
- **Sprache & Toolchain**: Rust 2021 Edition, Cargo.
- **Server**: `tiny_http`. Synchroner, multi-threaded HTTP-Server ohne asynchrone Runtime-Komplexität (kein riesiges Tokio/Actix nötig), extrem schneller Build und minimale RAM-Nutzung (< 15 MB).
- **PDF-Engine**: `lopdf` (reines Rust für PDF-Manipulation: Laden, Speichern, Seiten drehen, Seiten renummerieren, Seiten löschen, Streams deflaten, Pruning, Text extrahieren).
- **Archivierung & Codierung**: `zip` mit deflate für Multi-File-Exports; `base64` und `serde`/`serde_json` für saubere API-Kommunikation.
- **Single-Binary-Auslieferung**: HTML (`index.html`), CSS (`style.css`) und JavaScript (`app.js`) werden via `include_str!` zur Compile-Zeit direkt in die Binärdatei eingebettet.
- **Browser-Auto-Launch**: Die Anwendung startet beim Ausführen automatisch den Standardbrowser unter `http://127.0.0.1:8080`.

### 2. Frontend & Dark Mode:
- **Design**: Modernes PDF24-Kachel-Dashboard mit SVG/Emoji-Icons und Badges.
- **Dark Mode**: Standardmäßig aktive Dark-Theme-Palette (#0b1120, #1e293b, Akzent #f97316 Rust-Orange) mit flüssigem Umschalter auf Light Mode in der Kopfzeile (gespeichert in `localStorage`).
- **Drag-and-Drop**: Interaktive Uploadzonen mit optischem Hover-Effekt.
- **Visuelle Vorschau**: Schnelle clientseitige Thumbnail-Generierung im Canvas via `pdf.js` für interaktives Drehen (↻), Umsortieren (◀ ▶) und Löschen (✕).
- **Download-Pipeline**: Automatisches Entgegennehmen und Triggern von Datei-Downloads über Data-Blobs.

### 3. Enthaltene PDF-Funktionen:
1. **PDF zusammenfügen (Merge)**: Mehrere PDF-Dateien renummerieren, Page-Trees zusammenführen und als ein Dokument exportieren.
2. **PDF teilen (Split)**: Seitenbereiche (z. B. "1-3, 5") oder jede Seite einzeln als ZIP exportieren.
3. **Seiten verwalten (Organize)**: Seiten rotieren (`/Rotate`), Reihenfolge ändern (`Kids`-Array) und Seiten löschen.
4. **PDF komprimieren (Compress)**: Stream-Deflation, Objekt-Pruning und Producer-Bereinigung. Zeigt Ersparnis in % an.
5. **Bilder zu PDF (Images to PDF)**: JPEG-Dateien in ein standardisiertes A4-PDF mit Image-XObjects einbetten.
6. **Wasserzeichen (Watermark)**: Text semi-transparent und diagonal auf alle Seiten stempeln.
7. **Seitenzahlen (Page Numbers)**: Position (z. B. Unten Mitte) und Format (z. B. "Seite X von Y") einfügen.
8. **Text extrahieren (Extract Text)**: Gesamten lesbaren Text extrahieren und als `.txt` herunterladen.

### 4. Lieferumfang:
- `Cargo.toml`: Konfiguration aller Dependencies.
- `src/main.rs`: Embedded HTTP Server, Routing, Browser-Launcher.
- `src/pdf_ops.rs`: Pure-Rust PDF-Manipulationslogik mit `lopdf`.
- `static/index.html`, `static/style.css`, `static/app.js`: Das eingebettete Dashboard.
- `start.bat`: Windows One-Click-Starter.
```
