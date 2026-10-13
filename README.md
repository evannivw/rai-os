# RAI OS

Proyecto de código abierto para explorar la construcción de un sistema operativo
con Rust, mediante entregas pequeñas y educativas.

## Paso 1: compilar y ejecutar Rust

Por ahora tenemos un programa de consola que se ejecuta en macOS.
Todavía no arranca como un sistema operativo ni incluye un kernel o Android.
El objetivo de esta entrega es comprobar las herramientas y entender el ciclo
editar → compilar → ejecutar.

### Herramientas

- **Rust** es el lenguaje; `rustc` es su compilador.
- **Cargo** organiza el proyecto y permite compilar, ejecutar y probarlo.

Comprueba la instalación:

```sh
rustc --version
cargo --version
```

Si faltan, instala Rust con las instrucciones oficiales:
https://rust-lang.org/tools/install/
En un Mac con Homebrew también puedes usar `brew install rust`.

### Archivos

- `Cargo.toml`: nombre, versión y edición del lenguaje del paquete. La edición
  2024 selecciona las reglas del lenguaje; no es el número de versión de Rust.
  `[dependencies]` está vacío porque no necesitamos bibliotecas externas.
- `src/main.rs`: código del ejecutable. `fn main()` define la función donde
  empieza el programa. Las llaves delimitan su cuerpo y `println!` es una macro
  que imprime una línea; el punto y coma termina la instrucción.
- `Cargo.lock` (se generará al usar Cargo): registra las versiones de las
  dependencias para repetir la compilación. Se guardará en Git para este ejecutable.
- `.gitignore`: excluye `target/`, donde Cargo guarda los archivos generados.
- `README.md`: guía de esta primera entrega.
- `LICENSE`: licencia Apache 2.0 del repositorio.

### Ejecutar y verificar

Desde la carpeta `rai-os`:

```sh
cargo run
cargo check
cargo fmt --check
cargo test
```

La salida de `cargo run` debe ser:

```text
RAI OS: primer paso en Rust
```

`cargo check` comprueba el código, `cargo fmt --check` revisa el formato y
`cargo test` ejecuta las pruebas disponibles. Esta entrega no tiene pruebas
unitarias: solo imprime una línea, por lo que su salida se comprueba al ejecutarla.

Para practicar, cambia el mensaje en `src/main.rs` y vuelve a ejecutar
`cargo run`. Cargo recompila el código modificado.

Cada entrega se revisa, verifica y sube a GitHub con un commit pequeño.

### Estado de la verificación inicial

La máquina tiene Xcode y acceso SSH a GitHub. Rust y Cargo aún no están
instalados: las descargas desde los servidores oficiales y Homebrew fallaron
por interrupciones de conexión. Los comandos anteriores se intentaron, pero
no pudieron ejecutarse por falta de Cargo. Queda pendiente instalar las
herramientas, ejecutar las verificaciones y guardar el `Cargo.lock` generado.
