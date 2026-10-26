# Compilar y ejecutar RAI-OS

## Requisitos

Rustup, QEMU con `qemu-system-aarch64` y máquina `virt-9.2` (QEMU >= 9.2),
Python 3 para la prueba y una shell POSIX. En macOS: `brew install qemu`.
Instala Rustup siguiendo https://rust-lang.org/tools/install/ y carga el PATH:

```sh
. "$HOME/.cargo/env"
cd ~/Development/Rust/rai-os
rustup show
qemu-system-aarch64 --version
```

`rust-toolchain.toml` fija Rust 1.90.0, rustfmt, clippy y el destino
`aarch64-unknown-none-softfloat`. Rustup los descarga en el primer uso.
En otra máquina, clona el repo público, entra a su carpeta y usa los mismos pasos.
No se necesita una API key, Xcode para enlazar el kernel ni un cross-GCC.

## Comandos

```sh
cargo build --locked          # ELF bare-metal de depuración
cargo run --locked            # Compila y arranca QEMU, UART en la terminal
python3 scripts/verify.py     # Arranque, shell y protocolo: 28 comprobaciones
python3 scripts/verify.py --release # Lo mismo con el kernel optimizado
cargo fmt --check
cargo clippy --bin rai-os --locked -- -D warnings
cargo build --release --locked
```

El ejecutable está en `target/aarch64-unknown-none-softfloat/debug/rai-os`.
También puedes arrancarlo con:

```sh
sh scripts/qemu.sh target/aarch64-unknown-none-softfloat/debug/rai-os
```

El primer mensaje real debe ser:

```text
RAI-OS: boot OK | AArch64 EL1 | UART PL011
```

QEMU comparte su entrada/salida estándar con la UART, sin monitor multiplexado.
Ctrl-C termina QEMU (el script deja activadas las señales de la terminal).
No se usa `-nographic`, para evitar que el monitor se mezcle con los mensajes.
`-accel tcg` permite ejecutar la misma CPU emulada en hosts ARM64 o x86_64;
solo macOS ARM64 está verificado inicialmente. No cambiar a HVF sin revisar
el contrato de arranque. No montar discos ni añadir dispositivos de red.

## Shell y mensajes

Escribe `help`, `info` o `echo hola Rust` y pulsa Enter. El shell tiene eco local
y admite Backspace/Delete. Es ASCII: todavía no acepta tildes o emojis.
El formato para un programa del host está en [PROTOCOL.md](PROTOCOL.md).
Las pruebas verifican mensajes, campos inválidos, límites y recuperación.

## Ejemplo original de macOS

El programa anterior está conservado; en el Mac ARM64 se ejecuta con:

```sh
cargo run --example host_hello --target aarch64-apple-darwin
```

El README inicial describe ese programa histórico. Para el kernel, sigue esta guía.
`cargo test` no prueba un kernel sin SO: se usa la prueba de integración QEMU.

## Diagnóstico

- Si falta Cargo, carga `$HOME/.cargo/env` en la terminal.
- Si falla Rustup de Homebrew durante la instalación, se logró avanzar en esta
  sesión con el instalador oficial Rustup 1.28.2; no es un fallo del kernel.
- Si falta `virt-9.2`, actualiza QEMU.
- Si la prueba agota el tiempo, lee `target/verification/uart.log`; un archivo
  vacío puede indicar que el kernel no llegó a Rust. No significa que arrancó.
- No cambies las opciones secure/virtualization/CPU: actualmente se exige EL1.
- El panic de Rust detiene el kernel. Reinicia QEMU para continuar.
