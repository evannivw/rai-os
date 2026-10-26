# RAI-OS — plan del hackathon

Fecha: 2026-10-26. Horarios locales: America/Guatemala.
Estado: plan propuesto; no se implementaron nuevas funciones al escribirlo.

## Fuentes y prioridad

- Participant Guide compartida íntegramente por el usuario en el chat el 26.
  Es la fuente de requisitos detallados de este plan; falta su enlace/versionado.
- Página oficial consultada el 26:
  https://dev.meta.ai/events/global-hackathon
- Página del organizador consultada el 26:
  https://cerebralvalley.ai/e/meta-ai-developer-hackathon

La guía señala kickoff el 26 y Building period del 27 de octubre al 4 de
noviembre. La página pública anuncia inicio el 26 y talleres desde el 27.
No asumir que la elegibilidad del trabajo del 26 está resuelta. Confirmar
con organizadores antes de presentar esa base como contribución elegible.
Entrega: 4 de noviembre, 21:00 ET/EST = 20:00 Guatemala. Meta interna propuesta:
entregar antes de las 18:00 Guatemala. Las fechas oficiales pueden cambiar.

## Propuesta y usuario

Tema propuesto: Push the Frontier. Categoría propuesta: Indie Developer si
participas individualmente; confirmar la categoría de tu registro.

RAI-OS será un prototipo de sistema operativo en Rust con una interfaz de agente
para inspeccionar su estado y ejecutar operaciones acotadas, con resultados
comprobables. Usuario inicial: desarrolladores que experimentan con kernels o
sistemas embebidos y necesitan diagnosticar y controlar un entorno limitado.
Esta utilidad y el potencial comercial son hipótesis a validar, no resultados.

Demo propuesta: el usuario pide comprobar el estado del sistema; un modelo Meta
consulta datos reales, selecciona una autoprueba permitida, recibe su resultado
y explica lo observado. Una solicitud no permitida se rechaza con un motivo
visible. Mostrar solicitud, operación propuesta, validación y resultado real.
El kernel no recibe código arbitrario del modelo.

## Requisitos de la guía y evidencia que prepararemos

| Requisito | Evidencia / acción |
| --- | --- |
| Incorporar modelos Meta | Llamada real desde el puente, modelo/proveedor identificados y resultado conectado a QEMU. Un mock no cuenta como integración final. |
| Repo público | Ya verificado público; publicar los cambios elegibles, comprobar acceso sin sesión. |
| Trabajo nuevo identificable | Commits con fechas reales, lista de archivos/funciones por periodo y sección de contribuciones. |
| Demo solo de lo creado durante el evento | Guion y video centrados en incrementos elegibles; declarar las bases previas. |
| Video de tres minutos | Grabar el 2–3 de noviembre, comprobar duración y enlace accesible. |
| Una entrega por equipo; máximo cuatro integrantes | Confirmar integrantes, categoría, representante y aceptación al evento; listar a todos. |
| Derechos sobre código/datos/assets | Mantener licencia y atribuciones; revisar las condiciones del modelo/proveedor elegido. |
| Presentación por plataforma CV | Comprobar campos y accesos antes del último día y conservar confirmación de envío. |

Preguntas de elegibilidad pendientes: la guía permite expresamente a startups
construir sobre código previo, pero no aclara aquí ese permiso para Indie.
Pregunta para #questions (no enviada): «Estoy inscrito en [categoría]. Tengo un
repositorio del 13 de octubre y un kernel mínimo desarrollado durante el kickoff
del 26. La guía fija Building period del 27 al 4. ¿Puedo usar esa base declarada
y presentar exclusivamente las funciones de agente desarrolladas del 27 al 4?
¿El trabajo del 26 es elegible?» Guardar la respuesta. No redatar ni borrar
historial para aparentar trabajo nuevo. Rehacer lo mismo no resuelve por sí solo
la elegibilidad del trabajo previo.

## Arquitectura propuesta

Usuario → puente Rust en el Mac → modelo Meta → propuesta estructurada →
validación del puente → UART → validación y ejecución en RAI-OS → resultado
→ explicación del modelo basada en ese resultado.

El puente es un programa normal del Mac: ahí viven HTTPS y la credencial en
variables de entorno. El kernel es el programa bare-metal dentro de QEMU.
Ambos deben validar operaciones y argumentos. Toda decisión del modelo debe
quedar limitada por código determinista. Errores de API y timeout se presentan
como errores, nunca como operaciones ejecutadas.

Operaciones propuestas para el incremento: consultar estado y contadores,
ejecutar una autoprueba acotada y consultar su resultado. Definir exactamente
qué comprueba la autoprueba antes de programarla; no inventar diagnósticos.
El objetivo mínimo es completar ese flujo real, no solo hacer echo del modelo.

Seguimos sin procesos aislados, scheduler, MMU, red del kernel, GUI, Android,
inferencia local ni garantías de seguridad o rendimiento de un SO de producción.
No ampliar a un framework de agentes ni intentar ganar todos los premios extra.

## Calendario propuesto y criterio de cierre

| Fecha | Trabajo | Se considera listo cuando… |
| --- | --- | --- |
| 26 oct | Reglas, alcance, procedencia; repaso guiado de Rust/Cargo y base existente | Sabes explicar compilador, Cargo, kernel, QEMU y UART; queda registrada la duda de elegibilidad. |
| 27 oct | Aclarar elegibilidad y congelar alcance; diseñar dos operaciones útiles | Operaciones, argumentos, resultados y límites están escritos; base previa identificada. |
| 28 oct | Puente Rust y operaciones del kernel | Un comando del host obtiene estado/autoprueba reales por UART con pruebas reproducibles. |
| 29 oct | Modelo Meta real conectado | Una petición en lenguaje natural completa una operación y devuelve un resultado observado. |
| 30–31 oct | Flujo de agente y validación | Rechaza operaciones inválidas; maneja timeout/desconexión; deja registro entendible del flujo. |
| 1 nov | Congelar funciones y estabilizar | Demo completa repetible; instrucciones probadas desde un checkout limpio. |
| 2 nov | Documentación, contribuciones y video de ensayo | Se entiende el problema, el papel de Meta y qué se construyó durante el evento. |
| 3 nov | Video final de tres minutos y revisión de entrega | Repo/enlaces accesibles; campos CV completos; dudas resueltas en el último día de soporte completo. |
| 4 nov | Verificación final y envío antes de las 18:00 Guatemala | Confirmación de entrega guardada, con dos horas de margen al límite oficial. |

Dependencias: acceso al modelo y aceptación al evento por confirmar. Haber
solicitado acceso a Meta no demuestra aceptación al hackathon. La API key no
se comparte en el chat ni se añade al repo. Elegir el modelo disponible tras
verificar acceso y documentación actuales. Si la API no está disponible, usar
un mock solo para desarrollar el transporte y resolver un proveedor elegible
antes de la demo; no etiquetar ese mock como integración Meta.

## Cómo distribuimos el esfuerzo según la guía

- Impacto, 30%: un usuario y una tarea concretos, con beneficio observable.
- Herramientas Meta, 25%: decisiones basadas en estado real y resultados; no un
  chatbot genérico ni una integración ornamental.
- Profundidad técnica, 20%: arranque reproducible, protocolo limitado, validación
  y pruebas de fallos. No promesas de aislamiento todavía inexistente.
- Demo, 25%: flujo completo, visible y repetible, con una solicitud rechazada.

## Aprendizaje y alcance inmediato

No es necesario volver a instalar Rust/Cargo ni reconstruir a ciegas el hito 1.
Ya está implementado y probado localmente; todavía falta revisarlo contigo.
Cada sesión: explicar una pieza, ejecutar un ejemplo pequeño, comprobar su
resultado y recién entonces pasar a la siguiente. El siguiente paso práctico
es compilar el ejemplo original de macOS y entender Cargo.toml y main.rs;
después veremos qué cambia al arrancar un kernel sin macOS debajo.
