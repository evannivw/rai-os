//! Separate boot images used to prove unknown SVCs never resume.
use core::arch::global_asm;
#[cfg(feature = "context-test-unknown-svc")]
global_asm!(
    ".section .text.context_negative, \"ax\"",
    ".global context_negative_trigger",
    ".global context_negative_resume",
    "context_negative_trigger:",
    "svc #0x5243",
    "context_negative_resume:",
    "ret",
);
#[cfg(feature = "context-test-wrong-site")]
global_asm!(
    ".section .text.context_negative, \"ax\"",
    ".global context_negative_trigger",
    ".global context_negative_resume",
    "context_negative_trigger:",
    "svc #0x5242",
    "context_negative_resume:",
    "ret",
);

pub fn run() {
    let name = if cfg!(feature = "context-test-unknown-svc") {
        "unknown-svc"
    } else {
        "wrong-site"
    };
    // First prove successful return, including clearing the nested-entry guard.
    assert!(super::check().is_ok());
    crate::uart::write_display(format_args!(
        "SELF-TEST: context {name} after valid returns\n"
    ));
    unsafe extern "C" {
        fn context_negative_trigger();
    }
    // SAFETY: isolated test image, valid EL1 stack/vectors. Assembly follows C ABI
    // and should reach the fatal handler. Returning is explicitly a test failure.
    unsafe { context_negative_trigger() };
    panic!("unexpected SVC resumed");
}
