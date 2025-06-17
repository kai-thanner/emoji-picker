use gtk::prelude::*;
use gtk::{ ApplicationWindow, MessageDialog, ButtonsType, MessageType};
use std::rc::Rc;
use std::process::Command;

use crate::settings::Einstellungen;

#[derive(Debug)]
enum Desktop {
	Cinnamon,
	Xfce,
	Mate,
	Kde,
	Gnome,
	Unbekannt,
}

#[derive(Debug)]
pub struct ShortcutErgebnis {
	pub desktop: String,
	pub erfolg: bool,
	pub meldung: String,
}

pub fn zeige_setup_dialog(fenster: &ApplicationWindow, einstellungen: &Rc<Einstellungen>) {
	let shortcut_info = setup_shortcut();

	crate::settings::speichere_settings(einstellungen);

	let dialog = MessageDialog::builder()
		.transient_for(fenster)
		.modal(true)
		.message_type(MessageType::Info)
		.buttons(ButtonsType::Ok)
		.text(&format!("{} - {}", shortcut_info.desktop, if shortcut_info.erfolg {
	            "Einrichtung erfolgreich 🎉"
	        } else {
	            "Einrichtung fehlgeschlagen ❌"
	        }
	    ))
		.secondary_text(&shortcut_info.meldung)
		.build();

	dialog.connect_response(|dialog, _| {
		dialog.close();
	});

	dialog.show();
}

pub fn setup_shortcut() -> ShortcutErgebnis {
	match detect_desktop() {
		Desktop::Cinnamon	=> setup_cinnamon(),
		Desktop::Xfce		=> setup_xfce(),
		Desktop::Mate 		=> setup_mate(),
		Desktop::Kde		=> setup_kde(),
		Desktop::Gnome		=> setup_gnome(),
		Desktop::Unbekannt	=> ShortcutErgebnis {
			desktop: "Unbekannt".into(),
			erfolg: false,
			meldung: "🚫 Desktopumgebung nicht erkannt. Bitte manuell konfigurieren.".into(),
		},
	}
}

fn detect_desktop() -> Desktop {
	use std::env;

	if let Ok(session) = env::var("XDG_CURRENT_DESKTOP") {
		let session = session.to_lowercase();
		if session.contains("cinnamon") {
			Desktop::Cinnamon
		} else if session.contains("xfce") {
			Desktop::Xfce
		} else if session.contains("mate") {
			Desktop::Mate
		} else if session.contains("kde") {
			Desktop::Kde
		} else if session.contains("gnome") {
			Desktop::Gnome
		} else {
			Desktop::Unbekannt
		}
	} else {
		Desktop::Unbekannt
	}
}

fn apply_gsettings(command: &[(&str, &[&str])]) -> bool {
	let mut alles_ok = true;

    for (cmd, args) in command {
        let status = Command::new(cmd).args(*args).status();
        match status {
            Ok(s) if s.success() => continue,
            Ok(s) => {
            	eprintln!("‼️ {} exit code {}", cmd, s.code().unwrap_or(-1));
            	alles_ok = false;
            },
            Err(e) => {
            	eprintln!("❌ Fehler beim Aufruf von {}: {}", cmd, e);
            	alles_ok = false;
            }
        }
    }
    alles_ok
}

fn setup_cinnamon() -> ShortcutErgebnis {
    println!("🛠 Versuche, Tastenkombi <Super>+. zu setzen...");

    let cmds = vec![
        ("gsettings", &["set", "org.cinnamon.desktop.keybindings", "custom-list", "['custom0']"][..]),
        ("gsettings", &["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/custom0/", "name", "Emoji Picker"][..]),
        ("gsettings", &["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/custom0/", "command", "emoji-picker"][..]),
        ("gsettings", &["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/custom0/", "binding", "['<Super>period']"][..]),
    ];

    let erfolg = apply_gsettings(&cmds);
    let meldung = if erfolg {
    	"✅ Tastenkombination erfolgreich eingerichtet.\n\nDu kannst den Emoji Picker nun mit Super+. starten.\n\n🔁 Hinweis: Falls es nicht sofort klappt, drücke Alt+F2, tippe `r` und bestätige mit Enter.".into()
    } else {
    	"‼️ Fehler bei der Einrichtung.\n\nBitte öffne die Tastenkombinationen und füge den Emoji Picker manuell hinzu.".into()
    };

    ShortcutErgebnis {
        desktop: "Cinnamon".into(),
        erfolg,
        meldung,
    }
}

fn setup_xfce() -> ShortcutErgebnis {
	println!("🛠 XFCE: Versuche, Tastenkombi <Super>+. zu setzen...");

	let status = Command::new("xfconf-query")
		.args(&[
			"--channel", "xfce4-keyboard-shortcuts",
			"--property", "/commands/custom/<Super>period",
			"--create",
			"--type", "string",
			"--set", "emoji-picker",
		])
		.status();

	match status {
		Ok(s) if s.success() => ShortcutErgebnis {
			desktop: "XFCE".into(),
			erfolg: true,
			meldung: "✅ Tastenkombination erfolgreich eingerichtet.\n\nDu kannst den Emoji Picker nun mit Super+. starten.".into(),
		},
		Ok(s) => ShortcutErgebnis {
			desktop: "XFCE".into(),
			erfolg: false,
			meldung: format!("‼️ Fehler – exit code {}", s.code().unwrap_or(-1)),
		},
		Err(e) => ShortcutErgebnis {
			desktop: "XFCE".into(),
			erfolg: false,
			meldung: format!("❌ Fehler beim Aufruf von xfconf-query: {}", e),
		},
	}
}

fn setup_mate() -> ShortcutErgebnis {
    ShortcutErgebnis {
        desktop: "MATE".into(),
        erfolg: true,
        meldung: "🛠 automatische Einrichtung nicht verfügbar.\n\n➡️ Bitte füge manuell eine Tastenkombination hinzu:\n    • Befehl: emoji-picker\n    • Tastenkombi: <Super>+.".into(),
    }
}

fn setup_kde() -> ShortcutErgebnis {
    ShortcutErgebnis {
        desktop: "KDE".into(),
        erfolg: true,
        meldung: "🛠 Automatische Einrichtung nicht möglich.\n\nBitte öffne Systemeinstellungen → Tastenkombinationen → Benutzerdefiniert und füge den Befehl `emoji-picker` mit <Super>+. hinzu.".into(),
    }
}

fn setup_gnome() -> ShortcutErgebnis {
    println!("🛠 Versuche, Tastenkombi <Super>+. zu setzen...");

    let cmds = vec![
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybindings", "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/']"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "name", "Emoji Picker"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "command", "emoji-picker"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "binding", "['<Super>period']"][..]),
    ];

	let erfolg = apply_gsettings(&cmds);
	let meldung = if erfolg {
	    "✅ Tastenkombination erfolgreich eingerichtet.\n\nDu kannst den Emoji Picker nun mit Super+. starten.".into()
	} else {
	    "‼️ Fehler bei der Einrichtung.\n\nBitte öffne die Tastenkombinationen und füge den Emoji Picker manuell hinzu.".into()
	};

    ShortcutErgebnis {
        desktop: "GNOME".into(),
        erfolg,
        meldung,
    }
}
