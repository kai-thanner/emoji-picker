// 	Internationalization - I18n

use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Deserialize, Clone, Debug)]
pub struct Sprache {
	pub title: String,
	pub settings_window: String,
	pub setup_done: String,
	pub setup_done_cinna: String,
	pub setup_fail: String,
	pub setup_fail_text: String,
	pub setup_not_available: String,
	pub set_desk_unknown: String,
	pub setup_exists: String,
	pub setup_done_xfce_gno: String,
	pub setup_fail_xfce_1: String,
	pub setup_fail_xfce_2: String,
	pub search_placeholder: String,
	pub set_key: String,
	pub set_key_tooltip: String,
	pub close_window_get: String,
	pub close_window_dnd: String,
	pub emoji_size: String,
	pub hist_reset: String,
	pub label_language: String,
	pub button_cancel: String,
	pub button_ok: String,

	pub debug_main_time_loading_language: String,
	pub debug_main_list_fail_to_copy: String,
	pub debug_main_list_copy_from_etc: String,
	pub debug_main_list_new_history: String,
	pub debug_main_time_css_loading: String,
	pub debug_main_settings_infofenster: String,
	pub debug_main_time_copy_from_etc: String,
	pub debug_main_time_emojis_load: String,
	pub debug_main_time_searchindex: String,
	pub debug_main_time_categorize_emoji: String,
	pub debug_main_time_to_searchfield: String,
	pub debug_main_time_set_window_keys: String,
	pub debug_main_time_info_window: String,
	pub debug_main_time_create_ui: String,

	pub debug_emoji_tabs_use_css: String,
	pub debug_emoji_tabs_css_failure: String,

	pub debug_shortcut_set_info_window: String,
	pub debug_shortcut_apply_gsettings_error: String,
	pub debug_shortcut_cinna_already_done: String,

	pub debug_gtk_theme_kde_gtk_theme: String,
	pub debug_gtk_theme_kde_gtk_fallback: String,
	pub debug_gtk_theme_kde_set_gtk_theme: String,
	pub debug_gtk_theme_kde_no_gtk_theme: String,
	pub debug_gtk_theme_kde_determinded_theme: String,
	pub debug_gtk_theme_kde_theme_folder: String,
	pub debug_gtk_theme_kde_darkmode_aktiv: String,
}

pub struct VerfuegbareSprache {
	pub code: String,		// z.B. "de"
	pub flagge: String,		// z.B. "🇩🇪"
	pub name: String,		// z.B. "Deutsch"
}

impl Sprache {
	fn pfad_sprachdateien() -> Vec<String> {
		let json_pfade = if cfg!(debug_assertions) {
			vec![
				"../assets/usr/share/emoji-picker/locale/".to_string(),	// start aus emoji-picker/src/
				"./assets/usr/share/emoji-picker/locale/".to_string(),	// start aus emoji-picker/
			]
		} else {
			vec!["/usr/share/emoji-picker/locale/".to_string()]			// start nach installation
		};
		json_pfade
	}

	// Liste verfügbarer Sprachcodes + Flaggen + Namen
	pub fn finde_verfuegbare_sprachen(debug: bool) -> Vec<VerfuegbareSprache> {
		let mut sprachen = vec![
			VerfuegbareSprache {
				code: "system".into(),
				flagge: "🌐".into(),
				name: "System".into(),
			}
		];

		let json_pfade = Self::pfad_sprachdateien();

		for pfad in json_pfade {
			let dateipfad = PathBuf::from(pfad.clone());
			if let Ok(entries) = fs::read_dir(&dateipfad) {
				if debug {
					println!("⏳ Loading from Path '{}'", pfad.clone());
				}
				for entry in entries.flatten() {
					let dateiname = entry.file_name().to_string_lossy().to_string();

					if let Some(code) = dateiname.strip_prefix("emoji-picker.").and_then(|s| s.strip_suffix(".json")) {
						if code == "system" { continue; } // vermeiden
						let flagge = Self::flagge_fuer_code(code);
						let name = Self::name_fuer_code(code);
						sprachen.push(VerfuegbareSprache {
							code: code.to_string(),
							flagge,
							name,
						});
					}
				}
			}
		}

		sprachen.sort_by(|a, b| {
			if a.code == "system" {
				std::cmp::Ordering::Less
			} else if b.code == "system" {
				std::cmp::Ordering::Greater
			} else {
				a.code.cmp(&b.code) // alphabetisch sortieren (außer "system")
			}
		});
		sprachen
	}

	fn flagge_fuer_code(code: &str) -> String {
		let teile: Vec<&str> = code.split(['-', '_']).collect();

	    // Wenn Sprach-Region vorhanden → nutze Region als Flaggen-Code
	    // Sonst fallback auf Sprachcode selbst
		let laendercode = if teile.len() >1 {
			teile[1].to_uppercase()
		} else {
			match teile[0] {
				"ar" => "SA".to_string(), // Arabisch → Saudi-Arabien
				"en" => "GB".to_string(), // Englisch → UK
				_    =>	teile[0].to_uppercase(), // fallback
			}
		};	

		if laendercode.len() == 2 {
			let chars: Vec<char> = laendercode.chars().collect();
			let base = 0x1F1E6;
			let f1 = char::from_u32(base + (chars[0] as u32 - 65)).unwrap_or('🏳');
			let f2 = char::from_u32(base + (chars[1] as u32 - 65)).unwrap_or('🏴');
			format!("{}{}", f1, f2)
		} else {
			"🌍".into()		// fallback für Codes wie "eo" oder falsch geschriebene
		}
	}

	fn name_fuer_code(code: &str) -> String {
		match code {
			"ar"	=> "العربية".into(),
			"da-DK"	=> "Dansk".into(),
			"de"    => "Deutsch".into(),
			"en-US" => "English (US)".into(),
			"en"	=> "English (UK)".into(),
			"es"    => "Español".into(),
			"fi"	=> "Suomi".into(),
			"fr"    => "Français".into(),
	        "it"    => "Italiano".into(),
	        "ja-JP" => "日本語".into(),
	        "nb-NO"	=> "Norsk Bokmål".into(),
	        "nl"	=> "Nederlands".into(),
	        "pl"    => "Polski".into(),
	        "pt-BR" => "Português (Brasil)".into(),
	        "pt"	=> "Português (Portugal)".into(),
	        "ru"    => "Русский".into(),
	        "sv"	=> "Svenska".into(),
	        "tr"    => "Türkçe".into(),
	        "uk-UA"	=> "українська".into(),
	        "zh-CN" => "中文".into(),
	        _       => format!("Sprache: {}", code),
		}.into()
	}

	pub fn lade_sprache(codes: &[impl AsRef<str>], debug: bool) -> Self {

		for code in codes {
			let code = code.as_ref();
			let dateiname = format!("emoji-picker.{}.json", code);
			let json_pfade = Self::pfad_sprachdateien();

			if debug {
				println!("📤 Loading language file '{}'", dateiname);
			}

			for pfad in json_pfade {
				let dateipfad = PathBuf::from(format!("{}{}", pfad, dateiname));
				if dateipfad.exists() {
					if let Ok(inhalt) = fs::read_to_string(&dateipfad) {
						if let Ok(parsed) = serde_json::from_str::<Sprache>(&inhalt) {
	                        if debug {
	                            println!("📂 Language file loaded from: {:?}\n🔀 Switching to '{}'", dateipfad, code);
	                        }
	                        return parsed;
	                    } else if debug {
	                    	println!("❌ Failed to parse language file: {:?}", dateipfad);
	                    }
					}
				}
			}

			if debug {
            	println!("🚫 Language file '{}' not found.", dateiname);
        	}
		}

	    // ⛑️ Fallback auf Englisch (fest definierter Pfad in assets)
	    let json_pfade = Self::pfad_sprachdateien();
	    for pfad in json_pfade {
		    let fallback = PathBuf::from(format!("{}emoji-picker.en.json", pfad));
		    if let Ok(backup) = fs::read_to_string(&fallback) {
		        if let Ok(parsed) = serde_json::from_str::<Sprache>(&backup) {
		            if debug {
		                println!("🚫 No language file found – using fallback (English UK)");
		            }
		            return parsed;
		        }
		    }
		}

	    panic!("🚫 No languagefile found – fallback failed.");
	}

	pub fn sprache_erkennen(code: &Option<String>, debug: bool) -> Self {

		match code {
			Some(inner) => Self::lade_sprache(&[inner], debug),
			None       => {
	            let mut pfad = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
	            pfad.push("emoji-picker/settings.ini");

	            if pfad.exists() {
	                if let Ok(content) = std::fs::read_to_string(&pfad) {
	                    for line in content.lines() {
	                        if line.trim().starts_with("sprache") {
	                            if let Some(value) = line.split('=').nth(1) {
	                                let code = value.trim();
	                                if debug {
	                                    println!("🌍 Detected language from settings.ini: {}", code);
	                                }
	                                if code != "system" {
	                                	return Self::lade_sprache(&[code], debug);	
	                                }
	                            }
	                        }
	                    }
	                }
	            }
	            // Systemsprache laden
			    let lang_env = std::env::var("LANG").unwrap_or_else(|_| "en".into());
			    
			    // lang_env → z.B. "de_DE.UTF-8" ➡️ "de_DE"
			    let lang_region = lang_env.split('.').next().unwrap_or("en");	// z.B. "de-AT"
			    let parts: Vec<&str> = lang_region.split(['_']).collect();
			    let lang = parts.get(0).unwrap_or(&"en");						// "de"
			    let region = parts.get(1);										// "AT"

			    let lang_code = if let Some(region) = region {
			    	vec![format!("{}-{}", lang, region), lang.to_string()]
			    } else {
			    	vec![lang.to_string()]
			    };

				if debug {
					println!("🌍 Detected system language: '{:?}'", lang_code);
				}

				Self::lade_sprache(&lang_code, debug)
			}
		}
	}
}