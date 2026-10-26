# Arquitectura del hito 1

Objetivo: kernel educativo AArch64 en QEMU, orientado a evolucionar hacia un
microkernel. Los módulos actuales comparten privilegios y memoria: NO son
procesos aislados. No hay scheduler, IPC entre procesos, MMU, heap ni userspace.

## Arranque mínimo

QEMU carga el ELF con `-kernel`. El linker coloca `_start` en `0x40080000`,
dentro de los 128 MiB de RAM de virt (RAM desde `0x40000000`), dejando espacio
al inicio para el DTB. El contrato soportado es `virt-9.2`, TCG, Cortex-A53,
una CPU, EL1, secure y virtualization desactivados. No se usa firmware.

`boot.S` enmascara interrupciones, verifica EL1, selecciona SP_EL1, prepara una
pila de 64 KiB alineada a 16 bytes, pone BSS a cero y llama `kernel_main`.
La pila está fuera del rango borrado de BSS. Un nivel de excepción inesperado
se detiene sin entrar a Rust (la prueba de arranque detectará timeout).

Rust usa `no_std` (sin servicios de un SO anfitrión), `no_main` (entrada propia)
y `core`. El destino `aarch64-unknown-none-softfloat` evita necesitar habilitar
SIMD/FPU para código generado por el compilador. No hay dependencias externas.
`rust-lld`, incluido en Rust, enlaza el ELF; Apple ld no es un linker ELF.
`build.rs` hace que cambiar el linker script fuerce un nuevo enlace.

## UART y seguridad

PL011 está en `0x09000000` para esta configuración de QEMU. Los registros se
leen/escriben con accesos volatile de 32 bits; volatile evita que el compilador
elimine MMIO, pero no provee sincronización. Se configura 115200, 8N1,
sin FIFO ni interrupciones. El driver espera los flags de disponibilidad.
La implementación es deliberadamente por sondeo: consume CPU mientras espera.
No es una demostración de eficiencia energética ni de rendimiento superior.

Cada bloque unsafe documenta sus condiciones. Los punteros MMIO solo son válidos
para esta máquina, con MMU apagada y acceso exclusivo desde una CPU. No debe
reutilizarse este driver directamente con concurrencia, otra placa o paginación.
Los símbolos del linker y el punto de entrada constituyen el contrato unsafe
con Assembly. No hay vectores de excepción: un fallo de CPU puede bloquear el
sistema; el panic de Rust imprime y se detiene, sin recuperación.

## Separación actual y dirección siguiente

`uart.rs` transporta bytes; `console.rs` delimita y valida líneas, maneja edición
y renderiza respuestas; `protocol.rs` interpreta mensajes estructurados;
`command.rs` interpreta comandos humanos y ejecuta un enum de operaciones
permitidas. Ambos intérpretes convergen en ese enum; no ejecutan texto como código.
Son módulos del mismo binario, sin dependencias genéricas de transporte.
Ver [protocolo](PROTOCOL.md). Un puente del Mac podrá transportar mensajes por la UART y llamar
a Meta Model API desde el host. El modelo propondrá operaciones; RAI-OS deberá
validarlas antes de ejecutarlas. No habrá credenciales ni HTTPS dentro del kernel.
El aislamiento real requerirá después memoria virtual, ejecución en EL0,
syscalls, planificación e IPC. Los nombres de módulos no sustituyen esas garantías.

## Referencias técnicas

- https://doc.rust-lang.org/stable/rustc/platform-support/aarch64-unknown-none.html
- https://www.qemu.org/docs/master/system/arm/virt.html
- https://github.com/qemu/qemu/blob/master/hw/arm/virt.c
- https://github.com/qemu/qemu/blob/master/hw/char/pl011.c
