//! Guscio AppKit comune: applicazione, menu standard, finestra principale.
//! Dark Mode, VoiceOver e scorciatoie standard arrivano gratis da AppKit.

use objc2::{MainThreadOnly, rc::Retained, runtime::Sel, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSMenu, NSMenuItem,
    NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

/// Avvia l'app: `build` riceve la finestra principale e ne riempie il contenuto.
pub fn run(name: &str, build: impl FnOnce(MainThreadMarker, &NSWindow)) -> ! {
    let mtm = MainThreadMarker::new().expect("run() va chiamato dal main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    app.setMainMenu(Some(&main_menu(mtm, name)));

    let window = main_window(mtm, name);
    build(mtm, &window);
    window.makeKeyAndOrderFront(None);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
    app.run();
    std::process::exit(0)
}

fn main_window(mtm: MainThreadMarker, title: &str) -> Retained<NSWindow> {
    let style = NSWindowStyleMask::Titled
        | NSWindowStyleMask::Closable
        | NSWindowStyleMask::Miniaturizable
        | NSWindowStyleMask::Resizable;
    let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(800., 500.));
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            rect,
            style,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    unsafe { window.setReleasedWhenClosed(false) };
    window.setTitle(&NSString::from_str(title));
    window.setFrameAutosaveName(&NSString::from_str("main"));
    window.center();
    window
}

pub fn item(mtm: MainThreadMarker, title: &str, action: Option<Sel>, key: &str) -> Retained<NSMenuItem> {
    unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &NSString::from_str(title),
            action,
            &NSString::from_str(key),
        )
    }
}

pub fn submenu(mtm: MainThreadMarker, title: &str, items: &[Retained<NSMenuItem>]) -> Retained<NSMenuItem> {
    let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &NSString::from_str(title));
    for i in items {
        menu.addItem(i);
    }
    let top = item(mtm, title, None, "");
    top.setSubmenu(Some(&menu));
    top
}

fn main_menu(mtm: MainThreadMarker, name: &str) -> Retained<NSMenu> {
    let sep = || NSMenuItem::separatorItem(mtm);
    let bar = NSMenu::new(mtm);
    let app_menu = submenu(mtm, name, &[
        item(mtm, &format!("Informazioni su {name}"), Some(sel!(orderFrontStandardAboutPanel:)), ""),
        sep(),
        item(mtm, &format!("Nascondi {name}"), Some(sel!(hide:)), "h"),
        item(mtm, "Mostra tutto", Some(sel!(unhideAllApplications:)), ""),
        sep(),
        item(mtm, &format!("Esci da {name}"), Some(sel!(terminate:)), "q"),
    ]);
    let edit = submenu(mtm, "Modifica", &[
        item(mtm, "Annulla", Some(sel!(undo:)), "z"),
        item(mtm, "Ripeti", Some(sel!(redo:)), "Z"),
        sep(),
        item(mtm, "Taglia", Some(sel!(cut:)), "x"),
        item(mtm, "Copia", Some(sel!(copy:)), "c"),
        item(mtm, "Incolla", Some(sel!(paste:)), "v"),
        item(mtm, "Seleziona tutto", Some(sel!(selectAll:)), "a"),
    ]);
    let window = submenu(mtm, "Finestra", &[
        item(mtm, "Riduci", Some(sel!(performMiniaturize:)), "m"),
        item(mtm, "Ridimensiona", Some(sel!(performZoom:)), ""),
        item(mtm, "Chiudi", Some(sel!(performClose:)), "w"),
    ]);
    bar.addItem(&app_menu);
    bar.addItem(&edit);
    bar.addItem(&window);
    bar
}
