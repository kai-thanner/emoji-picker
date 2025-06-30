mod settings;
mod shortcut;
mod emoji_tabs;
mod suchlogik;
mod i18n;
mod gtk_theme;

use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry,
    EventControllerKey, Grid, Notebook, Orientation, PolicyType, ScrolledWindow, Stack,
};
use gtk::gdk::{self, Clipboard};
use glib::clone;
use std::{
    cell::RefCell,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    rc::Rc,
    time::{Instant, SystemTime},
};

use crate::i18n::Sprache;

fn main() {
    // Zeitmessung für Programmstart
    let args: Vec<String> = std::env::args().collect();
    let debug: bool = if args.contains(&"--debug".to_string()) { true } else { false };
    let timer = Instant::now();

    if debug {
        println!("⏳ Debug output enabled, startup time {:?}", timer.elapsed());
    }

    // Argument --lang abfangen
    let mut sprachcode: Option<String> = None;
    for i in 0..args.len() {
        if args[i] == "--lang" && i + 1 < args.len() {
            sprachcode = Some(args[i + 1].clone());
        }
    }

    // Sprachpaket laden
    let sprachpaket = Rc::new(Sprache::sprache_erkennen(&sprachcode, debug));

    if debug {
        println!("⏳ {} {:?}", sprachpaket.debug_main_time_loading_language, timer.elapsed());
    }

    // Argumente abfangen
    if args.contains(&"--setup".to_string()) || args.contains(&"-S".to_string()) {
        shortcut::setup_shortcut(Rc::clone(&sprachpaket), debug);
        return;
    }

    if args.contains(&"--version".to_string()) || args.contains(&"-V".to_string()) {
        println!("Emoji Picker 📦 Version: {}", env!("CARGO_PKG_VERSION"));
        println!("Copyright © 2025");
        println!("Lizenz: MIT"); 
        println!("Geschrieben von: {}", env!("CARGO_PKG_AUTHORS")); 
        std::process::exit(0);
    }

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("\nUsage: emoji-picker [OPTIONS]");        
        println!("\nOptions:\n");        
        println!("-h,  --help              Print help");        
        println!("-V,  --version           Print version info and exit");
        println!("-S   --setup             Try to set keybinding");
        println!("     --debug             Enable debug output");
        std::process::exit(0);
    }

    crate::gtk_theme::pruefe_und_setze_gtk_theme_fuer_kde(Rc::clone(&sprachpaket), debug);

    let app: Application = Application::builder()
        .application_id("com.kai_thanner.emoji-picker")
        .build();

    app.connect_activate(move |app| {
        // Fenster erstellen
        let window = Rc::new(ApplicationWindow::builder()
            .application(app)
            .title(&sprachpaket.title)//.title("Emoji-Auswahl")
            .default_width(400)
            .default_height(400)
            .build()
        );

        // Hauptlayout
        let vbox = GtkBox::new(Orientation::Vertical, 5);
        window.set_child(Some(&vbox));
     
        // CSS für UI laden
        crate::emoji_tabs::lade_ui_css(Rc::clone(&sprachpaket), debug);

        if debug {
            println!("⏳ {}: {:?}", sprachpaket.debug_main_time_css_loading, timer.elapsed());
        }

        // 🔍 Suchfeld + ⚙️ Zahnrad-Button gemeinsam in Box
        let suchbox = GtkBox::new(Orientation::Horizontal, 6);

        // Suchfeld
        let suchfeld = Entry::new();
        suchfeld.add_css_class("search-entry");
        suchfeld.set_placeholder_text(Some(&sprachpaket.search_placeholder));
        suchfeld.set_hexpand(true); // expandiert innerhalb der Zeile
        suchbox.append(&suchfeld);

        // Zahnrad-Button (Oder was der Desktop vorgibt)
        let settings_button = Button::from_icon_name("emblem-system-symbolic");
        settings_button.set_tooltip_text(Some(&sprachpaket.settings_window));
        settings_button.set_margin_end(6);
        settings_button.set_margin_top(6);
        settings_button.set_size_request(28, 28);
        settings_button.add_css_class("flat");  // GTK4-Klasse für stilisierten Button
        suchbox.append(&settings_button);

        // Box in Hauptfenster einfügen
        vbox.append(&suchbox);

        // settings.ini auslesen / erstellen
        let einstellungen = Rc::new(settings::lade_settings());
        emoji_tabs::aktualisiere_emoji_style(einstellungen.emoji_size.get());

        // Konfiguration zwischenspeichern
        let zeige_infofenster = !einstellungen.setup_erledigt.get();

        if debug {
            println!("📤 {}: {:?}", sprachpaket.debug_main_settings_infofenster, einstellungen);
        }

        // Notebook für Kategorien
        let notebook = Rc::new(Notebook::new());
        notebook.set_tab_pos(gtk::PositionType::Left);
        notebook.set_vexpand(true);
        notebook.set_hexpand(true);
        
        let emoji_size = einstellungen.emoji_size.get();

        // Such-Grid und Scrollbereich
        let such_grid = Rc::new(Grid::new());
        such_grid.set_row_spacing(5);
        such_grid.set_column_spacing(5);
        such_grid.set_margin_top(10);
        such_grid.set_margin_bottom(10);
        such_grid.set_margin_start(12);
        such_grid.set_margin_end(12);

        let scroll_suche = ScrolledWindow::new();
        scroll_suche.set_policy(PolicyType::Never, PolicyType::Automatic);
        scroll_suche.set_child(Some(&*such_grid));

        // Stack als Suche über alle Kategorien
        let stack = Stack::new();
        stack.set_vexpand(true);
        stack.set_hexpand(true);
        stack.add_named(&*notebook, Some("notebook"));
        stack.add_named(&scroll_suche, Some("suche"));
        stack.set_visible_child_name("notebook");
        vbox.append(&stack);

        // Kategorien
        let kategorien = vec![
            ("history.list",    "🕓"),
            ("smileys.list",    "😄"),
            ("peoples.list",    "👨"),
            ("animals.list",    "🐰"),
            ("gestures.list",   "👋"),
            ("clothing.list",   "👕"),
            ("activity.list",   "🏀"),
            ("travel.list",     "✈️"),
            ("nature.list",     "🌲"),
            ("food.list",       "🍌"),
            ("objects.list",    "📎"),
            ("symbole.list",    "✅"),
            ("flags.list",      "🇩🇪"),
        ];

        // Clipboard vorbereiten
        let display = gtk::gdk::Display::default().unwrap();
        let clipboard = Rc::new(display.clipboard());

        for (datei, _) in &kategorien {
            // .list Dateien anlegen falls nicht vorhanden
            kopiere_von_etc_falls_fehlend(datei, Rc::clone(&sprachpaket), &debug);
        }

        if debug {
            println!("⏳ /etc/emoji-picker {} {:?}", sprachpaket.debug_main_time_copy_from_etc, timer.elapsed());
        }

        // Symbole parallel Laden
        let emojies_daten = Rc::new(RefCell::new(emoji_tabs::erstelle_tabs(&notebook, &kategorien[..], emoji_size)));

        // Einstellungsfenster öffnen nachdem alles geladen wurde
        {
            let einstellungen_settings_button = Rc::clone(&einstellungen);
            let window_settings_button = Rc::clone(&window);
            let emojies_daten_settings_button = Rc::clone(&emojies_daten);
            let sprachpaket_settings_button = Rc::clone(&sprachpaket);
            settings_button.connect_clicked(move |_| {
                settings::zeige_einstellungsfenster(
                    Rc::clone(&window_settings_button),
                    Rc::clone(&einstellungen_settings_button),
                    Rc::clone(&emojies_daten_settings_button),
                    Rc::clone(&sprachpaket_settings_button),
                    debug,
                 );
                settings::speichere_settings(&einstellungen_settings_button);
            });
        }

        if debug {
            println!("⏳ {}: {:?}", sprachpaket.debug_main_time_emojis_load, timer.elapsed());
        }

        // Suchindex erstellen (flache Liste aller Symbole)
        let such_index = Rc::new(
            emojies_daten
                .borrow()
                .iter()
                .filter(|(label, _)| *label != "🕓")        // History nicht durchsuchen   
                .flat_map(|(_, symbole)| symbole.0.clone()) // .0 ist Vec<Symbol>, .1 wäre Rc<Grid>
                .collect::<Vec<_>>()
        );

        if debug {
            println!("⏳ {}: {:?}", sprachpaket.debug_main_time_searchindex, timer.elapsed());
        }

        // Symbole in Kategorien einfügen, incl. Buttons, ToolTip und Drag&Drop
        emoji_tabs::fuege_emojis_ein(
            &emojies_daten.borrow(),
            Rc::clone(&clipboard),
            Rc::clone(&window),
            Rc::clone(&einstellungen),
        );

        if debug {
            println!("⏳ {} {:?}", sprachpaket.debug_main_time_categorize_emoji, timer.elapsed());
        }

        //Suchlogik
        suchlogik::verbinde_suchfeld(
            &suchfeld,
            Rc::clone(&such_grid),
            stack.clone(),
            Rc::clone(&such_index),
            Rc::clone(&clipboard),
            Rc::clone(&window),
            Rc::clone(&einstellungen),
        );

        if debug {
            println!("⏳ {} {:?}", sprachpaket.debug_main_time_to_searchfield, timer.elapsed());
        }

        // Variabeln für Suchfunktion und verhalten der Entertaste
        let emojies_daten_suchfeld = Rc::clone(&emojies_daten);
        let einstellungen_suchfeld = Rc::clone(&einstellungen);

        // Einstellung der Suchfunktion und verhalten der Entertaste
        #[allow(deprecated)]                        // glib wird gerade umgebaut, daher gibt es Warnungen für clone!. Bei nächstem Update auf Funktion prüfen!
        suchfeld.connect_activate(clone!(
            @weak stack,
            @weak such_grid, 
            @weak clipboard, 
            @weak window,
            @strong einstellungen_suchfeld, 
            @strong emojies_daten_suchfeld => move |_| {

            let stack_visible = stack.visible_child_name();
            let fenster_schliessen = einstellungen_suchfeld.fenster_schliessen.get();

            if stack_visible == Some("suche".into()) {
                // Aktiv: Suchansicht → erstes Ergebnis aus Such-Grid nehmen
                let mut child = such_grid.first_child();
                while let Some(widget) = child {
                    child = widget.next_sibling();
                    if let Some(button) = widget.downcast_ref::<Button>() {
                        if let Some(emoji) = button.label().map(|s| s.to_string()) {
                            kopiere_und_schliesse(&emoji, &clipboard, &window, fenster_schliessen, &emoji_tabs::finde_begriffe(&emoji, &emojies_daten_suchfeld.borrow()));
                            break;
                        }
                    }
                }
            } else {
                // Kein Suchbegriff → Enter kopiert erstes Emoji aus history.list
                if let Some((_, grid)) = emojies_daten_suchfeld.borrow().get("🕓") {
                    let mut child = grid.first_child();
                    while let Some(widget) = child {
                        child = widget.next_sibling();
                        if let Some(button) = widget.downcast_ref::<Button>() {
                            if let Some(emoji) = button.label().map(|s| s.to_string()) {
                                kopiere_und_schliesse(&emoji, &clipboard, &window, fenster_schliessen, &emoji_tabs::finde_begriffe(&emoji, &emojies_daten_suchfeld.borrow()));
                                break;
                            }
                        }
                    }
                }
            }
        }));


        // Escape schließt Fenster
        let controller = EventControllerKey::new();
        let window_controller = Rc::clone(&window);
        controller.connect_key_pressed(move |_, keyval, _, _| {
            if keyval == gdk::Key::Escape {
                window_controller.close();
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        window.add_controller(controller);

        // Steuerung der Tabs mit Tab (leider ohne Shift+Tab)
        let controller_tab = EventControllerKey::new();
        controller_tab.set_propagation_phase(gtk::PropagationPhase::Capture);
        let notebook_controller_tab = notebook.clone();

        controller_tab.connect_key_pressed(move |_, keyval, _keycode, state| {
            use gdk::Key;

            if keyval == Key::Tab && !state.intersects(gdk::ModifierType::SHIFT_MASK) {
                let current = notebook_controller_tab.current_page().unwrap_or(0);
                let total = notebook_controller_tab.n_pages();
                notebook_controller_tab.set_current_page(Some((current + 1) % total));
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });

        if debug {
            println!("⏳ {} {:?}", sprachpaket.debug_main_time_set_window_keys, timer.elapsed());
        }

        window.add_controller(controller_tab);
        suchfeld.grab_focus();

        if zeige_infofenster {
            shortcut::zeige_setup_dialog(
                window.as_ref(),
                &einstellungen,
                Rc::clone(&sprachpaket),
                debug);

            if debug {
                println!("⏳ {} {:?}", sprachpaket.debug_main_time_info_window, timer.elapsed());
            }
        }

        // GTK-Fokus-Bug-Workaround: Doppelt aufrufen, damit das Fenster wirklich im Vordergrund erscheint
        window.present();
        window.present();

        if debug {
            println!("⏳ {} {:?}", sprachpaket.debug_main_time_create_ui, timer.elapsed());
        }
    });

    let empty: Vec<String> = vec![];
    app.run_with_args(&empty);
    // app.run();
}

// ███████╗██╗   ██╗███╗   ██╗ ██████╗████████╗██╗ ██████╗ ███╗   ██╗
// ██╔════╝██║   ██║████╗  ██║██╔════╝╚══██╔══╝██║██╔═══██╗████╗  ██║
// █████╗  ██║   ██║██╔██╗ ██║██║        ██║   ██║██║   ██║██╔██╗ ██║
// ██╔══╝  ██║   ██║██║╚██╗██║██║        ██║   ██║██║   ██║██║╚██╗██║
// ██║     ╚██████╔╝██║ ╚████║╚██████╗   ██║   ██║╚██████╔╝██║ ╚████║
// ╚═╝      ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝

fn kopiere_von_etc_falls_fehlend(dateiname: &str, sprachpaket: Rc<Sprache>, debug: &bool) {
    let ziel_pfad = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("emoji-picker")
        .join(dateiname);


    let etc_pfad = PathBuf::from("/etc/emoji-picker").join(dateiname);

    let muss_kopieren = match (fs::metadata(&ziel_pfad), fs::metadata(&etc_pfad)) {
        (Err(_), Ok(_)) => true, // Lokale Datei fehlt, aber etc-Datei existiert
        (Ok(local_meta), Ok(etc_meta)) => {
            let local_time = local_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let etc_time = etc_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            etc_time > local_time // etc ist neuer -> überschreiben
        }
        _ => false, // etc fehlt oder beides fehlt -> nichts tun
    };

    if muss_kopieren {
        let _ = std::fs::create_dir_all(ziel_pfad.parent().unwrap());
        if let Err(e) = fs::copy(&etc_pfad, &ziel_pfad) {
            eprintln!("❌ {} {}: {}", sprachpaket.debug_main_list_fail_to_copy, dateiname, e);
        } else {
            println!("📁 {} /etc/emoji-picker: {}", sprachpaket.debug_main_list_copy_from_etc, dateiname);
        }
    }else if dateiname == "history.list" && !ziel_pfad.exists() {
        // 🆕 history.list erstellen wenn nicht schon vorhanden
        let _ = std::fs::create_dir_all(ziel_pfad.parent().unwrap());
        let _ = std::fs::write(&ziel_pfad, "");
        if *debug {
            println!("📁 {}: {}", sprachpaket.debug_main_list_new_history, dateiname);
        }
    }

}

fn kopiere_und_schliesse(
    emoji: &str,
    clipboard: &Clipboard,
    window: &ApplicationWindow,
    schliessen: bool,
    begriffe: &[String],
) {
    clipboard.set_text(emoji);
    let zeile = format!("{} {}", emoji, begriffe.join(" "));

    // History speichern
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker/history.list");

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(pfad) {
        let _ = writeln!(file, "{}", zeile);
    }

    if schliessen {
        window.close();
    }
}