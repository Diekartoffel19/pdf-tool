# PDF Werkzeugkasten Pro (PDF Toolkit Desktop)

100% native Rust Desktop-Applikation zur vollständigen Offline-Bearbeitung, Strukturierung, Stempelung, Bereinigung und Verwaltung von PDF-Dateien. Keine Web-App, kein Electron, keine Cloud-Abhängigkeiten.

## Funktionsübersicht (14 Werkzeuge)

### Seiten & Struktur
1. **📄 Zusammenführen & Duplex-Mischer (Merge & Interleave)**:
   - Beliebig viele PDFs in wählbarer Reihenfolge zu einem Dokument verbinden.
   - **Duplex-Scanner Modus**: Vorderseiten-PDF und Rückseiten-PDF alternierend mischen (inkl. automatischer Rückwärts-Sortierung der Rückseiten).
2. **✂️ Teilen & Extrahieren (Split)**:
   - Bestimmte Seitenbereiche (z.B. `1-3, 5, 8-`, `gerade`, `ungerade`) extrahieren.
   - Dokument alle *N* Seiten in Teil-PDFs aufteilen.
   - Jede Seite einzeln als PDF exportieren.
3. **🗂️ Seiten-Manager (Organize)**:
   - Seiten in beliebiger Reihenfolge neu sortieren oder duplizieren (z.B. `3, 1, 5-2, 1`).
   - Seitenreihenfolge mit 1 Klick komplett umkehren (`N..1`).
   - Leerseiten (DIN A4 Hoch/Quer) an beliebiger Position einfügen.
4. **🔄 Seiten Drehen (Rotate)**: Alle, gerade, ungerade oder ausgewählte Seiten um 90°, 180° oder 270° drehen.
5. **🗑️ Seiten Löschen (Delete)**: Überflüssige Seiten (oder alle geraden/ungeraden Seiten) entfernen und Struktur bereinigen.

### Stempel, Nummern & Layout
6. **💧 Wasserzeichen & Stempel (Watermark)**:
   - Vorlagen (`VERTRAULICH`, `ENTWURF`, `KOPIE`, `BEZAHLT`, `ORIGINAL`) oder freier Text.
   - Diagonal (45°) oder horizontal, frei wählbare Schriftgröße, Farbe und Ebene (Vorder-/Hintergrund).
7. **🔢 Seitenzahlen & Bates-Nummerierung (Page Numbers)**:
   - Formate: `Seite 1 von N`, `1 / N`, `1, 2, 3` oder `Bates (z.B. DOK-0001)`.
   - 6 Positionen (Oben/Unten, Links/Mitte/Rechts), Startnummer und Option zum Überspringen des Deckblatts.
8. **📐 Zuschneiden & Seitenformat (Crop & Scale)**:
   - Seitenränder millimetergenau beschneiden (Oben, Unten, Links, Rechts in mm).
   - Seiten samt Inhalt proportional auf DIN A4 (Hoch/Quer), DIN A3, DIN A5 oder US Letter skalieren.

### Bilder & Optimierung
9. **🗜️ Komprimieren (Compress & Optimize)**: Dateigröße mit Object-Streams und Cross-Reference-Streams (Level 9) verkleinern.
10. **🖼️ Bilder zu PDF (Images to PDF)**: Bilder (JPG, PNG, BMP) in Originalgröße oder eingepasst auf DIN A4 (Hoch/Quer) mit konfigurierbarem Rand umwandeln.
11. **🔍 Bilder Extrahieren (Extract Images)**: Alle eingebetteten Bild-Objekte verlustfrei als JPG / PNG exportieren.

### Schutz, Datenschutz & Analyse
12. **🔒 Passwort & Rechte (Security)**:
    - AES-128 / AES-256 Verschlüsselung mit Öffnungs- und Besitzer-Passwort.
    - Granulare Berechtigungen (Drucken, Kopieren, Bearbeiten, Kommentare/Formulare).
    - Passwortschutz bestehender PDFs aufheben (Decrypt).
13. **🛡️ Datenschutz-Reiniger (PDF Sanitizer)**:
    - Entfernt Metadaten, XMP-Streams, Kommentare/Annotationen, Formularfelder, Lesezeichen, JavaScript/Aktionen und verwaiste PDF-Objekte.
14. **📝 Metadaten-Editor & 📊 Analyse / Textsuche**:
    - Titel, Autor, Thema, Stichwörter, Creator und Producer bearbeiten oder leeren.
    - Tiefenanalyse (DIN-Formaterkennung, Bilder-/Schriften-Zähler, Objektanzahl) und Volltext-Extraktion mit Wortzähler, Live-Suchfilter und Zwischenablage-Export.

## Starten & Kompilieren

```powershell
# Entwicklungsmodus starten
cargo run

# Optimierte Release-Version bauen
cargo build --release
.\target\release\pdf-toolkit-rust.exe

# Tests ausführen
cargo test
```
