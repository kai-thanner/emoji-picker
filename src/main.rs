use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Entry, Label, Notebook, Orientation, PolicyType, ScrolledWindow, Button, Grid, Stack, EventControllerKey, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk::gdk::{self, Clipboard};
use std::fs;
use std::rc::Rc;
use std::path::PathBuf;
use std::collections::HashMap;

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
    if args.contains(&"--setup-shortcut".to_string()) {
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

        // Suchfeld anlegen
        let suchfeld = Entry::new();
        suchfeld.set_placeholder_text(Some("🔍 Suche nach Symbolnamen..."));
        vbox.append(&suchfeld);

        // Notebook für Kategorien
        let notebook = Rc::new(Notebook::new());
        notebook.set_vexpand(true);
        notebook.set_hexpand(true);

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
            ("symbole.list", "✅"),
            ("smileys.list", "😄"),
            ("peoples.list", "👨"),
            ("tiere.list", "🐰"),
            ("gestures.list", "👋"),
            ("clothing.list", "👕"),
            ("travel.list", "✈️"),
            ("acivity.list", "🏀"),
            ("nature.list", "🌲"),
            ("food.list", "🍌"),
            ("objects.list", "📎"),
            ("flags.list", "🇩🇪"),
        ];

        // Clipboard vorbereiten
        let display = gtk::gdk::Display::default().unwrap();
        let clipboard = Rc::new(display.clipboard());

        // Symboldaten laden & Grids aufbauen
        let symbol_daten: Rc<HashMap<String, (Vec<Symbol>, Rc<Grid>)>> = Rc::new(
            // UI Setup            
            kategorien.iter().map(|(datei, label)| {
                // .list Dateien anlegen falls nicht vorhanden
                kopiere_von_skel_falls_fehlend(datei);

                // Lade Symbole
                let symbole = lade_symbole(datei);
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
                let label_widget = Label::new(Some(label));
                let css = format!(
                    "label {{ font-size: {}px; padding: 2px; }}",
                    *emoji_size
                );
                let provider = CssProvider::new();
                provider.load_from_data(&css);
                label_widget.style_context().add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
 
                notebook.append_page(&scroll, Some(&label_widget));
                (label.to_string(), (symbole, Rc::clone(&grid)))
            }).collect(),
        );

        // Symbole in Kategorien einfügen
        for (_label, (symbole, grid)) in symbol_daten.iter() {
            let mut i = 0;
            for symbol in symbole.iter() {
                // Emoji grösse
                let button = Button::with_label(&symbol.emoji);
                button.set_focusable(false);
                button.set_size_request(*emoji_size, *emoji_size);
                let css = format!("button {{ font-size: {}px; }}", *emoji_size);
                let provider = CssProvider::new();
                provider.load_from_data(&css);
                button.style_context().add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);

                button.set_hexpand(false);
                button.set_halign(gtk::Align::Center);

                // Tooltip mit Suchbegriffen
                button.set_tooltip_text(Some(&symbol.begriffe.join(", ")));

                let emoji = symbol.emoji.clone();
                let clipboard = Rc::clone(&clipboard);
                let window = Rc::clone(&window);
                let schliessen = Rc::clone(&fenster_schliessen);

                button.connect_clicked(move |_| {
                    kopiere_und_schliesse(&emoji, &clipboard, &window, *schliessen.as_ref());
                });

                let row = i / 10;
                let col = i % 10;
                grid.attach(&button, col as i32, row as i32, 1, 1);
                i += 1;
            }
        }

        //Suchlogic
        let symbol_daten_clone = Rc::clone(&symbol_daten);
        let such_grid_clone = Rc::clone(&such_grid);
        let clipboard_clone = Rc::clone(&clipboard);
        let window_clone = Rc::clone(&window);
        let schliessen_clone = Rc::clone(&fenster_schliessen);
        let emoji_size_clone = Rc::clone(&emoji_size);

        suchfeld.connect_changed(move |entry| {
            let text = entry.text().to_string().to_lowercase();

            if text.is_empty() {
                stack.set_visible_child_name("notebook");
                return;
            }

            // Suche aktivieren
            stack.set_visible_child_name("suche");

            // Suchgrid leeren
            let mut child = such_grid_clone.first_child();
            while let Some(widget) = child {
                child = widget.next_sibling();
                such_grid_clone.remove(&widget);
            }
            let mut i = 0;
            for (_label, (symbole, _)) in symbol_daten_clone.iter() {
                for symbol in symbole.iter() {
                        let filter_text = text.trim().to_lowercase();
                        let filter_kompakt = filter_text.replace(' ', "");

                        let filter_wörter: Vec<_> = filter_text.split_whitespace().collect();
                        let joined = symbol.begriffe.join("").to_lowercase();
                        let begriffe_vec = symbol.begriffe.iter().map(|s| s.to_lowercase()).collect::<Vec<String>>();

                        // Kombinationen aus direkt benachbarten Wörtern (Fenster)
                        let mut kombis_fenster = (2..=begriffe_vec.len())
                            .flat_map(|n| begriffe_vec.windows(n).map(|w| w.join("")));

                        let passt = joined.contains(&filter_kompakt)
                            || filter_wörter
                                .iter()
                                .all(|wort| symbol.begriffe.iter().any(|b| b.contains(wort)))
                            || kombis_fenster
                                .any(|kombi| kombi.contains(&filter_kompakt));

                        if passt {
                        let button = Button::with_label(&symbol.emoji);
                        button.set_size_request(*emoji_size_clone, *emoji_size_clone);
                        let css = format!("button {{ font-size: {}px; }}", *emoji_size_clone);
                        let provider = CssProvider::new();
                        provider.load_from_data(&css);
                        button.style_context().add_provider(&provider, STYLE_PROVIDER_PRIORITY_APPLICATION);

                        button.set_hexpand(false);
                        button.set_halign(gtk::Align::Center);

                        let emoji = symbol.emoji.clone();
                        let clipboard = Rc::clone(&clipboard_clone);
                        let window = Rc::clone(&window_clone);
                        let schliessen = Rc::clone(&schliessen_clone);

                        button.set_tooltip_text(Some(&symbol.begriffe.join(", ")));

                        button.connect_clicked(move |_| {
                            kopiere_und_schliesse(&emoji, &clipboard, &window, *schliessen.as_ref());
                        });

                        let row = i / 10;
                        let col = i % 10;
                        such_grid_clone.attach(&button, col as i32, row as i32, 1, 1);
                        i += 1;
                    }
                }
            }
        });

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
    });

    app.run();
}

fn kopiere_von_skel_falls_fehlend(dateiname: &str) {
    let ziel_pfad = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("emoji-picker")
        .join(dateiname);

    if !ziel_pfad.exists() {
        let skel_pfad = PathBuf::from("/etc/skel/.config/emoji-picker").join(dateiname);
        if skel_pfad.exists() {
            let _ = std::fs::create_dir_all(ziel_pfad.parent().unwrap());
            let _ = std::fs::copy(&skel_pfad, &ziel_pfad);
            println!("📁 Kopiert aus /etc/skel: {}", dateiname);
        }
    }
}

fn kopiere_und_schliesse(emoji: &str, clipboard: &Clipboard, window: &ApplicationWindow, schliessen: bool) {
    clipboard.set_text(emoji);
    if schliessen {
        window.close();
    }
}

fn lade_symbole(dateiname: &str) -> Vec<Symbol> {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker");
    pfad.push(dateiname);
    fs::read_to_string(pfad).unwrap_or_default().lines().filter_map(|zeile| {
        let mut parts = zeile.split_whitespace();
        let emoji = parts.next()?.to_string();
        let begriffe = parts.map(|s| s.to_lowercase()).collect();
        Some(Symbol { emoji, begriffe })
    }).collect()
}

fn lade_settings() -> Einstellungen {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker/settings.ini");
    if !pfad.exists() {
        let _ = fs::create_dir_all(pfad.parent().unwrap());
        let _ = fs::write(&pfad, "[Allgemein]\nsetup_erledigt = false\nfenster_schliessen = true\nemoji_size = 20\n");
    }
    let content = fs::read_to_string(&pfad).unwrap_or_default();
    println!("🔧 settings.ini Inhalt:\n{}", content);
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
    println!("🧪 Auswertung: setup_erledigt = {} | fenster_schliessen = {} | emoji_size = {}", setup_erledigt, fenster_schliessen, emoji_size);
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