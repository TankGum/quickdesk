//! macOS permissions QuickDesk needs: Accessibility, so its ⌘V reaches the
//! app you were typing in (macOS drops synthetic key presses otherwise).

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::string::CFString;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
}

/// Whether the user allowed QuickDesk under Privacy & Security → Accessibility.
pub fn accessibility_trusted() -> bool {
    // SAFETY: a plain query without arguments.
    unsafe { AXIsProcessTrusted() }
}

/// Ask for the permission: macOS shows its own dialog, which opens the
/// Accessibility list with QuickDesk in it. Returns whether it is granted now.
pub fn request_accessibility() -> bool {
    // kAXTrustedCheckOptionPrompt
    let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), CFBoolean::true_value().as_CFType())]);
    // SAFETY: a valid CFDictionary that outlives the call.
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

/// Open System Settings at Privacy & Security → Accessibility.
pub fn open_accessibility_settings() -> std::io::Result<()> {
    std::process::Command::new("/usr/bin/open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .status()
        .map(drop)
}
