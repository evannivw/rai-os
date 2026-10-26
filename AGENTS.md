# RAI-OS: instrucciones permanentes de colaboración

## Contexto

Kernel educativo Rust AArch64 en QEMU, orientado a microkernel a largo plazo.
Proyecto para Meta Global AI Developer Hackathon 2026. Leer docs/WORKLOG.md,
docs/HACKATHON_PLAN.md y docs/ARCHITECTURE.md antes de cambiar el alcance.
El usuario aprende Rust y sistemas: avanzar en incrementos pequeños.

## Explicación y alcance

- Antes de cada paso, explicar en español qué se ejecutará y para qué.
- Enseñar una idea por incremento, con ejemplos breves y precisos; evitar muros
  de texto y no adelantar varias fases sin acompañar el aprendizaje.
- Al terminar, indicar qué cambió, cómo se verificó y qué sigue pendiente.
- La interfaz inicial será una consola del host que refleje datos reales de
  QEMU. No presentarla como GUI nativa del kernel ni fabricar telemetría o IA.

## Git: regla solicitada por el usuario

- Cada incremento terminado y verificado debe quedar en un commit coherente.
- Publicar con push al remoto autorizado origin de manera frecuente, al cerrar
  cada incremento. El usuario ya autorizó estos commits y pushes; no pedir
  confirmación de nuevo para cambios propios de RAI-OS dentro de ese alcance.
- Revisar diff, archivos incluidos y pruebas antes de commit. Nunca añadir
  credenciales, target/ ni cambios ajenos. Usar rutas explícitas al añadir archivos.
- Mensajes con un título claro y cuerpo cuando corresponda: problema/cambio,
  motivo y verificaciones ejecutadas. No afirmar pruebas que no se hicieron.
- Comprobar el resultado del push y la coincidencia de HEAD con la rama remota.
  Si falla, informar el estado exacto y resolver sin force-push ni borrar historia.
- Conservar fechas reales. No dividir trabajo ya hecho para aparentar una
  cronología de desarrollo inexistente; identificar snapshots acumulados.

## Clean Architecture, proporcional al proyecto

- Reglas y operaciones de dominio independientes de UART, QEMU, terminal y API Meta.
- Separar interpretación/validación, coordinación de casos de uso y adaptadores
  de entrada/salida. Las dependencias van hacia las reglas de dominio.
- El kernel no depende de la interfaz del host ni de un proveedor de IA.
- La interfaz muestra resultados; no concentra reglas ni accede a MMIO.
- Un módulo tiene una responsabilidad clara. Introducir interfaces/traits solo
  cuando una frontera real lo necesite; evitar frameworks y capas vacías.
- Mantener unsafe pequeño y documentar sus precondiciones concretas.
- Los módulos actuales comparten memoria y privilegios: no son procesos aislados.

## Verificación y hackathon

- Para cambios del kernel/protocolo: formato, Clippy y prueba real QEMU relevante;
  verificar release cuando cambie código ejecutado o configuración de compilación.
- Para documentación: revisar coherencia, enlaces locales y git diff --check.
  No repetir pruebas sin cambios o motivos que lo justifiquen.
- Registrar versiones, resultados y limitaciones. Distinguir datos reales,
  simulación controlada, configuración y funciones todavía no implementadas.
- Repositorio público; identificar trabajo previo y nuevo; integración Meta real
  y video de tres minutos según Participant Guide proporcionada por el usuario.
- Kickoff 26-oct; la guía fija construcción 27-oct a 4-nov. La elegibilidad del
  trabajo del 26 y de la base previa para Indie sigue pendiente de aclaración.
- Entrega: 4-nov-2026, 20:00 Guatemala; meta interna 18:00. No prometer superioridad
  de rendimiento/seguridad ni elegibilidad sin evidencia.
