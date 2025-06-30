use gtk::prelude::*;
use gtk::{ ApplicationWindow, MessageDialog, ButtonsType, MessageType};
use std::rc::Rc;
use std::process::Command;

use crate::settings::Einstellungen; 
use crate::settings;
use crate::i18n::Sprache;

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

pub fn zeige_setup_dialog(
	fenster: &ApplicationWindow,
	einstellungen: &Rc<Einstellungen>,
	sprachpaket: Rc<Sprache>,
	debug: bool
) {
	let shortcut_info = setup_shortcut(Rc::clone(&sprachpaket), debug);

	let dialog = MessageDialog::builder()
		.transient_for(fenster)
		.modal(true)
		.message_type(MessageType::Info)
		.buttons(ButtonsType::Ok)
		.text(&format!("{} - {}", shortcut_info.desktop, if shortcut_info.erfolg {
	            sprachpaket.setup_done.clone() // "Einrichtung erfolgreich 🎉"
	        } else {
	            sprachpaket.setup_fail.clone() // "Einrichtung fehlgeschlagen ❌"
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

    if debug {
	    println!("💾 {}", sprachpaket.debug_shortcut_set_info_window);
	}
}

pub fn setup_shortcut(sprachpaket: Rc<Sprache>, debug: bool) -> ShortcutErgebnis {
	let ergebnis = match detect_desktop() {
		Desktop::Cinnamon	=> setup_cinnamon(Rc::clone(&sprachpaket), debug),
		Desktop::Xfce		=> setup_xfce(Rc::clone(&sprachpaket)),
		Desktop::Mate 		=> setup_mate(Rc::clone(&sprachpaket)),
		Desktop::Kde		=> setup_kde(Rc::clone(&sprachpaket)),
		Desktop::Gnome		=> setup_gnome(Rc::clone(&sprachpaket)),
		Desktop::Unbekannt	=> ShortcutErgebnis {
			desktop: "Unbekannt".into(),
			erfolg: false,
			meldung: sprachpaket.set_desk_unknown.clone().into(),
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

fn apply_gsettings(command: &[(&str, &[&str])], sprachpaket: Rc<Sprache>) -> bool {
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
            	eprintln!("❌ {} {}: {}", sprachpaket.debug_shortcut_apply_gsettings_error, cmd, e);
            	alles_ok = false;
            }
        }
    }
    alles_ok
}

fn setup_cinnamon(sprachpaket: Rc<Sprache>, debug: bool) -> ShortcutErgebnis {
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
    				if debug {
    					println!("{}: {}", sprachpaket.debug_shortcut_cinna_already_done, eintrag);
    				}
    				return ShortcutErgebnis {
    					desktop: "Cinnamon".into(),
    					erfolg: true,
    					meldung: sprachpaket.setup_exists.clone().into(),
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

    let erfolg = apply_gsettings(&cmds, Rc::clone(&sprachpaket));

    let meldung = if erfolg {
    	sprachpaket.setup_done_cinna.clone().into()
    } else {
    	sprachpaket.setup_fail_text.clone().into()
    };

    ShortcutErgebnis {
        desktop: "Cinnamon".into(),
        erfolg,
        meldung,
    }
}

fn setup_xfce(sprachpaket: Rc<Sprache>) -> ShortcutErgebnis {
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
			meldung: sprachpaket.setup_done_xfce_gno.clone().into(),
		},
		Ok(s) => ShortcutErgebnis {
			desktop: "XFCE".into(),
			erfolg: false,
			meldung: format!("{} – exit code: {}", sprachpaket.setup_fail_xfce_1, s.code().unwrap_or(-1)),
		},
		Err(e) => ShortcutErgebnis {
			desktop: "XFCE".into(),
			erfolg: false,
			meldung: format!("{} xfconf-query: {}", sprachpaket.setup_fail_xfce_2, e),
		},
	}
}

fn setup_mate(sprachpaket: Rc<Sprache>) -> ShortcutErgebnis {
    ShortcutErgebnis {
        desktop: "MATE".into(),
        erfolg: false,
        meldung: sprachpaket.setup_not_available.clone().into(),
    }
}

fn setup_kde(sprachpaket: Rc<Sprache>) -> ShortcutErgebnis {
    ShortcutErgebnis {
        desktop: "KDE".into(),
        erfolg: false,
        meldung: sprachpaket.setup_not_available.clone().into(),
    }
}

fn setup_gnome(sprachpaket: Rc<Sprache>) -> ShortcutErgebnis {
    println!("🛠 Versuche, Tastenkombi <Super>+. zu setzen...");

    let cmds = vec![
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/']"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "name", "Emoji Picker"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "command", "emoji-picker"][..]),
        ("gsettings", &["set", "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/", "binding", "<Super>period"][..]),
    ];

	let erfolg = apply_gsettings(&cmds, Rc::clone(&sprachpaket));
	let meldung = if erfolg {
	    sprachpaket.setup_done_xfce_gno.clone().into()
	} else {
	    sprachpaket.setup_fail_text.clone().into()
	};

    ShortcutErgebnis {
        desktop: "GNOME".into(),
        erfolg,
        meldung,
    }
}
