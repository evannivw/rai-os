//! Presentation only: renders observed results, never fabricates telemetry.
use super::model::Reply;

pub fn header() {
    println!("\n=== RAI-OS | Host console ===");
    println!("Mac/Linux host interface; kernel runs inside QEMU.");
    println!("AI: not connected | Resource telemetry: not implemented");
    println!("Waiting for the actual kernel boot...");
}

pub fn connected() {
    println!("Connection: kernel boot and UART console confirmed.");
    println!("Tasks: task.start <text> | task.status | task.cancel <id>");
    println!(
        "Commands: help | info | uptime | mmu.info | context.check | timer.check | schedule.check | schedule.isolate | ipc.check | ipc.wait | record.count <key> | worker.start | worker.count <key> | worker.status | worker.stop | schedule.run | schedule.fault | user.run | user.spin | memory.info | memory.check | echo <ASCII text> | exit\n"
    );
}

pub fn reply(reply: &Reply) {
    let state = if reply.success { "OK" } else { "REJECTED" };
    println!("[kernel / {state}] {}", reply.payload);
}

pub fn input_error(error: &str) {
    println!("[input rejected; not sent] {error}");
}
