use dbus::arg;
use dbus::blocking::Connection;
use std::time::Duration;
use clap::{command,arg, ArgAction};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum DbusAction {
    ActivateApp,
    ListApp,
}

fn get_str_property(conn : &Connection, name: &str, path: & str) -> Result<String>{
    let proxy = conn.with_proxy(name, path, Duration::from_millis(5000));
    use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
    let metadata: Box<dyn arg::RefArg> = proxy.get("org.kde.StatusNotifierItem", "Id")?;
    Ok(metadata.as_str().map(str::to_string).unwrap())
}

fn dbus_activate(conn: &Connection, name: &str, path: &str) -> Result<()>{
    println!("Query name {} with path {}", name, path);
    let proxy = conn.with_proxy(name, path, Duration::from_millis(5000));
    let _ = proxy.method_call("org.kde.StatusNotifierItem", "Activate", (0,0))?;
    Ok(())
}

fn exec(app: &str, action: DbusAction) -> Result<()> {
    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy("org.kde.StatusNotifierWatcher", "/StatusNotifierWatcher", Duration::from_millis(5000));
    
    use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
    let metadata: Box<dyn arg::RefArg> = proxy.get("org.kde.StatusNotifierWatcher", "RegisteredStatusNotifierItems")?;

    let mut iter = metadata.as_iter().unwrap();
    while let Some(key) = iter.next() {
        let key_str = key.as_str().unwrap();
        let items = key_str.split_once('/');
        if let Some((k, v)) = items {
            let mut str = "/".to_owned();
            str.push_str(v);
            let r = get_str_property(&conn, k, &str)?;
            match action {
                DbusAction::ListApp => println!("=> {}", r),
                DbusAction::ActivateApp => {
                    if &r == app{
                        dbus_activate(&conn, k, &str)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let matches = command!()
        .version("0.0.1")
        .arg(
            arg!([name] "application id")
                .required(false)
            )
        .arg(
            arg!(-l --list ... "List all application id on the tray")
            .action(ArgAction::SetTrue)
        )
        .get_matches();
    if matches.get_flag("list") {
        let _ = exec("", DbusAction::ListApp);
    }
    if let Some(name) = matches.get_one::<String>("name") {
        let _ = exec(name, DbusAction::ActivateApp);
    }
    Ok(())
}
