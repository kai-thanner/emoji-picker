use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, Button, CheckButton, ComboBoxText,
    Dialog, Grid, Label, Orientation, ResponseType, SpinButton,
};
use std::{
	cell::{Cell, RefCell},
	collections::HashMap,
	fs,
	path::PathBuf,
	rc::Rc,
};

use crate::shortcut;
use crate::emoji_tabs::*; 
use crate::i18n::Sprache;

#[derive(Clone, Debug)]
pub struct Einstellungen {
    pub setup_erledigt: Cell<bool>,
    pub fenster_schliessen: Cell<bool>,
    pub fenster_offen_bei_drag: Cell<bool>,
    pub emoji_size: Cell<i32>,
    pub sprache: RefCell<String>,
}

// ╔══════════════════════════════════════════════════════════════╗
// ║                  Ablauf: settings.ini Usage                  ║
// ╚══════════════════════════════════════════════════════════════╝
//
//     [ Programmstart ]
//             │
//             ▼
// ╔═══════════════════════╗
// ║    lade_settings()    ║  ←  settings.ini: emoji_size, setup_erledigt etc.
// ╚═══════════════════════╝
//             │
//             │
//             ├──────────────────────────────► Argument: --setup ────────────────────────────┐
//             │               											 		              │												
//             │																              ▼
//             ▼															      ╔═══════════════════════╗
//  [ Hauptfenster läuft ]─────────────────[ setup_erledigt == false ]  	      ║	 setup_shortcut()     ║
//             │                                        │					      ╚═══════════════════════╝
//             │                                        ▼							          │
//             │                             ╔═══════════════════════╗		    	          ▼
//             ▼                             ║  Zeige Setup-Dialog   ║	             [  Programmende  ]
//     [ Zahnrad-Klick ]                     ╚═══════════════════════╝
//             │                                        │
//             │                                        │ 
//             ▼                                        ▼
// ╔═══════════════════════╗                 ╔═══════════════════════╗
// ║    lade_settings()    ║                 ║    lade_settings()    ║
// ╚═══════════════════════╝                 ╚═══════════════════════╝
//             │                                        │
//             ▼                                        ▼
// ╔═══════════════════════╗                 ╔═══════════════════════╗
// ║ User ändert Optionen  ║                 ║  Shortcut einrichten  ║
// ╚═══════════════════════╝                 ╚═══════════════════════╝
//             │                                        │
//             ▼                                        ▼
// ╔═══════════════════════╗                 ╔═══════════════════════╗
// ║  speichere_settings() ║                 ║  speichere_settings() ║  ← settings.ini: setup_erledigt = true
// ╚═══════════════════════╝                 ╚═══════════════════════╝
//             │                                        │
//             ▼                                        ▼
//  [ Einstellungen aktiv ]                  [ Kein Setup-Dialog mehr ]



pub fn zeige_einstellungsfenster(
	parent: Rc<ApplicationWindow>,
	einstellungen: Rc<Einstellungen>,
	emojies_daten: Rc<RefCell<HashMap<String, (Vec<Symbol>, Rc<Grid>)>>>,
	sprachpaket: Rc<Sprache>,
	debug: bool,
) {
	let dialog = Dialog::builder()
		.transient_for(parent.as_ref())
		.title(&sprachpaket.settings_window)
		.default_width(300)
		.default_height(200)
		.modal(true)
		.build();

	let content_area = dialog.content_area();

    let vbox = GtkBox::new(Orientation::Vertical, 8);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);
    content_area.append(&vbox);

	// // 🔁 Shortcut erneut aktivieren
	let shortcut_button = Button::with_label(&sprachpaket.set_key);
	shortcut_button.set_tooltip_text(Some(&sprachpaket.set_key_tooltip));
	{
		let parent_shortcut = Rc::clone(&parent);
		let einstellungen_shortcut = Rc::clone(&einstellungen);
		let sprachpaket_shortcut = Rc::clone(&sprachpaket);

		shortcut_button.connect_clicked(move |_|{
			shortcut::zeige_setup_dialog(
				parent_shortcut.as_ref(),
				&einstellungen_shortcut,
				Rc::clone(&sprachpaket_shortcut),
				debug);
		});
	}
	vbox.append(&shortcut_button);

	// 🪟 Fenster nach Auswahl schließen
	let fenster_schliessen_checkbox = CheckButton::with_label(&sprachpaket.close_window_get);
	fenster_schliessen_checkbox.set_active(einstellungen.fenster_schliessen.get());
	vbox.append(&fenster_schliessen_checkbox);

	// Drag-&-Drop Verhalten
	let drag_checkbox = CheckButton::with_label(&sprachpaket.close_window_dnd);
	drag_checkbox.set_active(einstellungen.fenster_offen_bei_drag.get());
	vbox.append(&drag_checkbox);

	// 🔠 Emoji-Größe
	let size_label = Label::new(Some(&format!("{} (px):", &sprachpaket.emoji_size)));
	let emoji_size_spinner = SpinButton::with_range(10.0, 100.0, 2.0);
	emoji_size_spinner.set_value(einstellungen.emoji_size.get() as f64);
	vbox.append(&size_label);
	vbox.append(&emoji_size_spinner);

	// 📖 Verlauf löschen
	let verlauf_button = Button::with_label(&sprachpaket.hist_reset);
	verlauf_button.add_css_class("verlauf-reset");

	{
		let emojies_daten = Rc::clone(&emojies_daten);
		verlauf_button.connect_clicked(move |_| {
			crate::emoji_tabs::leere_history_tab(&[("history.list", "🕓")], &emojies_daten);
		});
	}

	vbox.append(&verlauf_button);

	// 🌍 Sprache auswählen
	let sprachwahl_box = GtkBox::new(Orientation::Horizontal, 8);

	// Label-Text
	let label_sprachwahl = Label::new(Some(&sprachpaket.label_language));
	label_sprachwahl.set_halign(gtk::Align::Start); // Links ausrichten
	label_sprachwahl.set_valign(gtk::Align::Center); // Zentriert zur Combobox

	// ComboBox
	let sprachwahl = ComboBoxText::new();
	let verfuegbare_sprachen = Sprache::finde_verfuegbare_sprachen(debug);
	let aktuelle_sprache = einstellungen.sprache.borrow().clone();

	for eintrag in &verfuegbare_sprachen {
		let label = format!("{} {}", eintrag.flagge, eintrag.name);
		sprachwahl.append(Some(&eintrag.code), &label);
	}
	sprachwahl.set_active_id(Some(&aktuelle_sprache));

	// Box befüllen
	sprachwahl_box.append(&label_sprachwahl);
	sprachwahl_box.append(&sprachwahl);

	// In die Einstellungs-Box einfügen
	vbox.append(&sprachwahl_box);

	// 🆗 Dialog OK / Schließen
	dialog.add_button(&sprachpaket.button_cancel, ResponseType::Cancel);
	dialog.add_button(&sprachpaket.button_ok, ResponseType::Ok);
	dialog.show();
	
	let einstellungen_neu = Rc::clone(&einstellungen);
	dialog.connect_response(move |dialog, response| {
	    if response == ResponseType::Ok {
	        einstellungen_neu.fenster_schliessen.set(fenster_schliessen_checkbox.is_active());
	        einstellungen_neu.fenster_offen_bei_drag.set(drag_checkbox.is_active());
	        einstellungen_neu.emoji_size.set(emoji_size_spinner.value() as i32);
	        crate::emoji_tabs::aktualisiere_emoji_style(einstellungen_neu.emoji_size.get());
	        crate::emoji_tabs::aktualisiere_tablabel_style(einstellungen.emoji_size.get());

	        // ausgewählte Sprache festlegen
			let sprach_id = sprachwahl.active_id().unwrap_or_else(|| "system".into());
			einstellungen.sprache.replace(sprach_id.to_string());

	        // Fenstergrösse anpassen
	        let neue_groesse = einstellungen.emoji_size.get();
			let fenster_breite = 13 * (neue_groesse + 4); // 13 Emojis pro Zeile, etwas Puffer
			let fenster_hoehe = 10 * (neue_groesse + 20); // in etwa 10 Reihen

			let parent_resize_window = Rc::clone(&parent);
			glib::idle_add_local_once(move || {
				parent_resize_window.set_resizable(true); // nur zur Sicherheit
			    parent_resize_window.set_default_size(fenster_breite, fenster_hoehe);
			    parent_resize_window.queue_resize(); // zwingt GTK zur Neuberechnung
			});
	    }
		dialog.close();
	});

	dialog.show();

	// Warten auf Benutzerantwort durch RunLoop (Ersatz für 'dialog.run()' in GTK4)
	while dialog.is_visible() {
		while gtk::glib::MainContext::default().iteration(false) {}
	}
}

pub fn lade_settings() -> Einstellungen {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker/settings.ini");

    if !pfad.exists() {
        let _ = fs::create_dir_all(pfad.parent().unwrap());
        let _ = fs::write(&pfad, "[Allgemein]\nsetup_erledigt = false\nfenster_schliessen = true\nfenster_offen_bei_drag = true\nemoji_size = 20\nsprache = system\n");
    }
    
    let content = fs::read_to_string(&pfad).unwrap_or_default();

    let mut setup_erledigt = false;
    let mut fenster_schliessen = true;
    let mut fenster_offen_bei_drag = true;
    let mut emoji_size = 20;
    let mut sprache = "system".to_string();
    
    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("setup_erledigt") {
            if let Some(value) = line.split('=').nth(1) {
                setup_erledigt = value.trim() == "true";
            }
        }
        if line.starts_with("fenster_schliessen") {
            if let Some(value) = line.split('=').nth(1) {
                fenster_schliessen = value.trim() == "true";
            }
        }
        if line.starts_with("fenster_offen_bei_drag") {
            if let Some(value) = line.split('=').nth(1) {
                fenster_offen_bei_drag = value.trim() == "true";
            }
        }
        if line.starts_with("emoji_size") {
            if let Some(value) = line.split('=').nth(1) {
                emoji_size = value.trim().parse().unwrap_or(30);
            }
        }
        if line.starts_with("sprache") {
            if let Some(value) = line.split('=').nth(1) {
                sprache = value.trim().to_string();
            }
        }
    }

    Einstellungen {
        setup_erledigt: Cell::new(setup_erledigt),
        fenster_schliessen: Cell::new(fenster_schliessen),
        fenster_offen_bei_drag: Cell::new(fenster_offen_bei_drag),
        emoji_size: Cell::new(emoji_size),
        sprache: RefCell::new(sprache),
    }
}

pub fn speichere_settings(einstellungen: &Einstellungen) {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker/settings.ini");

    let inhalt = format!(
        "[Allgemein]\nsetup_erledigt = {}\nfenster_schliessen = {}\nfenster_offen_bei_drag = {}\nemoji_size = {}\nsprache = {}\n",
        einstellungen.setup_erledigt.get(),
        einstellungen.fenster_schliessen.get(),
        einstellungen.fenster_offen_bei_drag.get(),
        einstellungen.emoji_size.get(),
        einstellungen.sprache.borrow(),
    );

    let _ = fs::write(&pfad, inhalt);
}