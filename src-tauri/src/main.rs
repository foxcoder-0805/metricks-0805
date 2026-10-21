// Windows: не показываем консольное окно в релизной сборке.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    metricks_lib::run();
}
