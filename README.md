# 👻 Emoji-Picker v1.1.2
![Rust](https://img.shields.io/badge/Rust-1.87-orange?logo=rust)
![GTK4](https://img.shields.io/badge/GTK-4.x-blue?logo=gnome)
![License](https://img.shields.io/badge/License-MIT-green?style=flat)
Ein schneller, einfacher Emoji-Picker für Linux (GTK-basiert, in Rust geschrieben).

## 💡 Features
* 🔎 Schnelle Live-Suche nach Emojis via Stichworte
* 📑 Kategorien über Tabs organisiert
* ⚙️ Einfache Konfiguration über das Einstellungsmenü
* ⌨️ Aufrufbar per Tastenkombination Super + .
* ✅ Erstkonfiguration beim ersten Start
* **🕔 Verlauf**: zuletzt genutzte Emojis, klickbar oder über Suche
* **⚙️ Konfigurierbar**:
  * ↕️ Größe der Emojis einstellbar
  * 🏡 Verhalten: Fenster schließen oder offen bleiben nach Auswahl/Drag’n’Drop
  * ⌨️ Shortcut erneut setzen über `→ Einstellungen → Tastenkürzel`
* **🪟 GTK4 + Cross‑Desktop**: Funktioniert unter gängigen Desktops wie GNOME, KDE, Cinnamon, XFCE, ...
* 🚀 Leichtgewichtig & ohne unnötige Abhängigkeiten

## 📸 Screenshots
#### Hauptfenster
![Emoji Picker GUI](screenshots/emoji-picker1.png)
#### Suchfunktion aktiv
![Suche aktiv](screenshots/emoji-picker2.png)
#### Optionsmenü
![Einstellungsfenster](screenshots/emoji-picker3.png)

## 🔧 Installation
### .deb-Paket (empfohlen für Debian, Mint, Tuxedo, Ubuntu):
```bash
sudo apt install ./emoji-picker_1.1.2_amd64.deb
```
### 💻 Manuell kompilieren:
```bash
git clone https://github.com/kai-thanner/emoji-picker.git
cd emoji-picker
cargo build --release
```
Die fertige Binärdatei liegt dann in `target/release/emoji-picker`

## 🛠 CLI‑Optionen
```bash
emoji-picker [OPTIONS]

Options:
  -h, --help      Hilfe anzeigen
  -V, --version   Versionsinfo (aktuelle Version: 1.1.2)
  -S, --setup     Tastenkombination einrichten
      --debug     Debug‑Logging aktivieren
```

## ⚙️ Konfiguration & Einstellungen
Beim ersten Start wird unter `~/.config/emoji-picker/settings.ini` automatisch eine
Konfigurationsdatei erstellt:
```ini
[Allgemein]
setup_erledigt = true          # Ob das Setup bereits durchgeführt wurde
fenster_schliessen = true      # Fenster nach Emoji-Auswahl automatisch schließen
fenster_offen_bei_drag = true  # Fenster bei Drag & Drop geöffnet lassen
emoji_size = 20                # Emoji-Größe in Pixeln
```
Die Werte lassen sich direkt in der Datei oder über das Einstellungsmenü ändern.

## 🎹 Tastenkombinationen im Emoji Picker
| Aktion                                    | Tastenkombination     |
| ----------------------------------------- | --------------------- |
| Emoji Picker starten                      | `Super` + `.`         |
| Nach Emojis suchen                        | Einfach lostippen     |
| erstes (oder ausgewähltes) Emoji kopieren | `Enter`               |
| Zwischen Kategorien wechseln              | `Tab`                 |
| Emoji mit Pfeiltasten auswählen           | `←` / `→` / `↑` / `↓` |
| Fenster schließen                         | `Esc`                 |
#### 🔍 Hinweise
  🔹 Die Suche beginnt automatisch beim Tippen – keine extra Maus nötig.  
  🔹 Die zuletzt genutzten Emojis findest du links oben im Verlauf.  
  🔹 Per Drag & Drop kannst du Emojis auch direkt in andere Programme ziehen.
#### 💡 Bonus-Tipp
Wenn du Drag & Drop nutzt, kannst du im Einstellungsfenster festlegen, ob das Picker-Fenster dabei offen bleiben soll.

## 📂 Speicherorte
| Datei/Ordner                                             | Beschreibung                    |
| -------------------------------------------------------- | ------------------------------- |
| `/usr/bin/emoji-picker`                                  | Ausführbare Datei               |
| `/usr/share/applications/emoji-picker.desktop`           | Eintrag im Startmenü            |
| `/usr/share/icons/hicolor/_x_/apps/emoji-picker.png` 	   | Icons 16x16 - 512x512           |
| `/usr/share/emoji-picker/`                               | .css Datei für GUI-Fenster      |
| `/etc/emoji-picker/`                                     | .list-Dateien als Vorlage       |
| `~/.config/emoji-picker/`                                | Nutzerdaten (History, Settings) |

## 🧩 Bekannte Einschränkungen
| Umgebung | Verhalten                            | Hinweis                                           |
| -------- | ------------------------------------ | ------------------------------------------------- |
| KDE      | GTK-Themes werden ggf. ignoriert     | Automatischer Fallback auf Breeze / Breeze-Dark   |
| KDE      | Tastenkombi wird nicht angelegt      | Shortcut nach Setup manuell setzen                |
| MATE     | Tastenkombi wird nicht angelegt      | Shortcut nach Setup manuell setzen                |

## 👨‍⚖️ Lizenz
Dieses Projekt steht unter der MIT-Lizenz. Siehe [LICENSE](LICENSE).

## 👨‍💻 Entwickler
> Erstellt von Kai Thanner