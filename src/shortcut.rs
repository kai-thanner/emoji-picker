use gtk::prelude::*;
use gtk::{ ApplicationWindow, MessageDialog, ButtonsType, MessageType};
use std::rc::Rc;
use std::process::Command;

use crate::settings::Einstellungen; 
use crate::settings;

#[derive(Debug, PartialEq)]
pub enum Desktop {
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

pub fn zeige_setup_dialog(fenster: &ApplicationWindow, einstellungen: &Rc<Einstellungen>, debug: u8) {
	let shortcut_info = setup_shortcut();

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

	// 📁 Config-Datei aktualisieren		
    // ⏩ Auch wenn kein Shortcut möglich ist, nicht erneut fragen
    einstellungen.setup_erledigt.set(true);
    settings::speichere_settings(&einstellungen);

    if debug == 1 {
	    println!("💾 Setup wurde angezeigt. setup_erledigt auf true gesetzt");
	}
}

pub fn setup_shortcut() -> ShortcutErgebnis {
	let ergebnis = match detect_desktop() {
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
	};
	ergebnis
}

pub fn detect_desktop() -> Desktop {
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

    // Bestehende Liste "Eigene Tastenkombinationen" abrufen
    let output = std::process::Command::new("gsettings")
    	.args(&["get", "org.cinnamon.desktop.keybindings", "custom-list"])
    	.output();

    let mut list = vec![];

    if let Ok(output) = output {
    	if output.status.success() {
    		let raw = String::from_utf8_lossy(&output.stdout);
    		list = raw
    			.trim_matches(['[', ']', '\n', ' ', '\''].as_ref())
    			.split(',')
    			.map(|s| s.trim_matches(&['\'', ' '][..]).to_string())
    			.filter(|s| !s.is_empty())
    			.collect();
    	}
    }

    // Prüfen, ob emoji-picker bereits eingetragen ist
    for eintrag in &list {
    	let full_path = format!("/org/cinnamon/desktop/keybindings/custom-keybindings/{}/", eintrag);
    	let output = std::process::Command::new("gsettings")
    		.args(&[
    			"get",
    			&format!("org.cinnamon.desktop.keybindings.custom-keybinding:{}", full_path),
    			"command",
    		])
    		.output();

    	if let Ok(output) = output {
    		if output.status.success() {
    			let raw = String::from_utf8_lossy(&output.stdout);
    			if raw.contains("emoji-picker") {
    				println!("Emoji Picker bereits in '{}' eingetragen. Kein neuer Eintrag nötig", eintrag);
    				return ShortcutErgebnis {
    					desktop: "Cinnamon".into(),
    					erfolg: true,
    					meldung: "✅ Tastenkombination war bereits vorhanden.".into(),
    				};
    			}
    		}
    	}

    }


    // Eintrag suchen (custom0, custom1, ...)
    let mut custom_key = String::new();
    for i in 0..50 {
    	let key = format!("custom{}", i);
    	if !list.contains(&key) {
    		custom_key = key;
    		list.push(custom_key.clone());
    		break;
    	}
    }

    // Pfad zum Ziel
    let full_path = format!("org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/{}/", custom_key);

    // Keybinding setzen
    let list_string = format!(
    	"[{}]",
    	list.iter()
    		.map(|s| format!("'{}'", s))
    		.collect::<Vec<_>>()
    		.join(", ")
    );
    let gsettings_custom_list	= ["set", "org.cinnamon.desktop.keybindings", "custom-list", &list_string];
    let gsettings_name 			= ["set", &full_path, "name", "Emoji Picker"];
    let gsettings_command		= ["set", &full_path, "command", "emoji-picker"];
    let gsettings_binding		= ["set", &full_path, "binding", "['<Super>period']"];

    let cmds = vec![
        ("gsettings", &gsettings_custom_list[..]),
    	("gsettings", &gsettings_name[..]),
 	   	("gsettings", &gsettings_command[..]),
    	("gsettings", &gsettings_binding[..]),
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
        erfolg: false,
        meldung: "🛠 automatische Einrichtung nicht verfügbar.\n\n➡️ Bitte füge manuell eine Tastenkombination hinzu:\n    • Befehl: emoji-picker\n    • Tastenkombi: <Super>+.".into(),
    }
}

fn setup_kde() -> ShortcutErgebnis {
    ShortcutErgebnis {
        desktop: "KDE".into(),
        erfolg: false,
        meldung: "🛠 Automatische Einrichtung nicht möglich.\n\nBitte öffne Systemeinstellungen → Tastenkombinationen → Benutzerdefiniert und füge den Befehl `emoji-picker` mit <Super>+. hinzu.".into(),
    }
}

fn setup_gnome() -> ShortcutErgebnis {
    println!("🛠 Versuche, Tastenkombi <Super>+. zu setzen...");

    let cmds = vec![
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/']"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "name", "Emoji Picker"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "command", "emoji-picker"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "binding", "<Super>period"][..]),
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
