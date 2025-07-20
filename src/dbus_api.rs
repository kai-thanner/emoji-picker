use dbus::blocking::{Connection, Proxy};
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use std::time::Duration;
use std::sync::mpsc::Sender;

// Prüft, ob schon ein Picker läuft.
// Ruft 'RaiseWindow' per D-Bus auf, wenn erreichbar. Sonst false.
pub fn pruefe_ob_picker_laeuft() -> bool {
    if let Ok(conn) = Connection::new_session() {
        let proxy = Proxy::new(
            "de.kai_thanner.emoji_picker",
            "/de/kai_thanner/emoji_picker",
            Duration::from_millis(500),
            &conn,
        );
        let res = proxy.method_call::<(), (), &str, &str>("de.kai_thanner.emoji_picker", "Quit", ());
        println!("Proxy-Call result: {:?}", res);
        res.is_ok()
    } else {
        println!("Could not connect to session bus!");
        false
    }
}

// Startet den D-Bus-Service.
pub fn starte_dbus_service(sender: Sender<&'static str>) {
    use dbus::channel::Sender as DbusSender;

    let conn = Connection::new_session().expect("D-Bus Session Connection failed");
    let result = conn.request_name("de.kai_thanner.emoji_picker", false, true, false);

    if let Err(e) = result {
        eprintln!("Unable to register D-Bus name: {e}");
        // Hier: Signalisiere dem Mainthread, dass er NICHT weiter machen soll
        std::process::exit(0);
    }

    let rule = MatchRule::new_method_call();

    conn.start_receive(rule, Box::new(move |msg, handler_conn| {
        println!("Got a D-Bus message: {:?}", msg);
        if let Some(method) = msg.member() {
            println!("Method member: {:?}", method);
            if method == "Quit".into() {
                println!("Quit called via D-Bus!");
                // Nur Signal "show_window" schicken
                let _ = sender.send("show_window");
                let reply = msg.method_return();
                if let Err(e) = DbusSender::send(handler_conn, reply) {
                    eprintln!("Failed to send D-Bus reply: {:?}", e);
                }
            }
        }
        true
    }));

    // Loop am Leben halten
    loop {
        conn.process(Duration::from_millis(1000)).unwrap();
    }
}
