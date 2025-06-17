use gtk::prelude::*;
use gtk::{ApplicationWindow, Button, DragSource, Grid, Label, Notebook, PolicyType, ScrolledWindow, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use gtk::gdk;
use gtk::gdk::{Clipboard, ContentProvider, DragAction};
use std::collections::HashMap;
use std::{
    cell::RefCell,
    fs,
    path::PathBuf,
    rc::Rc,
};

use crate::{settings::Einstellungen, kopiere_und_schliesse};

#[derive(Clone)]
pub struct Symbol {
    pub emoji: String,
    pub begriffe: Vec<String>,
}

pub fn erstelle_tabs(
    notebook: &Notebook,
    kategorien: &[(impl AsRef<str>, impl AsRef<str>)],
    emoji_size: i32,
) -> HashMap<String, (Vec<Symbol>, Rc<Grid>)> {
    let mut emoji_daten = HashMap::new();

    for (datei, label) in kategorien {
        let emojies = lade_emojies(datei.as_ref());

        let grid = Rc::new(Grid::new());
        grid.set_row_spacing(5);
        grid.set_column_spacing(5);
        grid.set_margin_top(10);
        grid.set_margin_bottom(10);
        grid.set_margin_start(12);
        grid.set_margin_end(12);

        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        scroll.set_child(Some(&*grid));

        let label_widget = Label::new(Some(label.as_ref()));
        label_widget.add_css_class("kategorie-tab");
        aktualisiere_tablabel_style(emoji_size);

        notebook.append_page(&scroll, Some(&label_widget));
        emoji_daten.insert(label.as_ref().to_string(), (emojies, Rc::clone(&grid)));
    }

    emoji_daten
}

pub fn fuege_emojis_ein(
    emojies_daten: &HashMap<String, (Vec<Symbol>, Rc<Grid>)>,
    clipboard: Rc<Clipboard>,
    window: Rc<ApplicationWindow>,
    einstellungen: Rc<Einstellungen>,
) {
    let emoji_size = einstellungen.emoji_size.get();

    for (_label, (symbole, grid)) in emojies_daten.iter() {
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
            let begriffe = symbol.begriffe.clone();
            let clipboard = Rc::clone(&clipboard);
            let window_click = Rc::clone(&window);
            let einstellungen_click = Rc::clone(&einstellungen);
            button.connect_clicked(move |_| {
                let schliessen = einstellungen_click.fenster_schliessen.get();
                kopiere_und_schliesse(&emoji, &clipboard, &window_click, schliessen, &begriffe);
            });

            // Drag & Drop
            let emoji_clone = symbol.emoji.clone();
            let window_drag = Rc::clone(&window);
            let einstellungen_drag = Rc::clone(&einstellungen);

            let drag_source = DragSource::new();
            drag_source.set_actions(DragAction::COPY);
            drag_source.connect_prepare(move |_, _, _| {
                Some(ContentProvider::for_value(&emoji_clone.to_value()))
            });
            drag_source.connect_drag_end(move |_, _, _| {
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

pub fn finde_begriffe(emoji: &str, daten: &HashMap<String, (Vec<Symbol>, Rc<Grid>)>) -> Vec<String> {
    for (_name, (symbole, _)) in daten.iter() {
        for symbol in symbole {
            if symbol.emoji == emoji {
                return symbol.begriffe.clone();
            }
        }
    }
    vec![]
}

fn lade_emojies(dateiname: &str) -> Vec<Symbol> {
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker");
    pfad.push(dateiname);
    let inhalt = fs::read_to_string(pfad)        // Datei (pfad) als String lesen
                        .unwrap_or_default();    // Wenn Datei fehlt oder fehlerhaft, ersetze durch leeren String "" - verhindert einen crash

    if dateiname == "history.list" {
        // History: Emojis zählen, sortieren, eindeutige Einträge erzeugen
        let mut zaehler: HashMap<String, (usize, Vec<String>)> = HashMap::new();

        for zeile in inhalt.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let mut teile = zeile.split_whitespace();
            if let Some(emoji) = teile.next() {
                let begriffe: Vec<String> = teile.map(|b| b.to_lowercase()).collect();
                let eintrag = zaehler.entry(emoji.to_string()).or_insert((0, begriffe.clone()));
                eintrag.0 += 1;
            }
        }
        let mut sortiert: Vec<_> = zaehler.into_iter().collect();
        sortiert.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // Nach Häufigkeit absteigend
        
        sortiert.into_iter()
            .take(30)                   // auf die 30 häufigsten Begrenzt
            .map(|(emoji, (_, begriffe))| Symbol { emoji, begriffe })
            .collect()
    } else {   
    //     // Normale .list-Dateien: klassisch einlesen
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

pub fn lade_ui_css() {
	let global_css = CssProvider::new();
    for pfad in [
        // "/usr/share/emoji-picker/emoji-picker.css",
        "./assets/usr/share/emoji-picker/emoji-picker.css" // nur für Entwicklung
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
    _kategorien: &[(impl AsRef<str>, impl AsRef<str>)],
    emojies_daten: &Rc<RefCell<HashMap<String, (Vec<Symbol>, Rc<Grid>)>>>,
) {
    let mut emojies_daten = emojies_daten.borrow_mut();

    if let Some((vec_emojies, grid)) = emojies_daten.get_mut("🕓") {
        vec_emojies.clear();            // leert das Emojie-Vec
        while let Some(kind) = grid.first_child() {
            grid.remove(&kind);         // entfernt Buttons aus dem Grid
        }
    }

    // Optional: history.list wirklich leeren
    let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    pfad.push("emoji-picker/history.list");
    let _ = std::fs::write(pfad, "");
}