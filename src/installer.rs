use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

// Embeds the compiled standalone binary payload
const EXE_PAYLOAD: &[u8] = include_bytes!("../target/release/pdf-toolkit.exe");

fn main() {
    println!("============================================================");
    println!("       PDF TOOLKIT - INSTALLER (RUST NATIVE)");
    println!("============================================================");
    println!();

    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
    let install_dir = PathBuf::from(local_app_data).join("Programs").join("PDF-Toolkit");

    println!("[1/4] Erstelle Installationsverzeichnis...");
    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("Fehler beim Erstellen des Ordners: {}", e);
        pause_and_exit(1);
    }
    println!("      Pfad: {}", install_dir.display());

    println!("[2/4] Installiere Programmdateien (PDF Toolkit Standalone)...");
    let target_exe = install_dir.join("pdf-toolkit.exe");
    if let Err(e) = fs::write(&target_exe, EXE_PAYLOAD) {
        eprintln!("Fehler beim Schreiben der Binärdatei: {}", e);
        pause_and_exit(1);
    }
    println!("      Datei geschrieben ({} Bytes)", EXE_PAYLOAD.len());

    println!("[3/4] Erstelle Deinstallationsskript...");
    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
    let app_data = std::env::var("APPDATA").unwrap_or_default();
    let desktop_lnk = format!("{}\\Desktop\\PDF Toolkit.lnk", user_profile);
    let start_lnk = format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\PDF Toolkit.lnk", app_data);

    let uninstaller_script = format!(
        "@echo off\r\n\
title PDF Toolkit Deinstallation\r\n\
echo Deinstalliere PDF Toolkit...\r\n\
taskkill /F /IM pdf-toolkit.exe >nul 2>&1\r\n\
timeout /t 1 /nobreak >nul\r\n\
del /f /q \"{}\" >nul 2>&1\r\n\
del /f /q \"{}\" >nul 2>&1\r\n\
cd ..\r\n\
rmdir /s /q \"{}\" >nul 2>&1\r\n\
echo.\r\n\
echo [V] PDF Toolkit wurde vollstaendig entfernt.\r\n\
pause\r\n",
        desktop_lnk, start_lnk, install_dir.display()
    );
    let uninstall_bat = install_dir.join("uninstall.bat");
    let _ = fs::write(&uninstall_bat, uninstaller_script);

    println!("[4/4] Erstelle Windows-Verknuepfungen (Desktop & Startmenue)...");
    let ps_script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell\r\n\
$DesktopPath = [Environment]::GetFolderPath('Desktop')\r\n\
$StartMenuPath = [Environment]::GetFolderPath('Programs')\r\n\
$Shortcut1 = $WshShell.CreateShortcut(\"$DesktopPath\\PDF Toolkit.lnk\")\r\n\
$Shortcut1.TargetPath = '{exe}'\r\n\
$Shortcut1.WorkingDirectory = '{dir}'\r\n\
$Shortcut1.Description = 'PDF Toolkit - PDF24 Rust Alternative'\r\n\
$Shortcut1.Save()\r\n\
$Shortcut2 = $WshShell.CreateShortcut(\"$StartMenuPath\\PDF Toolkit.lnk\")\r\n\
$Shortcut2.TargetPath = '{exe}'\r\n\
$Shortcut2.WorkingDirectory = '{dir}'\r\n\
$Shortcut2.Description = 'PDF Toolkit - PDF24 Rust Alternative'\r\n\
$Shortcut2.Save()\r\n",
        exe = target_exe.display(),
        dir = install_dir.display()
    );

    let _ = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script])
        .output();

    println!();
    println!("============================================================");
    println!("  [OK] Installation erfolgreich abgeschlossen!");
    println!("       Desktop-Verknuepfung erstellt");
    println!("       Startmenue-Eintrag erstellt");
    println!("       Deinstaller: {}", uninstall_bat.display());
    println!("============================================================");
    println!();
    print!("Moechtest du PDF Toolkit jetzt direkt starten? [J/n]: ");
    let _ = io::stdout().flush();

    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    let input = line.trim();
    if input.is_empty() || input.eq_ignore_ascii_case("j") || input.eq_ignore_ascii_case("y") {
        let _ = Command::new("cmd")
            .args(["/C", "start", "", &target_exe.to_string_lossy()])
            .spawn();
    }
}

fn pause_and_exit(code: i32) -> ! {
    println!("\nDruecke die Eingabetaste zum Beenden...");
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    std::process::exit(code);
}
