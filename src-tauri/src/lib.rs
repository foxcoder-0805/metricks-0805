/// Точка входа приложения.
///
/// Плагины и команды здесь добавляются по мере необходимости: сейчас
/// оболочке не нужно ничего, кроме показа окна с веб-интерфейсом.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    configure_webkit();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение Metricks");
}

/// WebKitGTK на Wayland по умолчанию рисует через DMABUF-буферы. Композитор
/// (Hyprland) отвечает на них ошибкой протокола, и окно падает с
/// «Error 71 (Protocol error) dispatching to Wayland display».
/// Программный путь отрисовки работает стабильно, а для дашборда метрик
/// разница в производительности незаметна.
#[cfg(target_os = "linux")]
fn configure_webkit() {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
}

#[cfg(not(target_os = "linux"))]
fn configure_webkit() {}

