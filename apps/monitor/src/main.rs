//! Monitor: processi con CPU e memoria, ordinabili, aggiornati ogni 2 s; termina/forza uscita.

use std::cell::{OnceCell, RefCell};

use appkit_shell::{item, submenu};
use objc2::{
    DefinedClass, MainThreadOnly, define_class, rc::Retained, runtime::AnyObject, sel,
};
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSApplication, NSEventModifierFlags, NSFont,
    NSLayoutAttribute, NSMenuItem, NSMenuItemValidation, NSScrollView, NSSearchField, NSStackView,
    NSStackViewDistribution, NSTableColumn, NSTableView, NSTableViewDataSource, NSTextAlignment,
    NSTextFieldCell, NSUserInterfaceLayoutOrientation,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSIndexSet, NSInteger, NSObject, NSObjectProtocol, NSRunLoop,
    NSRunLoopCommonModes, NSSortDescriptor, NSString, NSTimer,
};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System};

// Solo CPU e memoria: niente disco ed eseguibile, per ridurre il costo di ogni aggiornamento.
fn refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing().with_cpu().with_memory()
}

#[derive(Clone)]
struct Row {
    pid: u32,
    name: String,
    cpu: f32,
    mem: u64,
}

const COLS: [(&str, &str, f64); 4] = [
    ("name", "Nome processo", 300.),
    ("cpu", "% CPU", 80.),
    ("mem", "Memoria", 100.),
    ("pid", "PID", 70.),
];

struct State {
    sys: System,
    all: Vec<Row>,
    rows: Vec<Row>,
    key: String,
    asc: bool,
    query: String,
}

#[derive(Default)]
struct Ivars {
    state: RefCell<Option<State>>,
    table: OnceCell<Retained<NSTableView>>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    struct Monitor;

    unsafe impl NSObjectProtocol for Monitor {}

    unsafe impl NSMenuItemValidation for Monitor {
        #[unsafe(method(validateMenuItem:))]
        fn validate_menu_item(&self, _item: &NSMenuItem) -> bool {
            self.selected_pid().is_some()
        }
    }

    unsafe impl NSTableViewDataSource for Monitor {
        #[unsafe(method(numberOfRowsInTableView:))]
        fn rows_count(&self, _t: &NSTableView) -> NSInteger {
            self.with(|s| s.rows.len() as NSInteger)
        }

        #[unsafe(method_id(tableView:objectValueForTableColumn:row:))]
        fn value(&self, _t: &NSTableView, col: Option<&NSTableColumn>, row: NSInteger) -> Option<Retained<AnyObject>> {
            self.value_for(col, row)
        }

        #[unsafe(method(tableView:sortDescriptorsDidChange:))]
        fn sort_changed(&self, t: &NSTableView, _old: &NSArray<NSSortDescriptor>) {
            if let Some(d) = t.sortDescriptors().firstObject() {
                self.with(|s| {
                    s.key = d.key().map(|k| k.to_string()).unwrap_or_default();
                    s.asc = d.ascending();
                });
                self.reload();
            }
        }
    }

    impl Monitor {
        #[unsafe(method(tick:))]
        fn tick(&self, _t: &NSTimer) {
            self.with(|s| { s.sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind()); });
            self.reload();
        }

        #[unsafe(method(quitProcess:))]
        fn quit_process(&self, _s: &NSMenuItem) { self.kill(false) }

        #[unsafe(method(forceQuitProcess:))]
        fn force_quit_process(&self, _s: &NSMenuItem) { self.kill(true) }

        #[unsafe(method(filterChanged:))]
        fn filter_changed(&self, field: &NSSearchField) {
            let q = field.stringValue().to_string().to_lowercase();
            self.with(|s| s.query = q);
            self.reload();
        }
    }
);

fn cell(r: &Row, id: &str) -> String {
    match id {
        "name" => r.name.clone(),
        "cpu" => format!("{:.1}", r.cpu),
        "mem" => bytes(r.mem),
        _ => r.pid.to_string(),
    }
}

/// Colonna numerica: cifre a larghezza fissa e allineamento a destra, così i valori si leggono in colonna.
fn numeric_cell(mtm: MainThreadMarker) -> Retained<NSTextFieldCell> {
    let cell = NSTextFieldCell::new(mtm);
    cell.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(13., 0.)));
    cell.setAlignment(NSTextAlignment::Right);
    cell
}

fn bytes(b: u64) -> String {
    let mb = b as f64 / 1_048_576.;
    if mb >= 1024. { format!("{:.2} GB", mb / 1024.) } else { format!("{mb:.1} MB") }
}

fn sort(rows: &mut [Row], key: &str, asc: bool) {
    rows.sort_by(|a, b| {
        let o = match key {
            "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            "mem" => a.mem.cmp(&b.mem),
            "pid" => a.pid.cmp(&b.pid),
            _ => a.cpu.total_cmp(&b.cpu),
        };
        if asc { o } else { o.reverse() }
    });
}

impl Monitor {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(Ivars::default());
        unsafe { objc2::msg_send![super(this), init] }
    }

    fn with<R>(&self, f: impl FnOnce(&mut State) -> R) -> R {
        f(self.ivars().state.borrow_mut().as_mut().expect("stato"))
    }

    fn value_for(&self, col: Option<&NSTableColumn>, row: NSInteger) -> Option<Retained<AnyObject>> {
        let id = col?.identifier().to_string();
        let text = self.with(|s| s.rows.get(row as usize).map(|r| cell(r, &id)))?;
        Some(Retained::into_super(Retained::into_super(NSString::from_str(&text))))
    }

    fn table(&self) -> &NSTableView {
        self.ivars().table.get().expect("tabella")
    }

    fn selected_pid(&self) -> Option<u32> {
        let i = self.table().selectedRow();
        (i >= 0).then(|| self.with(|s| s.rows.get(i as usize).map(|r| r.pid))).flatten()
    }

    fn reload(&self) {
        let sel = self.selected_pid();
        let (index, summary) = self.with(|s| {
            s.all = s.sys.processes().values().map(|p| Row {
                pid: p.pid().as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                cpu: p.cpu_usage(),
                mem: p.memory(),
            }).collect();
            sort(&mut s.all, &s.key, s.asc);
            // sysinfo misura ogni processo in % di un core: dividiamo per i core per una % di sistema.
            let cores = std::thread::available_parallelism().map_or(1, |n| n.get()) as f32;
            let cpu: f32 = s.all.iter().map(|r| r.cpu).sum::<f32>() / cores;
            let q = s.query.clone();
            s.rows = s.all.iter()
                .filter(|r| q.is_empty() || r.name.to_lowercase().contains(&q) || r.pid.to_string().contains(&q))
                .cloned()
                .collect();
            let index = sel.and_then(|pid| s.rows.iter().position(|r| r.pid == pid));
            let total = s.all.len();
            let text = if q.is_empty() {
                format!("{total} processi · CPU totale {cpu:.1}%")
            } else {
                format!("{} di {total} processi · CPU totale {cpu:.1}%", s.rows.len())
            };
            (index, text)
        });
        let t = self.table();
        t.reloadData();
        match index {
            Some(i) => t.selectRowIndexes_byExtendingSelection(&NSIndexSet::indexSetWithIndex(i), false),
            None => unsafe { t.deselectAll(None) },
        }
        if let Some(w) = t.window() { w.setSubtitle(&NSString::from_str(&summary)); }
    }

    fn kill(&self, force: bool) {
        let Some(pid) = self.selected_pid() else { return };
        let name = self.with(|s| s.sys.process(Pid::from_u32(pid)).map(|p| p.name().to_string_lossy().into_owned()));
        let Some(name) = name else { return };
        let alert = NSAlert::new(self.mtm());
        let verb = if force { "Forzare l'uscita da" } else { "Terminare" };
        alert.setMessageText(&NSString::from_str(&format!("{verb} \u{201c}{name}\u{201d}?")));
        alert.setInformativeText(&NSString::from_str("I dati non salvati del processo andranno persi."));
        alert.addButtonWithTitle(&NSString::from_str(if force { "Forza uscita" } else { "Termina" }));
        alert.addButtonWithTitle(&NSString::from_str("Annulla"));
        if alert.runModal() != NSAlertFirstButtonReturn { return }
        let sig = if force { Signal::Kill } else { Signal::Term };
        let ok = self.with(|s| s.sys.process(Pid::from_u32(pid)).and_then(|p| p.kill_with(sig)).unwrap_or(false));
        if !ok {
            let err = NSAlert::new(self.mtm());
            err.setMessageText(&NSString::from_str(&format!("Impossibile chiudere \u{201c}{name}\u{201d}")));
            err.setInformativeText(&NSString::from_str("Il processo appartiene a un altro utente o al sistema."));
            err.runModal();
        }
    }
}

fn main() {
    appkit_shell::run("Monitor", |mtm, window| {
        let mut sys = System::new();
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind());
        let monitor = Monitor::new(mtm);
        *monitor.ivars().state.borrow_mut() = Some(State { sys, all: vec![], rows: vec![], key: "cpu".into(), asc: false, query: String::new() });

        let table = NSTableView::new(mtm);
        for (id, title, width) in COLS {
            let id = NSString::from_str(id);
            let col = NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), &id);
            col.setTitle(&NSString::from_str(title));
            col.setWidth(width);
            col.setSortDescriptorPrototype(Some(&NSSortDescriptor::sortDescriptorWithKey_ascending(Some(&id), id.to_string() == "name")));
            if id.to_string() != "name" {
                unsafe { col.setDataCell(&numeric_cell(mtm)) };
                col.headerCell().setAlignment(NSTextAlignment::Right);
            }
            table.addTableColumn(&col);
        }
        table.setUsesAlternatingRowBackgroundColors(true);
        table.setRowHeight(20.);
        table.setAutosaveName(Some(&NSString::from_str("processi")));
        table.setAutosaveTableColumns(true);
        unsafe { table.setDataSource(Some(objc2::runtime::ProtocolObject::from_ref(&*monitor))) };
        monitor.ivars().table.set(table.clone()).ok();
        let cpu_desc = NSSortDescriptor::sortDescriptorWithKey_ascending(Some(&NSString::from_str("cpu")), false);
        table.setSortDescriptors(&NSArray::from_retained_slice(&[cpu_desc]));

        let scroll = NSScrollView::new(mtm);
        scroll.setHasVerticalScroller(true);
        scroll.setDocumentView(Some(&table));

        let search = NSSearchField::new(mtm);
        search.setPlaceholderString(Some(&NSString::from_str("Cerca per nome o PID")));
        search.setSendsSearchStringImmediately(true);
        unsafe {
            search.setTarget(Some(&monitor));
            search.setAction(Some(sel!(filterChanged:)));
        }

        let stack = NSStackView::new(mtm);
        stack.setOrientation(NSUserInterfaceLayoutOrientation::Vertical);
        stack.setAlignment(NSLayoutAttribute::Width);
        stack.setDistribution(NSStackViewDistribution::Fill);
        stack.addArrangedSubview(&search);
        stack.addArrangedSubview(&scroll);
        window.setContentView(Some(&stack));
        window.setTitle(&NSString::from_str("Monitor attività"));

        // Menu Processo: Termina (⌥⌘Q), Forza uscita (⌥⇧⌘Q)
        let quit = item(mtm, "Termina processo", Some(sel!(quitProcess:)), "q");
        quit.setKeyEquivalentModifierMask(NSEventModifierFlags::Command | NSEventModifierFlags::Option);
        let force = item(mtm, "Forza uscita", Some(sel!(forceQuitProcess:)), "Q");
        force.setKeyEquivalentModifierMask(NSEventModifierFlags::Command | NSEventModifierFlags::Option);
        unsafe {
            quit.setTarget(Some(&monitor));
            force.setTarget(Some(&monitor));
        }
        if let Some(bar) = NSApplication::sharedApplication(mtm).mainMenu() {
            bar.insertItem_atIndex(&submenu(mtm, "Processo", &[quit, force]), 2);
        }

        // Il timer trattiene `monitor` (target), quindi vive quanto l'app.
        let timer = unsafe {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(2.0, &monitor, sel!(tick:), None, true)
        };
        unsafe { NSRunLoop::currentRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes) };
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordina_e_formatta() {
        let r = |pid, name: &str, cpu, mem| Row { pid, name: name.into(), cpu, mem };
        let mut rows = vec![r(1, "b", 5., 10), r(2, "A", 50., 5), r(3, "c", 1., 20)];
        sort(&mut rows, "cpu", false);
        assert_eq!(rows.iter().map(|r| r.pid).collect::<Vec<_>>(), [2, 1, 3]);
        sort(&mut rows, "name", true);
        assert_eq!(rows[0].name, "A");
        assert_eq!(bytes(1_572_864), "1.5 MB");
        assert_eq!(bytes(3 << 30), "3.00 GB");
    }
}
