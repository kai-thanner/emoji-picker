mod suchlogik;

use suchlogik::verbinde_suchfeld;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Entry, EventControllerKey,
    Grid, Label, Notebook, Orientation, PolicyType, ScrolledWindow, Stack,
    STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use gtk::gdk::{self, Clipboard};
use std::{
    collections::HashMap,
    fs, fs::OpenOptions,
    path::PathBuf,
    rc::Rc,
    time::Instant,
    io::Write,
};
use rayon::prelude::*;

#[derive(Clone)]
struct Symbol {
    emoji: String,
    begriffe: Vec<String>,
}

struct Einstellungen {
    setup_erledigt: bool,
    fenster_schliessen: bool,
    emoji_size: i32,
}

fn main() {
    // Zeitmessung für Programmstart
    let debug_startzeit = 0;
    let start = Instant::now();
    if debug_startzeit == 1 {
        println!("Programmstart bei {:?}", start.elapsed());
    }

    // settings.ini auslesen / erstellen
    let mut einstellungen = lade_settings();

    // Konfiguration zwischenspeichern
    let fenster_schliessen = Rc::new(einstellungen.fenster_schliessen);
    let emoji_size = Rc::new(einstellungen.emoji_size);

    let zeige_infofenster = !einstellungen.setup_erledigt;

    if zeige_infofenster {
        setup_shortcut();
        einstellungen.setup_erledigt = true;
        speichere_settings(&einstellungen);
    }

    // Argumente abfangen
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--setup".to_string()) {
        setup_shortcut();
        return;
    }

    let app: Application = Application::builder()
        .application_id("com.kai_thanner.emoji-picker")
        .build();

    app.connect_activate(move |app| {
        // Fenster erstellen
        let window = Rc::new(ApplicationWindow::builder()
            .application(app)
            .title("Emoji-Auswahl")
            .default_width(400)
            .default_height(400)
            .build()
        );

        // Hauptlayout
        let vbox = GtkBox::new(Orientation::Vertical, 5);
        window.set_child(Some(&vbox));
     
        // CSS für UI laden
        let global_css = CssProvider::new();
        for pfad in [
            "/usr/share/emoji-picker/emoji-picker.css",
            "./emoji-picker.css" // nur für Entwicklung
        ] {
            if fs::metadata(pfad).is_ok() {
                global_css.load_from_path(pfad);
                gtk::style_context_add_provider_for_display(
                    &gdk::Display::default().unwrap(),
                    &global_css,
                    STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
                break;
            } else {
                eprintln!("🚫 CSS-Datei nicht gefunden: {}", pfad);
            }
        }
        if debug_startzeit == 1 {
            println!("🖌 CSS-Datei geladen in: {:?}", start.elapsed());
        }

        // Suchfeld anlegen
        let suchfeld = Entry::new();
        suchfeld.add_css_class("search-entry");
        suchfeld.set_placeholder_text(Some("🔍 Suche nach Symbolnamen..."));
        vbox.append(&suchfeld);

        // Notebook für Kategorien
        let notebook = Rc::new(Notebook::new());
        notebook.set_vexpand(true);
        notebook.set_hexpand(true);

        // Symbolgröße
        let css = format!("button.emoji {{ font-size: {}px; }}", *emoji_size);
        let emoji_css = CssProvider::new();
        emoji_css.load_from_data(&css);
        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &emoji_css,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );


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
            kopiere_von_etc_falls_fehlend(datei);
        }

        if debug_startzeit == 1 {
            println!("📁 /etc/emoji-picker Kopieren fertig nach {:?}", start.elapsed());
        }

        // Symbole parallel Laden
        let lade_daten: Vec<(String, Vec<Symbol>)> = kategorien
            .par_iter()
            .map(|(datei, label)| {
                // Lade Symbole - jetzt parallel (Nebenläufig)
                let symbole = lade_symbole(datei);
                (label.to_string(), symbole)
            })
            .collect();

        if debug_startzeit == 1 {
            println!("🙂 Emojis geladen in: {:?}", start.elapsed());
        }

        // Suchindex erstellen (flache Liste aller Symbole)
        let such_index = Rc::new(
            lade_daten
                .iter()
                .flat_map(|(_, symbole)| symbole.clone())
                .collect::<Vec<_>>()
        );

        if debug_startzeit == 1 {
            println!("🔍 Suchindex erstellt in: {:?}", start.elapsed());
        }

        // Grids aufbauen     
        let symbol_daten: Rc<HashMap<String, (Vec<Symbol>, Rc<Grid>)>> = Rc::new(
            // UI Setup            
            lade_daten
                .into_iter()
                .map(|(label, symbole)| {
                    let grid = Rc::new(Grid::new());
                    grid.set_row_spacing(5);
                    grid.set_column_spacing(5);
                    grid.set_margin_top(10);
                    grid.set_margin_bottom(10);
                    grid.set_margin_start(12);
                    grid.set_margin_end(12);

                    // Scroll-Container erzeugen und vbox einbetten
                    let scroll = ScrolledWindow::new();
                    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
                    scroll.set_child(Some(&*grid));

                    // Emoji-Größe für die Kategorie-Labels
                    let label_widget = Label::new(Some(&label));
                    let css = format!(
                        "label {{ font-size: {}px; padding: 2px; }}",
                        *emoji_size
                    );
                    emoji_css.load_from_data(&css);
                    label_widget.add_css_class("kategorie-tab");
                    label_widget.style_context().add_provider(&emoji_css, STYLE_PROVIDER_PRIORITY_APPLICATION);
     
                    notebook.append_page(&scroll, Some(&label_widget));
                    (label, (symbole, Rc::clone(&grid)))
                })
                .collect(),
        );

        if debug_startzeit == 1 {
            println!("🕸 Grid aufgebaut in {:?}", start.elapsed());
        }

        // Symbole in Kategorien einfügen
        for (_label, (symbole, grid)) in symbol_daten.iter() {
            let mut buttons = Vec::new();

            for symbol in symbole.iter() {
                let button = Button::with_label(&symbol.emoji);
                button.set_focusable(false);
                button.set_size_request(*emoji_size, *emoji_size);
                button.add_css_class("emoji");

                button.set_hexpand(false);
                button.set_halign(gtk::Align::Center);

                // Tooltip mit Suchbegriffen
                let begriffe = symbol.begriffe.join(", ");
                button.set_has_tooltip(true);
                button.connect_query_tooltip(move |_, _, _, _, tooltip| {
                    tooltip.set_text(Some(&begriffe));
                    true // Zeige Tooltip
                });

                let emoji = symbol.emoji.clone();
                let clipboard = Rc::clone(&clipboard);
                let window = Rc::clone(&window);
                let schliessen = Rc::clone(&fenster_schliessen);

                button.connect_clicked(move |_| {
                    kopiere_und_schliesse(&emoji, &clipboard, &window, *schliessen.as_ref());
                });

                buttons.push(button);
            }

            for (i, button) in buttons.into_iter().enumerate() {
                let row = i / 13;
                let col = i % 13;
                grid.attach(&button, col as i32, row as i32, 1, 1);
            }
        }

        if debug_startzeit == 1 {
            println!("📥 Emojis in Kategorien eingefügt in {:?}", start.elapsed());
        }

        //Suchlogik
        verbinde_suchfeld(
            &suchfeld,
            Rc::clone(&such_grid),
            stack.clone(),
            Rc::clone(&such_index),
            Rc::clone(&clipboard),
            Rc::clone(&window),
            Rc::clone(&fenster_schliessen),
            Rc::clone(&emoji_size),
        );

        if debug_startzeit == 1 {
            println!("🔍 Suchfeld erzeugt in {:?}", start.elapsed());
        }

        // Mit Enter das erste Symbol auswählen
        let such_grid_clone2 = Rc::clone(&such_grid);
        let clipboard_clone2 = Rc::clone(&clipboard);
        let window_clone2 = Rc::clone(&window);
        let schliessen2 = Rc::clone(&fenster_schliessen);

        suchfeld.connect_activate(move |_| {
            let mut child = such_grid_clone2.first_child();
            while let Some(widget) = child {
                child = widget.next_sibling();
                if let Some(button) = widget.downcast_ref::<Button>() {
                    if let Some(emoji) = button.label() {
                        kopiere_und_schliesse(&emoji, &clipboard_clone2, &window_clone2, *schliessen2.as_ref());
                        break;
                    }
                }
            }
        });

        // Escape schließt Fenster
        let controller = EventControllerKey::new();
        let win_clone = Rc::clone(&window);
        controller.connect_key_pressed(move |_, keyval, _, _| {
            if keyval == gdk::Key::Escape {
                win_clone.close();
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        window.add_controller(controller);

        // Steuerung der Tabs mit Tab ohne Shift+Tab
        let controller_tab = EventControllerKey::new();
        controller_tab.set_propagation_phase(gtk::PropagationPhase::Capture);
        let notebook_clone_tab = notebook.clone();

        controller_tab.connect_key_pressed(move |_, keyval, _keycode, state| {
            use gdk::Key;

            if keyval == Key::Tab && !state.intersects(gdk::ModifierType::SHIFT_MASK) {
                let current = notebook_clone_tab.current_page().unwrap_or(0);
                let total = notebook_clone_tab.n_pages();
                notebook_clone_tab.set_current_page(Some((current + 1) % total));
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });

        if debug_startzeit == 1 {
            println!("🕹 Fenstersteuerung erstellt in {:?}", start.elapsed());
        }

        window.add_controller(controller_tab);
        suchfeld.grab_focus();

        if zeige_infofenster {
            let dialog = gtk::MessageDialog::builder()
                .transient_for(window.as_ref())
                .modal(true)
                .message_type(gtk::MessageType::Info)
                .buttons(gtk::ButtonsType::Ok)
                .text("Emoji Picker eingerichtet 🎉")
                .secondary_text("Du kannst ihn ab sofort mit Super+. starten.\n\nHinweis: Wenn die Tastenkombi noch nicht funktioniert, öffne die Tastatureinstellungen und bestätige den Eintrag.")
                .build();

            dialog.connect_response(|dialog, _| {
                dialog.close();
            });

            dialog.show();
        }

        window.present();

        if debug_startzeit == 1 {
            println!("🪟 UI erzeugt in {:?}", start.elapsed());
        }
    });

    app.run();
}

// ███████╗██╗   ██╗███╗   ██╗ ██████╗████████╗██╗ ██████╗ ███╗   ██╗
// ██╔════╝██║   ██║████╗  ██║██╔════╝╚══██╔══╝██║██╔═══██╗████╗  ██║
// █████╗  ██║   ██║██╔██╗ ██║██║        ██║   ██║██║   ██║██╔██╗ ██║
// ██╔══╝  ██║   ██║██║╚██╗██║██║        ██║   ██║██║   ██║██║╚██╗██║
// ██║     ╚██████╔╝██║ ╚████║╚██████╗   ██║   ██║╚██████╔╝██║ ╚████║
// ╚═╝      ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝

fn kopiere_von_etc_falls_fehlend(dateiname: &str) {
    let ziel_pfad = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("emoji-picker")
        .join(dateiname);

    if !ziel_pfad.exists() {
        let etc_pfad = PathBuf::from("/etc/emoji-picker").join(dateiname);
        if etc_pfad.exists() {
            let _ = std::fs::create_dir_all(ziel_pfad.parent().unwrap());
            let _ = std::fs::copy(&etc_pfad, &ziel_pfad);
            println!("📁 Kopiert aus /etc/emoji-picker: {}", dateiname);
        }else if dateiname == "history.list" {
            // 🆕 history.list erstellen wenn nicht schon vorhanden
            let _ = std::fs::create_dir_all(ziel_pfad.parent().unwrap());
            let _ = std::fs::write(&ziel_pfad, "");
            println!("📁 Erstellt: {}", dateiname);
        }
    }
}

fn kopiere_und_schliesse(emoji: &str, clipboard: &Clipboard, window: &ApplicationWindow, schliessen: bool) {
    clipboard.set_text(emoji);

    // History speichern
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker/history.list");

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(pfad) {
        let _ = writeln!(file, "{}", emoji);
    }

    if schliessen {
        window.close();
    }
}

fn lade_symbole(dateiname: &str) -> Vec<Symbol> {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker");
    pfad.push(dateiname);
    let inhalt = fs::read_to_string(pfad)        // Datei (pfad) als String lesen
                        .unwrap_or_default();    // Wenn Datei fehlt oder fehlerhaft, ersetze durch leeren String "" - verhindert einen crash

    if dateiname == "history.list" {
        // History: Emojis zählen, sortieren, eindeutige Einträge erzeugen
        let mut zaehler = std::collections::HashMap::new();

        for zeile in inhalt.lines().map(str::trim).filter(|l| !l.is_empty()) {
            *zaehler.entry(zeile.to_string()).or_insert(0) += 1;
        }

        let mut eintraege: Vec<_> = zaehler.into_iter().collect();
        eintraege.sort_by(|a, b| b.1.cmp(&a.1)); // nach Häufigkeit, absteigend

        eintraege.into_iter()
            .take(30)                   // auf die 30 häufigsten Begrenzt
            .map(|(emoji, _)| Symbol {
                emoji,
                begriffe: vec!["history".to_string()],
            })
            .collect()
    } else {   
        // Normale .list-Dateien: klassisch einlesen
        inhalt
            .lines()                // Zeile für Zeile
            .filter_map(|zeile| {
                let mut parts = zeile.split_whitespace();                   // Trenne an Leerzeichen
                let emoji = parts.next()?.to_string();                      // Erstes Element ist das Emoji
                let begriffe = parts.map(|s| s.to_lowercase()).collect();   // Dahinter alle Begriffe kleingeschrieben
                Some(Symbol { emoji, begriffe })
            }).collect()            // wandelt Some(Symbol) in Vec<Symbol> um
    }
}

fn lade_settings() -> Einstellungen {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker/settings.ini");
    if !pfad.exists() {
        let _ = fs::create_dir_all(pfad.parent().unwrap());
        let _ = fs::write(&pfad, "[Allgemein]\nsetup_erledigt = false\nfenster_schliessen = true\nemoji_size = 20\n");
    }
    let content = fs::read_to_string(&pfad).unwrap_or_default();
    // println!("🔧 settings.ini Inhalt:\n{}", content);
    let mut setup_erledigt = false;
    let mut fenster_schliessen = true;
    let mut emoji_size = 20;
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
        } else if line.starts_with("emoji_size") {
            if let Some(value) = line.split('=').nth(1) {
                emoji_size = value.trim().parse().unwrap_or(30);
            }
        }
    }
    // println!("🧪 Auswertung: setup_erledigt = {} | fenster_schliessen = {} | emoji_size = {}", setup_erledigt, fenster_schliessen, emoji_size);
    Einstellungen { setup_erledigt, fenster_schliessen, emoji_size }
}

fn setup_shortcut() {
    use std::process::Command;
    use std::env;

    println!("🛠 Versuche, Tastenkombi <Super>+. zu setzen...");

    if env::var("DISPLAY").is_err() {
        eprintln!("❌ Keine DISPLAY-Umgebung gefunden. Bitte im Desktop-Terminal ausführen.");
        return;
    }

    let cmds = vec![
        ("gsettings", &["set", "org.cinnamon.desktop.keybindings", "custom-list", "['custom0']"]),
        ("gsettings", &["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/custom0/", "name", "Emoji Picker"]),
        ("gsettings", &["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/custom0/", "command", "emoji-picker"]),
        ("gsettings", &["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/custom0/", "binding", "['<Super>period']"]),
    ];

    for (cmd, args) in cmds {
        let status = Command::new(cmd).args(args).status();
        match status {
            Ok(s) if s.success() => continue,
            Ok(s) => eprintln!("⚠️  {} exit code {}", cmd, s.code().unwrap_or(-1)),
            Err(e) => eprintln!("❌ Fehler beim Aufruf von {}: {}", cmd, e),
        }
    }

    println!("✅ Tastenkombination eingerichtet (sofern möglich).");
}

fn speichere_settings(einstellungen: &Einstellungen) {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker/settings.ini");

    let inhalt = format!(
        "[Allgemein]\nsetup_erledigt = true\nfenster_schliessen = {}\nemoji_size = {}\n",
        einstellungen.fenster_schliessen,
        einstellungen.emoji_size
    );

    let _ = fs::write(&pfad, inhalt);
}