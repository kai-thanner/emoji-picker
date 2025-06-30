use std::{
    rc::Rc,
    process::Command,
    fs,
    path::Path,
};

use crate::shortcut::{detect_desktop, Desktop};
use crate::i18n::Sprache;

pub fn pruefe_und_setze_gtk_theme_fuer_kde(sprachpaket: Rc<Sprache>, debug: bool) {
    if !matches!(detect_desktop(), Desktop::Kde) {
        return;
    }

    if let Some(theme_basis) = ermittle_kde_theme(Rc::clone(&sprachpaket), debug) {
        let gtk_theme = finde_kde_gtk_theme_schreibweise(&theme_basis, Rc::clone(&sprachpaket), debug);

        let theme_name = match gtk_theme {
            Some(ref korrekt)   => {
                if debug {
                    println!("✅ {}: {}",sprachpaket.debug_gtk_theme_kde_gtk_theme, korrekt);
                }
                korrekt.clone()
            }
            None                => {
                // Fallback
                let is_dark = kde_ist_dark_mode(&theme_basis, Rc::clone(&sprachpaket), debug);
                let fallback = if is_dark { "Breeze-Dark" } else { "Breeze" };

                if debug {
                    println!("❗️'{}' {} '{}'", theme_basis, sprachpaket.debug_gtk_theme_kde_gtk_fallback, fallback);
                }
                fallback.to_string()
            }
        };

        // Funktion seit Rust 1.77 unsafe. Hier unbedenktlich da nicht Nebenläufig genutzt
        unsafe {
            std::env::set_var("GTK_THEME", &theme_name);
        }

        if debug {
            println!("🎨 {}: {}", sprachpaket.debug_gtk_theme_kde_set_gtk_theme, theme_name);
        }

    } else if debug {
        println!("🚫 {}.", sprachpaket.debug_gtk_theme_kde_no_gtk_theme);
    }
}

fn ermittle_kde_theme(sprachpaket: Rc<Sprache>, debug: bool) -> Option<String> {
    let ausgabe = Command::new("kreadconfig5")
        .args(&["--group", "Icons", "--key", "Theme"])
        .output()
        .ok()?;

    let theme = String::from_utf8_lossy(&ausgabe.stdout).trim().to_string();
    
    if debug {
        println!("{} >> {}", sprachpaket.debug_gtk_theme_kde_determinded_theme, theme);
    }
    
    if !theme.is_empty() {
        Some(theme)
    } else {
        None
    }
}

fn finde_kde_gtk_theme_schreibweise(basisname: &str, sprachpaket: Rc<Sprache>, debug: bool) -> Option<String> {
    let theme_dir = "/usr/share/themes";
    let dirs = fs::read_dir(theme_dir).ok()?;

    for dir in dirs.flatten() {
        let name = dir.file_name().to_string_lossy().to_string();

        if debug {
            println!("{} >> {}", sprachpaket.debug_gtk_theme_kde_theme_folder, name);
        }

        if name.to_lowercase() == basisname.to_lowercase() {
            // z.B. breeze-dark -> Breeze-Dark
            let theme_path = format!("{theme_dir}/{}/gtk-4.0", name);

            if Path::new(&theme_path).exists() {
                return Some(name); // das korrekt geschriebene Theme
            }
        }
    }
    None
}

fn kde_ist_dark_mode(theme_name: &str, sprachpaket: Rc<Sprache>, debug: bool) -> bool {
    let lower = theme_name.to_lowercase();

    if debug {
        println!("{} >> {}", sprachpaket.debug_gtk_theme_kde_darkmode_aktiv, theme_name);
    }

    lower.contains("dark") || lower.contains("night") || lower.contains("noir") || lower.contains("dunkel") || lower.contains("nacht") || lower.contains("schwarz")
}