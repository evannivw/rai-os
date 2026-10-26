//! Presentation only: renders observed results, never fabricates telemetry.
use super::model::Reply;

pub fn header() {
    println!("\n=== RAI-OS | Consola del host ===");
    println!("Interfaz del Mac/Linux; kernel dentro de QEMU.");
    println!("IA: no conectada | Telemetria de recursos: no implementada");
    println!("Esperando arranque real del kernel...");
}

pub fn connected() {
    println!("Conexion: arranque y consola UART confirmados.");
    println!("Comandos: help | info | echo <texto ASCII> | exit\n");
}

pub fn reply(reply: &Reply) {
    let state = if reply.success { "OK" } else { "RECHAZADO" };
    println!("[kernel / {state}] {}", reply.payload);
}

pub fn input_error(error: &str) {
    println!("[entrada rechazada; no enviada] {error}");
}
