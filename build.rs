fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("extern/audio_permission_check.c")
            .compile("audio_permission_check");
    }
}
