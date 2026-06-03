#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|a| a == "--native-messaging") {
        keyvault_lib::run_native_messaging();
    } else {
        keyvault_lib::run();
    }
}
