# Protocolo UART v1

Un formato de líneas pequeño y explícito, sin JSON ni asignación dinámica.
Es transporte de solicitudes dentro de este prototipo, **no IPC entre procesos**.
El shell y el puente comparten UART0; debe haber un solo escritor/lector del host.
No hay autenticación, negociación, concurrencia ni deduplicación de IDs.

## Formato

En los ejemplos, `\t` representa un byte TAB y `\n` un byte LF reales:

```text
@rai1\t1\thelp\n
@rai1\t2\tinfo\n
@rai1\t3\techo\thola desde el host\n
```

Respuestas (siempre terminadas en CRLF):

```text
@rai1\t1\tok\thelp | info | echo <text>\r\n
@rai1\t3\tok\thola desde el host\r\n
@rai1\t4\terr\tunknown_operation\r\n
```

Gramática: `@rai1 TAB id TAB operacion [TAB argumento] fin-de-linea`.
`help` e `info` exigen exactamente tres campos; `echo` exige cuatro, con
argumento posiblemente vacío. Los nombres distinguen mayúsculas/minúsculas.
El ID tiene de 1 a 10 dígitos ASCII, valor 1..4294967295; se aceptan ceros
iniciales y se preserva el texto del ID. El kernel no almacena IDs anteriores.

- Límite de solicitud: 256 bytes sin el terminador. No se trunca.
- Entrada: ASCII imprimible y TAB solo como separador de campos.
- El argumento de echo no admite TAB, CR, LF, NUL, escape ni Unicode.
- Terminación: LF, CR o CRLF. CRLF cuenta como un único terminador.
- Líneas que empiezan con `@` son mensajes y no se hacen eco en la consola.
- Las demás son comandos humanos: `help`, `info`, `echo` o `echo <texto>`.
  Se hace eco de sus teclas; Backspace/Delete editan una línea humana.
  No hay prompt para mantener las respuestas de máquina al inicio de línea.
- Una línea humana vacía no ejecuta nada. No se normalizan espacios.
- El host debe esperar `Console ready:` antes de enviar y procesar respuestas
  que comiencen con `@rai1` después del banner. Enviar una petición y esperar
  su respuesta antes de la siguiente es suficiente para este hito.

## Errores

`bad_version`, `bad_id`, `bad_frame`, `bad_args`, `unknown_operation`,
`invalid_byte`, `line_too_long` y `uart_error`. Los errores de parseo conservan
un ID válido; un ID inválido o un error de transporte usa ID `0`.
Primero se valida versión, después ID y después operación/argumentos.
No se ejecuta una operación que no haya superado todas las validaciones.

Tras un byte inválido, error UART o exceso de longitud, se descarta hasta el
siguiente terminador. La parte restante jamás se ejecuta como comando.
El buffer es fijo; no hay heap. No hay timeout de línea en el kernel:
un mensaje incompleto espera su terminador. El puente deberá imponer timeout
y enviar un terminador o reiniciar QEMU para resincronizar.

## Futuro puente Meta (no implementado)

El puente iniciará QEMU con `scripts/qemu.sh` y manejará stdin/stdout como hace
`scripts/verify.py`. La API key vivirá solo en el entorno del host. El puente
convertirá propuestas del modelo en una operación conocida, validará los campos
y las longitudes y serializará esta gramática. Nunca debe concatenar salida
arbitraria del modelo a la UART: un salto de línea inicia otra petición.
RAI-OS repetirá la validación antes de ejecutar. Actualmente solo existen
operaciones informativas, no ejecución de código ni acceso a archivos.

La interfaz no es una frontera de seguridad frente a un host malicioso.
Antes de añadir operaciones con efectos habrá que definir permisos, política,
modo exclusivo de protocolo y, después, aislamiento real de servicios.
