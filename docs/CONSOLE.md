# Consola del host — primer incremento de interfaz

Es una interfaz de terminal escrita en Rust que se ejecuta en el Mac. Arranca
QEMU y comunica solicitudes estructuradas con el kernel mediante UART. No es
una interfaz gráfica dentro de RAI-OS ni un agente de IA. La IA y la telemetría
de recursos aparecen explícitamente como no implementadas.

## Ejecutar

Desde la raíz del repositorio:

```sh
. "$HOME/.cargo/env"
sh scripts/console.sh
```

El script compila el kernel AArch64, detecta el destino del compilador anfitrión
y compila/ejecuta el ejemplo control_console para ese destino. La aplicación del
host usa std; el kernel continúa no_std/no_main. No se añadieron dependencias.
Solo se ha probado macOS ARM64. El script selecciona automáticamente el destino
Linux si se usa allí, pero Linux todavía no está verificado.

Al conectar se consulta info al kernel real. Comandos: help, info, echo <texto
ASCII>, exit. exit o fin de entrada cierran QEMU y recogen su proceso hijo.
La consola original sigue disponible con cargo run --locked.

## Separación de responsabilidades

- examples/control_console/model.rs: operaciones y validación pura del protocolo;
  sin acceso a procesos, terminal o hardware. Rechaza inyección de separadores,
  exceso de longitud, IDs no correspondientes y controles en las respuestas.
- examples/control_console/qemu.rs: adaptador de proceso/QEMU/UART. Espera el
  banner y Console ready antes de declarar conexión. Timeout de 10 segundos;
  solo una solicitud pendiente. Error de transporte cierra la sesión, sin
  reintentos que pudieran duplicar operaciones.
- examples/control_console/view.rs: presentación de resultados observados.
- examples/control_console.rs: composición y coordinación del caso de uso.
- El kernel no depende de ninguno de estos archivos.

Clean Architecture se aplica a fronteras y dirección de dependencias. No se
introducen traits o paquetes vacíos solo para imitar un diagrama. Cuando exista
un segundo transporte o un caso de uso que lo requiera, se extraerá su interfaz.

## Verificación

```sh
cargo test --locked --example control_console --target aarch64-apple-darwin
cargo clippy --locked --example control_console --target aarch64-apple-darwin -- -D warnings
python3 scripts/verify_console.py
```

En otro host, sustituir aarch64-apple-darwin por el valor host de rustc -vV.
La prueba de integración verifica arranque real, info/echo, rechazo y recuperación
de una entrada con TAB, fallo por ELF inexistente y cierre por fin de entrada.
El registro legible se genera en target/verification/host-console.log.

## Límites y siguiente incremento

La pantalla es deliberadamente textual, sin redibujado ni controles gráficos.
La conexión se comprueba al arrancar y en cada intercambio; no hay monitor de
conexión continuo ni reconexión automática. El adaptador confía en el ejecutable
QEMU local y el kernel del proyecto; no es un sandbox para procesos host hostiles.
Los permisos verificables entre agentes, sus presupuestos y la integración Meta
siguen pendientes. No se infiere uso de CPU, GPU, batería o RAM desde la
configuración de QEMU. Los futuros presupuestos simulados deberán etiquetarse.

Siguiente paso propuesto: definir un evento y un estado pequeño del kernel,
mostrarlos en esta consola y explicar su recorrido. Después se incorporarán
trabajos acotados y sus límites; solo entonces decisiones del coordinador Meta.
