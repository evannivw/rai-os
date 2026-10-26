# Estado y procedencia del trabajo

Fechas expresadas en America/Guatemala (UTC−06:00). No se deben alterar fechas
ni atribuir trabajo anterior al periodo oficial del evento.

## Antes de esta sesión

- `46e681e`: 2026-10-13, commit inicial.
- `3e6d9c9`: 2026-10-13, programa de consola Rust, README y licencia.
- Inicio de esta sesión: árbol Git limpio, remoto SSH
  `git@github.com:evannivw/rai-os.git`. Acceso local confirmado en Mac ARM64,
  Darwin 25.3.0. No se encontró AGENTS.md en el repo ni sus ancestros.
- El contenido del README original y la licencia se conservan; se añadió una
  nota inicial que enlaza las instrucciones actuales. El programa original se conserva
  en `examples/host_hello.rs`.

## 2026-10-26 — kickoff / preparación

Se comenzó la base bare-metal AArch64: entrada Assembly, linker, Rust sin std,
UART y verificación automatizada de arranque real. Los cambios de esta sesión
están fechados el 26. Su elegibilidad sigue pendiente: la guía compartida después
fija Building period del 27 al 4, aunque el kickoff es el 26.

## Evento: contexto inicial (actualizado tras recibir la guía)

- Kickoff: 26 de octubre de 2026.
- Construcción: 27 de octubre al 4 de noviembre de 2026.
- Entrega indicada: 4 de noviembre de 2026, 20:00 Guatemala
  (5 de noviembre, 02:00 UTC).
- Hay una discrepancia entre comenzar hoy durante el kickoff y el inicio del
  periodo de construcción mañana. Mantener fechas reales y confirmar con la
  organización qué trabajo preparatorio es elegible.
- Repositorio público, identificar contribuciones del evento, integrar un
  modelo de Meta y presentar un video de 3 minutos.
- El usuario ya solicitó acceso a Meta. Este hito no necesita una API key.
- Visibilidad pública verificada el 26 mediante GitHub REST sin autenticación:
  `GET /repos/evannivw/rai-os` devolvió `private: false`. No se ha publicado código
  de esta sesión.
- Inicialmente no se había recibido la guía. Más tarde el 26, el usuario pegó
  la Participant Guide: confirma repo público, integración Meta, video de tres
  minutos y Building period 27-oct → 4-nov. Falta enlace/versionado y resolver
  la elegibilidad del trabajo del kickoff y del código previo en categoría Indie.

## Pendiente de fases posteriores

Puente host y Meta Model API, demo con una operación validada, aislamiento real
de servicios, video y revisión de requisitos de entrega. No hay IA integrada,
red, GUI, Android ni inferencia local en este hito.

## Resultado del hito 1 — 26 de octubre de 2026

Implementado localmente después de comprobar primero el arranque mínimo:
UART de entrada/salida, shell help/info/echo, separación de transporte,
interpretación y ejecución, mensajes estructurados v1 y rechazo de entradas
inválidas. Sin dependencias de crates, sin API keys, sin integración Meta todavía.

### Verificación ejecutada en este Mac

- Rust 1.90.0, Cargo 1.90.0; destino aarch64-unknown-none-softfloat.
- Enlace con rust-lld incluido; ELF estático AArch64 comprobado con `file`.
- QEMU 11.1.2, virt-9.2, Cortex-A53, TCG, 1 CPU, 128 MiB.
- Primer incremento: compilación y recepción del banner real antes de ampliar.
- Incremento completo: 28 comprobaciones de consola/protocolo pasaron tanto en
  debug como en release (56 intercambios/comprobaciones en total, más arranque).
- Incluyen comandos humanos, edición, mensajes válidos, versión e ID inválidos,
  argumentos incorrectos, operación no autorizada, NUL/escape/no ASCII, límite
  exacto de 256 bytes, exceso, recuperación, CRLF, mensajes consecutivos y fragmentados.
- `cargo fmt --check` y Clippy con warnings como errores pasaron.
- `cargo run --locked` probado en terminal: echo interactivo y salida con Ctrl-C.
- Ejemplo original de macOS compilado y ejecutado: `RAI OS: primer paso en Rust`.
- `git diff --check` sin errores.

Evidencias: [primer arranque](evidence/2026-10-26-first-boot.txt),
[consola debug](evidence/2026-10-26-debug-uart.txt),
[consola release](evidence/2026-10-26-release-uart.txt).
Las copias para revisión representan CR, TAB y Backspace como \r, \t y \b
literales; LF separa las líneas. Los registros de target/verification conservan
los bytes UART originales. No se han alterado los resultados.

No se probaron hardware físico, Linux/Windows como host, errores eléctricos de
UART (paridad/overrun), excepciones de CPU ni aislamiento de procesos.
No hay pruebas de rendimiento ni afirmaciones de superioridad sobre otros SO.

### Herramientas y continuidad

Se instalaron QEMU y Rustup con Homebrew y sus dependencias. El Rustup de Homebrew
1.29.2 falló con SIGSEGV al instalar componentes; se usó el instalador oficial
1.28.2. El compilador y los componentes quedaron instalados y verificados; el
intento posterior de autoactualización de Rustup falló, sin impedir las pruebas.
No se modificaron archivos de configuración de la shell. Cargar
`. "$HOME/.cargo/env"` al abrir otra terminal.

Al terminar la implementación inicial, los cambios estaban solo en el árbol
local, sin commit ni push. Véase la actualización de publicación más abajo.
Para continuar desde otra computadora, primero hay que guardar estos archivos
en Git y publicarlos, manteniendo su fecha real del 26 y el historial del 13.
No incluir target/, .env ni credenciales. No atribuir estas contribuciones al
periodo de construcción del 27 sin confirmar la regla con la organización.

### SHA-256 de los ELF comprobados

- debug: `a483404dfd1f6b4f341efc369c8fcbce34e7ffc82f68c8b785b55b85ccea4a7c`
- release: `ab4cc476869087391f2a87e52cc39b4d4b228f2fd2f3d7cc22268e9a825b76fd`

## Plan y revisión de reglas — 26 de octubre

Se creó [HACKATHON_PLAN.md](HACKATHON_PLAN.md) a partir de la Participant Guide
compartida por el usuario y las páginas públicas oficiales. Estas anuncian
inicio el 26 y talleres desde el 27; la guía sí distingue Building period del
27 al 4. Conservar esta discrepancia hasta obtener aclaración de los organizadores.
No se enviaron mensajes a Discord ni se aceptaron términos en nombre del usuario.
Este paso modifica documentación, no implementa la fase de agentes.

## Primera publicación autorizada — 26 de octubre

El usuario solicitó reglas permanentes en AGENTS.md, commits por incremento y
push frecuente. Las reglas quedaron publicadas en `75b69a5`. Se prepara ahora
un único snapshot del hito 1 acumulado en esta conversación, sin simular una
secuencia de commits anteriores. La fecha sigue siendo la real del 26.

Antes de guardar este snapshot se repitieron cargo fmt --check, Clippy con
-D warnings y verificación QEMU debug/release: 28 comprobaciones en cada perfil,
más arranque. Todas pasaron. Se revisaron los archivos incluidos y patrones
comunes de credenciales; no se encontraron coincidencias. Esto no reemplaza
una auditoría de seguridad. La elegibilidad del trabajo sigue pendiente.

## Consola del host — 26 de octubre

El snapshot del kernel `3b846d7` fue publicado en origin/main, después del commit
de reglas `75b69a5`. El usuario autorizó commits y pushes por incremento.
Se añadió control_console como ejecutable Rust del host, sin cambiar el kernel:
coordinación, validación pura, adaptador QEMU y presentación separados.

Verificaciones ejecutadas: dos pruebas unitarias de validación; Clippy sin
warnings; integración con QEMU real para info/echo, entrada inválida y posterior
recuperación, ELF inexistente y fin de entrada. Solo macOS ARM64 verificado.
La pantalla informa que la IA y la telemetría aún no están implementadas.
No hay procesos de agentes aislados, medidas de batería ni interfaz gráfica.
La documentación de consola explica uso, arquitectura y límites.
