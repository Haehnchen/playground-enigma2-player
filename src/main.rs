fn main() {
    if std::env::var_os("GSK_RENDERER").is_none() {
        // SAFETY: Startup is still single-threaded; GTK and mpv are initialized below.
        unsafe { std::env::set_var("GSK_RENDERER", "gl") };
    }

    // SAFETY: Configure the process locale before GTK or mpv can start threads.
    // Preserve the user's locale, with the numeric category required by libmpv.
    unsafe {
        libc::setlocale(libc::LC_ALL, c"".as_ptr());
        libc::setlocale(libc::LC_NUMERIC, c"C".as_ptr());
    }
    gtk::disable_setlocale();
    std::process::exit(enigma2_player_core::app::run_from_env());
}
