# 👻 Emoji-Picker v1.1.0

Ein schneller, einfacher Emoji-Picker für Linux (GTK-basiert, in Rust geschrieben).

## 💡 Features

* 🔎 Schnelle Live-Suche nach Emojis via Stichworte
* 📑 Kategorien über Tabs organisiert
* 📥 Klick oder Enter kopiert Emoji direkt in Zwischenablage
* ⚙️ Einfache Konfiguration über das Einstellungsmenü
* ⌨️ Aufrufbar per Tastenkombination (Super+.)
* ✅ Erstkonfiguration beim ersten Start
* 🚀 Leichtgewichtig & ohne unnötige Abhängigkeiten

---

## 📸 Screenshots

### Hauptfenster

![Emoji Picker GUI](screenshots/emoji-picker1.png)

### Suchfunktion aktiv

![Suche aktiv](screenshots/emoji-picker2.png)

### Optionsmenü

![Suche aktiv](screenshots/emoji-picker3.png)

---

## 🔧 Installation

### .deb-Paket (empfohlen für Debian, Mint, Tuxedo, Ubuntu):

```bash
sudo apt install ./emoji-picker_1.1.0_amd64.deb
```

### 💻 Manuell kompilieren:

```bash
git clone https://github.com/kai-thanner/emoji-picker.git
cd emoji-picker
cargo build --release
```

Die fertige Binärdatei liegt dann in `target/release/emoji-picker`

---

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

---

## ⌨️ Tastenkombination einrichten

Falls beim ersten Start keine Tastenkombination gesetzt wurde:
getestet unter: Linux Mint 22.1 Cinnamon, Xfce, Mate, Kde, Gnome
Manuelle Konfig nötig: Mate & Kde

```bash
emoji-picker --setup	Alternativ im UI -> Einstellungen
```

Diese legt die tastenkombination Super+. für den Emoji-Picker an.

---

## 📂 Speicherorte

| Datei/Ordner                                             | Beschreibung                    |
| -------------------------------------------------------- | ------------------------------- |
| `/usr/bin/emoji-picker`                                  | Ausführbare Datei               |
| `/usr/share/applications/emoji-picker.desktop`           | Eintrag im Startmenü            |
| `/usr/share/icons/hicolor/_x_/apps/emoji-picker.png` 	   | Icons 16x16 - 512x512           |
| `/usr/share/emoji-picker/`                               | .css Datei für GUI-Fenster      |
| `/etc/emoji-picker/`                                     | .list-Dateien als Vorlage       |
| `~/.config/emoji-picker/`                                | Nutzerdaten (History, Settings) |

---

## 👨‍⚖️ Lizenz

Dieses Projekt steht unter der MIT-Lizenz. Siehe [LICENSE](LICENSE).

## 👨‍💻 Entwickler

> Erstellt von Kai Thanner

```markdown
![Rust](https://img.shields.io/badge/Rust-1.87-orange?logo=rust)
![GTK4](https://img.shields.io/badge/GTK-4.x-blue?logo=gnome)