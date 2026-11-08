# PDF Werkzeugkasten (PDF Toolkit Desktop)

100% native Rust Desktop-Applikation zur vollständigen Offline-Bearbeitung und Verwaltung von PDF-Dateien. Keine Web-App, kein Electron, keine Cloud-Abhängigkeiten.

## Funktionsübersicht

1. **📄 Zusammenführen (Merge)**: Beliebig viele PDFs in wählbarer Reihenfolge zu einem Dokument verbinden.
2. **✂️ Teilen & Extrahieren (Split)**:
   - Bestimmte Seitenbereiche (z.B. `1-3, 5, 8-10`) in eine neue PDF extrahieren.
   - Jede einzelne Seite automatisch als separate Datei exportieren.
3. **🔄 Seiten Drehen (Rotate)**: Alle oder ausgewählte Seiten um 90°, 180° oder 270° drehen.
4. **🗑️ Seiten Löschen (Delete)**: Überflüssige Seiten aus Dokumenten entfernen und Dokumentenstruktur bereinigen.
5. **🗜️ Komprimieren (Compress & Optimize)**: Dateigröße mit Object-Streams und Cross-Reference-Streams (Level 9) drastisch verkleinern.
6. **🖼️ Bilder zu PDF (Images to PDF)**: Bilder (JPG, PNG, BMP) in ein mehrseitiges PDF umwandeln.
7. **🔍 Bilder Extrahieren (Extract Images)**: Alle eingebetteten Bild-Objekte (JPG / PNG) aus PDFs exportieren.
8. **🔒 Verschlüsseln & Schützen (Security)**: Öffnungs- und Besitzer-Passwort mit AES-128 oder AES-256 vergeben.
9. **🔓 Entschlüsseln (Decrypt)**: Passwortgeschützte PDFs entsperren und ohne Kennwort abspeichern.
10. **📝 Metadaten-Editor**: Titel, Autor, Thema, Stichwörter, Ersteller und Producer lesen und anpassen.
11. **📊 Analyse & Text-Extraktion**: Dokumenten-Details (Objekte, Version, Verschlüsselung) und Text aus Seiten auslesen.

## Technische Details

- **Sprache**: 100% Rust (Edition 2024)
- **GUI-Framework**: `eframe` / `egui` (native GPU-beschleunigte Desktop-Oberfläche, 0% WebView)
- **PDF-Engine**: `lopdf` mit `embed_image` Feature
- **Bildverarbeitung**: `image`
- **Native Datei-Dialoge**: `rfd` (Windows API FileDialogs)

## Windows Installer & Starten

### Installer erstellen & ausführen:
```powershell
.\build-installer.ps1
.\dist\PDF_Werkzeugkasten_Setup_v1.0.0.exe
```

### Manuell starten (ohne Installer):
```powershell
cargo run --release
```
