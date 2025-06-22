use gtk::prelude::*;
use gtk::{Button, Grid, Stack};
use glib::source::idle_add_local;
use std::{cell::Cell, rc::Rc};

use crate::{emoji_tabs::Symbol, kopiere_und_schliesse, settings::Einstellungen};

pub fn verbinde_suchfeld(
    entry: &gtk::Entry,
    such_grid: Rc<Grid>,
    stack: Stack,
    such_index: Rc<Vec<Symbol>>,
    clipboard: Rc<gtk::gdk::Clipboard>,
    window: Rc<gtk::ApplicationWindow>,
    einstellungen: Rc<Einstellungen>,
) {
    let pending = Rc::new(Cell::new(false));

    entry.connect_changed({
        let entry = entry.clone();
        let pending = Rc::clone(&pending);
        let such_grid = Rc::clone(&such_grid);
        let such_index = Rc::clone(&such_index);
        let clipboard = Rc::clone(&clipboard);
        let window = Rc::clone(&window);
        let stack = stack.clone();
        let einstellungen = Rc::clone(&einstellungen);

        move |_| {
            if pending.get() {
                return;
            }

            pending.set(true);

            let entry = entry.clone();
            let such_grid = Rc::clone(&such_grid);
            let such_index = Rc::clone(&such_index);
            let clipboard = Rc::clone(&clipboard);
            let window = Rc::clone(&window);
            let stack = stack.clone();
            let pending = Rc::clone(&pending);
            let einstellungen = Rc::clone(&einstellungen);

            idle_add_local(move || {
                pending.set(false);

                let schliessen = einstellungen.fenster_schliessen.get();
                let emoji_size = einstellungen.emoji_size.get();

                let text = entry.text().to_string();
                let filter_text = text.trim().to_lowercase();

                if filter_text.is_empty() {
                    stack.set_visible_child_name("notebook");
                    return glib::ControlFlow::Break;
                }

                stack.set_visible_child_name("suche");

                // Vorherige Buttons entfernen
                let mut child = such_grid.first_child();
                while let Some(widget) = child {
                    child = widget.next_sibling();
                    such_grid.remove(&widget);
                }

                let filter_kompakt = filter_text.replace(' ', "");
                let filter_wörter: Vec<_> = filter_text
                    .split_whitespace()
                    .filter(|w| !w.is_empty())
                    .collect();

                let mut i = 0;
                for symbol in such_index.iter() {
                    let joined = symbol.begriffe.join("").to_lowercase();
                    let begriffe = symbol.begriffe.clone();
                    let begriffe_vec = symbol.begriffe.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>();

                    let kombis_fenster = if begriffe_vec.len() >= 2 {
                        Some(
                            (2..=begriffe_vec.len())
                                .flat_map(|n| begriffe_vec.windows(n).map(|w| w.join("")))
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    };

                    let passt = joined.contains(&filter_kompakt)
                        || filter_wörter.iter().all(|wort| symbol.begriffe.iter().any(|b| b.contains(wort)))
                        || kombis_fenster
                            .as_ref()
                            .map(|kombis| kombis.iter().any(|k| k.contains(&filter_kompakt)))
                            .unwrap_or(false);

                    if !passt {
                        continue; // ❌ Überspringen, wenn nicht passt
                    }

                    // ✅ Begrenzung NACH dem Check
                    if i >= 100 {
                        break; // maximal 100 Emojis anzeigen
                    }

                    let button = Button::with_label(&symbol.emoji);
                    button.set_focusable(false);
                    button.set_size_request(emoji_size, emoji_size);
                    button.add_css_class("emoji");
                    button.set_hexpand(false);
                    button.set_halign(gtk::Align::Center);
                    button.set_tooltip_text(Some(&symbol.begriffe.join(", ")));

                    let emoji = symbol.emoji.clone();
                    let clipboard = Rc::clone(&clipboard);
                    let window = Rc::clone(&window);

                    button.connect_clicked(move |_| {
                        kopiere_und_schliesse(&emoji, &clipboard, &window, schliessen, &begriffe);
                    });

                    let row = i / 15;
                    let col = i % 15;
                    such_grid.attach(&button, col as i32, row as i32, 1, 1);
                    i += 1;
                }

                such_grid.show();
                glib::ControlFlow::Break
            });
        }
    });
}
