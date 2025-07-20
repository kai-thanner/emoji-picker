use gtk::prelude::*;
use gtk::{ApplicationWindow, Button, DragSource, Grid, Label, Notebook, PolicyType, ScrolledWindow, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk::gdk;
use gtk::gdk::{Clipboard, ContentProvider, DragAction, Toplevel};
use std::collections::HashMap;
use std::{
    cell::RefCell,
    fs,
    path::PathBuf,
    rc::Rc,
};

use crate::{settings::Einstellungen};
use crate::i18n::Sprache;

#[derive(Clone)]
pub struct Symbol {
    pub emoji: String,
    pub begriffe: Vec<String>,
    pub zaehler: usize
}

pub fn erstelle_tabs(
    notebook: &Notebook,
    kategorien: &[(impl AsRef<str>, impl AsRef<str>)],
    emoji_size: i32,
) -> HashMap<String, (Vec<Symbol>, Rc<Grid>)> {
    let mut emoji_daten = HashMap::new();

    for (datei, label) in kategorien {
        let emojies = lade_emojies(datei.as_ref());

        let grid = grid();

        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        scroll.set_child(Some(&*grid));

        let label_widget = Label::new(Some(label.as_ref()));
        label_widget.add_css_class("kategorie-tab");
        aktualisiere_tablabel_style(emoji_size);

        notebook.append_page(&scroll, Some(&label_widget));
        emoji_daten.insert(datei.as_ref().to_string(), (emojies, Rc::clone(&grid)));
        // emoji_daten.insert(label.as_ref().to_string(), (emojies, Rc::clone(&grid)));
    }

    emoji_daten
}

pub fn fuege_emojis_ein(
    emojies_daten: Rc<RefCell<HashMap<String, (Vec<Symbol>, Rc<Grid>)>>>,
    clipboard: Rc<Clipboard>,
    window: Rc<ApplicationWindow>,
    einstellungen: Rc<Einstellungen>,
) {
    let emoji_size = einstellungen.emoji_size.get();

    for (label, (symbole, grid)) in emojies_daten.borrow().iter() {
        let mut buttons = Vec::new();

        for symbol in symbole.iter() {
            let button = Button::with_label(&symbol.emoji);
            button.set_size_request(emoji_size, emoji_size);
            button.add_css_class("emoji");
            button.set_focusable(true);
            button.set_hexpand(false);
            button.set_halign(gtk::Align::Center);

            // Tooltip
            let begriffe = symbol.begriffe.join(", ");
            button.set_has_tooltip(true);
            button.connect_query_tooltip(move |_, _, _, _, tooltip| {
                tooltip.set_text(Some(&begriffe));
                true
            });

            // Klick
            let emoji = symbol.emoji.clone();
            let label_click = label.clone();           // z.B. "😄" für smileys.list
            let emojies_daten_click = Rc::clone(&emojies_daten);
            let clipboard_click = Rc::clone(&clipboard);
            let window_click = Rc::clone(&window);
            let einstellungen_click = Rc::clone(&einstellungen);
            button.connect_clicked(move |_| {
                let schliessen = einstellungen_click.fenster_schliessen.get();
                speichere_kopiere_und_schliesse(
                    &emoji,
                    Rc::clone(&emojies_daten_click),
                    Some(&label_click),
                    Some(&clipboard_click),
                    &window_click,
                    schliessen,
                );
            });

            // Drag & Drop
            let emoji_clone = symbol.emoji.clone();
            let emoji_zaehlen = symbol.emoji.clone();
            let emoji_daten_drag = emojies_daten.clone();
            let dateiname_zaehlen = if label != "🕓" { Some(label.clone()) } else {None};
            let window_drag = Rc::clone(&window);
            let einstellungen_drag = Rc::clone(&einstellungen);

            let drag_source = DragSource::new();
            drag_source.set_actions(DragAction::COPY);
            drag_source.connect_prepare(move |_, _, _| {
                Some(ContentProvider::for_value(&emoji_clone.to_value()))
            });
            drag_source.connect_drag_end(move |_, _, _| {
                // Nur Zähler erhöhen – kein Kopieren, kein Fenster schließen
                speichere_kopiere_und_schliesse(
                    &emoji_zaehlen,
                    Rc::clone(&emoji_daten_drag),
                    dateiname_zaehlen.as_deref(),
                    None,
                    &window_drag,
                    false,
                );

                if !einstellungen_drag.fenster_offen_bei_drag.get() {
                    window_drag.close();
                }
            });

            button.add_controller(drag_source);
            buttons.push(button);
        }

        for (i, button) in buttons.into_iter().enumerate() {
            let row = i / 14;
            let col = i % 14;
            grid.attach(&button, col as i32, row as i32, 1, 1);
        }
    }
}

fn lade_emojies(dateiname: &str) -> Vec<Symbol> {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker");
    pfad.push(dateiname);
    let inhalt = fs::read_to_string(pfad)        // Datei (pfad) als String lesen
                        .unwrap_or_default();    // Wenn Datei fehlt oder fehlerhaft, ersetze durch leeren String "" - verhindert einen crash

    inhalt
        .lines()                // Zeile für Zeile
        .filter_map(|zeile| {
            let mut parts = zeile.split_whitespace();                   // Trenne an Leerzeichen
            let emoji = parts.next()?.to_string();                      // Erstes Element ist das Emoji

            let mut zaehler = 0;
            let mut begriffe = Vec::new();

            if let Some(erster) = parts.next() {
                if let Some(rest) = erster.strip_suffix(':') {
                    // Format: ☺️ 12: ...
                    zaehler = rest.parse().unwrap_or(0);
                    begriffe = parts.map(|s| s.to_lowercase()).collect();
                } else {
                    // Kein Zaehler -> erster Begriff ist normal
                    begriffe.push(erster.to_lowercase());
                    begriffe.extend(parts.map(|s| s.to_lowercase()));   // Dahinter alle Begriffe kleingeschrieben
                }
            }
            Some(Symbol { emoji, begriffe, zaehler })
        }).collect()            // wandelt Some(Symbol) in Vec<Symbol> um
    
}

pub fn speichere_kopiere_und_schliesse(
    emoji: &str,
    daten: Rc<RefCell<HashMap<String, (Vec<Symbol>, Rc<Grid>)>>>,
    datei: Option<&str>,
    clipboard: Option<&Clipboard>,
    window: &ApplicationWindow,
    schliessen: bool,
) {
    let mut daten = daten.borrow_mut();

    // Entweder direkt via Label (schnell) oder Such-Schleife
    let eintrag = if let Some(datei) = datei {
        daten
            .get_mut(datei)
            .map(|(symbole, _)| (datei.to_string(), symbole))
    } else {
        daten
            .iter_mut()
            .find(|(_, (symbole, _))| symbole.iter().any(|s| s.emoji == emoji))
            .map(|(datei, (symbole, _))| (datei.clone(), symbole))
    };

    if let Some((datei, symbole)) = eintrag {
        if let Some(s) = symbole.iter_mut().find(|s| s.emoji == emoji) {
            s.zaehler += 1;
            speichere_emojies(&format!("{}", datei), symbole);
        }
    }

    if let Some(cb) = clipboard {
        cb.set_text(emoji);

        // 📋 Debug-Ausgabe aktiv?
        glib::timeout_add_once(std::time::Duration::from_millis(100), move || {
            let cb = gtk::gdk::Display::default().unwrap().clipboard();
            cb.read_text_async(None::<&gtk::gio::Cancellable>, move |res| {
                match res {
                    Ok(Some(text)) => println!("📋 Clipboard-Check: '{}'", text),
                    Ok(None) => println!("⚠️  Clipboard ist leer"),
                    Err(e) => eprintln!("❌ Fehler beim Lesen des Clipboards: {}", e),
                }
            });
        });
        
    }

    if schliessen {
        if let Some(surface) = window.surface() {
            if let Some(gdk_window) = surface.downcast::<Toplevel>().ok() {
                // gdk_window.minimize();
                gdk_window.hide();
            }
        } else {
                window.close();
        }
    }
}

fn speichere_emojies(dateiname: &str, symbole: &[Symbol]) {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker");
    pfad.push(dateiname);

    let mut zeilen = Vec::new();

    for symbol in symbole {
        let zaehler = symbol.zaehler;
        let emoji = &symbol.emoji;
        let begriffe = symbol.begriffe.join(" ");

        // Format: ☺️ 12: lächeln grinsen
        let zeile = if zaehler > 0 {
            format!("{} {}: {}", emoji, zaehler, begriffe)
        } else {
            format!("{} {}", emoji, begriffe)
        };

        zeilen.push(zeile);
    }

    let _ = fs::write(&pfad, zeilen.join("\n"));
}

// Erzeuge aus allen Symbolen die Top-100 History-Liste nach Nutzung
pub fn generiere_history_kategorie(
    daten: &HashMap<String, (Vec<Symbol>, Rc<Grid>)>,
) -> (Vec<Symbol>, Rc<Grid>) {

    // Alle Symbole aus allen Kategorien sammeln
    let mut symbole_alle: Vec<Symbol> = daten
        .iter()
        .flat_map(|(_, (symbole, _))| symbole.clone())
        .filter(|s| s.zaehler > 0)
        .collect();

    // Nach Nutzung sortieren (absteigend) und auf 100 begrenzen
    symbole_alle.sort_by(|a, b| b.zaehler.cmp(&a.zaehler));
    symbole_alle.truncate(100);

    // Neuen Grid erzeugen
    let grid = grid();

    (symbole_alle, grid)
}

fn grid() -> Rc<Grid> {
    let grid = Rc::new(Grid::new());
    grid.set_row_spacing(5);
    grid.set_column_spacing(5);
    grid.set_margin_top(10);
    grid.set_margin_bottom(10);
    grid.set_margin_start(12);
    grid.set_margin_end(12);
    grid
}

pub fn lade_ui_css(sprachpaket: Rc<Sprache>, debug: bool) {
	let global_css = CssProvider::new();
    let css_pfade = if cfg!(debug_assertions) {
        vec![   // nur für Entwicklung
            PathBuf::from(format!("../assets/usr/share/emoji-picker/emoji-picker.css")),    // // start aus emoji-picker/src/
            PathBuf::from(format!("./assets/usr/share/emoji-picker/emoji-picker.css")),     // start aus emoji-picker/
        ]
    } else {
        vec![  
            PathBuf::from(format!("/usr/share/emoji-picker/emoji-picker.css"))
        ]
    };

    for pfad in &css_pfade {
        if fs::metadata(pfad).is_ok() {
            global_css.load_from_path(pfad);
            gtk::style_context_add_provider_for_display(
                &gdk::Display::default().unwrap(),
                &global_css,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            if debug {
                println!("📤 {}: {:?}", sprachpaket.debug_emoji_tabs_use_css, pfad);
            }
            break;
        } else {
            if debug {
                eprintln!("🚫 {}: {:?}", sprachpaket.debug_emoji_tabs_css_failure, pfad);
            }
        }
    }
}

pub fn aktualisiere_tablabel_style(emoji_size: i32) {
    let css = format!("label.kategorie-tab {{ font-size: {}px; padding: 2px; }}", emoji_size);
    let provider = CssProvider::new();
    provider.load_from_data(&css);
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn aktualisiere_emoji_style(emoji_size: i32) {
    let css = format!("button.emoji {{ font-size: {}px; }}", emoji_size);
    let provider = CssProvider::new();
    provider.load_from_data(&css);
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn leere_history_tab(
    emojies_daten: &Rc<RefCell<HashMap<String, (Vec<Symbol>, Rc<Grid>)>>>,
    sprachpaket: Rc<Sprache>,
    debug: bool,
) {
    let mut emojies_daten = emojies_daten.borrow_mut();

    let history = emojies_daten.remove("🕓");
    if let Some((mut vec_emojies, grid)) = history {
        // Entferne Buttons aus dem Grid
        while let Some(child) = grid.first_child() {
            grid.remove(&child);
        }

        // Alle Zähler zurücksetzen und speichern
        for (dateiname, (symbole, _)) in emojies_daten.iter_mut() {
            if dateiname != "🕓" {
                for symbol in symbole.iter_mut() {
                    symbol.zaehler = 0;
                }
            }
            speichere_emojies(dateiname, symbole);
        }

        // Auch im 🕓-Tab selbst den Speicher leeren
        vec_emojies.clear();

        // Leeren 🕓-Tab wieder einfügen
        emojies_daten.insert("🕓".to_string(), (vec_emojies, grid));
    }

    if debug {
        println!("🧹 {}", sprachpaket.debug_emoji_tabs_clear_history);
    }
}