#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    zecho_lib::macos_panel::swizzle_activation_policy();
    zecho_lib::run()
}
