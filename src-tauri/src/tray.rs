use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Manager, Wry,
};

use envtools_application::use_case::disable_group::DisableGroupUseCase;
use envtools_application::use_case::enable_group::EnableGroupUseCase;
use envtools_domain::repository::GroupRepository;
use envtools_infrastructure::{FileStateWriter, TomlGroupRepository};

use crate::config_dir;

fn repo() -> TomlGroupRepository {
    TomlGroupRepository::new(config_dir().join("config.toml"))
}

fn writer() -> FileStateWriter {
    FileStateWriter::new(config_dir())
}

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(app)?;

    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("EnvTools - Environment Manager")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();

            if id == "quit" {
                app.exit(0);
                return;
            }
            if id == "show" {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                return;
            }

            if let Some(group_name) = id.strip_prefix("toggle:") {
                let r = repo();
                let w = writer();

                if let Ok(Some(group)) = r.find_by_name(group_name) {
                    if group.is_active() {
                        let uc = DisableGroupUseCase::new(&r, &w);
                        let _ = uc.execute(group_name);
                    } else {
                        let uc = EnableGroupUseCase::new(&r, &w);
                        let _ = uc.execute(group_name);
                    }
                }

                if let Ok(new_menu) = build_tray_menu(app) {
                    if let Some(tray) = app.tray_by_id("main") {
                        let _ = tray.set_menu(Some(new_menu));
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn build_tray_menu(app: &impl Manager<Wry>) -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
    let r = repo();
    let groups = r.find_all().unwrap_or_default();

    let menu = Menu::new(app)?;

    if groups.is_empty() {
        let empty_item = MenuItem::new(app, "No groups configured", false, None::<&str>)?;
        menu.append(&empty_item)?;
    } else {
        for group in &groups {
            let label = format!(
                "{} ({})",
                group.name(),
                if group.is_active() { "ON" } else { "OFF" }
            );
            let id = format!("toggle:{}", group.name());
            let item =
                CheckMenuItem::with_id(app, &id, &label, true, group.is_active(), None::<&str>)?;
            menu.append(&item)?;
        }
    }

    menu.append(&MenuItem::new(app, "", false, None::<&str>)?)?;

    let show_item = MenuItem::with_id(app, "show", "Open EnvTools", true, None::<&str>)?;
    menu.append(&show_item)?;

    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    menu.append(&quit_item)?;

    Ok(menu)
}
