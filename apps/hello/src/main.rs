use objc2_app_kit::{NSFont, NSTextAlignment, NSTextField};
use objc2_foundation::NSString;

fn main() {
    appkit_shell::run("Hello", |mtm, window| {
        let label = NSTextField::labelWithString(&NSString::from_str("Ciao da Rust + AppKit"), mtm);
        label.setFont(Some(&NSFont::systemFontOfSize(28.)));
        label.setAlignment(NSTextAlignment::Center);
        window.setContentView(Some(&label));
    });
}
