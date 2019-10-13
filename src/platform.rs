#[cfg(not(unix))]
pub fn platform_init() {}

#[cfg(unix)]
pub fn platform_init() {
    reset_sigpipe_handler();
}

// XXX: it seems weird, but for years Rust team is unable to resolve
// the issue with SIGPIPE handling in the standard library. Once they
// fix this issue, this stuff MUST be removed.
//
// The default Rust's behavior is to ignore the signal, which results in
// "broken pipe" errors in case of using UNIX pipes and aborting the
// receiving side (e.g. less). To overcome this, we explicitly reset
// the SIGPIPE handler to the default one, which would make us behave
// as the usual UNIX program: die as soon as we receive SIGPIPE.
//
// Refrence: https://github.com/rust-lang/rust/issues/46016
#[cfg(unix)]
fn reset_sigpipe_handler() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
