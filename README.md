# 👻 Emoji-Picker v1.0.0

Ein schneller, einfacher Emoji-Picker für Linux (GTK-basiert, Rust).

---

## 📈 Features

* ✨ Schnelle Live-Suche nach Emojis via Stichworte
* 📂 Kategorien über Tabs organisiert
* 📋 Klick oder Enter kopiert Emoji direkt in Zwischenablage
* 🔝 Einfache Konfiguration über settings.ini
* 🍀 Aufrufbar per Tastenkombination (z. B. Super+.)
* ✅ Erstkonfiguration beim ersten Start
* 🚀 Leichtgewichtig & ohne unnötige Abhängigkeiten

---

## 🔧 Installation

### .deb-Paket (empfohlen für Debian/Mint/Ubuntu):

```bash
sudo apt install ./emoji-picker_1.0.0_amd64.deb
```

### Manuell kompilieren:

```bash
git clone https://github.com/kai-thanner/emoji-picker.git
cd emoji-picker
cargo build --release
```

Die fertige Binärdatei liegt dann in `target/release/emoji-picker`

---

## ⚙️ Konfiguration & Einstellungen

Beim ersten Start wird unter `~/.config/emoji-picker/settings.ini` automatisch eine Datei erzeugt:

```ini
[Allgemein]
setup_erledigt = true          Legt fest ob Setup beim ersten Start ausgeführt wurde
fenster_schliessen = true      Ob das Fenster automatisch geschloßen wird
emoji_size = 20                Größeneinstellung der Emojis
```

Die Werte lassen sich dort jederzeit anpassen.

---

## ⌨ Tastenkombination einrichten

Falls beim ersten Start keine Tastenkombination gesetzt wurde:
getestet unter: Linux Mint 22.1 Cinnamon

```bash
emoji-picker --setup-shortcut
```

Diese legt unter Cinnamon die Kombination Super+. für den Emoji-Picker an.

---

## 🌍 Speicherorte

| Datei/Ordner                                    | Beschreibung                    |
| ----------------------------------------------- | ------------------------------- |
| `/usr/bin/emoji-picker`                         | Ausführbare Datei               |
| `/usr/share/applications/emoji-picker.desktop`  | Eintrag im Startmenü            |
| `/usr/share/icons/hicolor/.../emoji-picker.png` | Icon                            |
| `/etc/skel/.config/emoji-picker/`               | .list-Dateien als Vorlage       |
| `~/.config/emoji-picker/`                       | Nutzerdaten (Symbole, Settings) |

---

## ✏️ Lizenz

Dieses Projekt steht unter der MIT-Lizenz. Siehe [LICENSE](LICENSE).

## 👨‍💻 Entwickler

> Erstellt von Kai Thanner
