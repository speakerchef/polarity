fn main() {
    cc::Build::new()
        .file("extern/audio_permission_check.c")
        .compile("audio_permission_check");
}
