# AN-066 — Plan de aislamiento concurrente y fallos de workers

## Objetivo

Demostrar que el prototipo `experimental-workers` conserva ADR-005 bajo
concurrencia: un worker reutiliza solo el hilo del host, mientras cada trabajo
crea y descarta su propio `Store` e instancia. La entrega amplía pruebas de
integración; no habilita la feature por defecto, no incorpora un pool de
instancias y no mide rendimiento de producto.

## Superficie y enfoque

La implementación se limita inicialmente a
`crates/runtime/tests/workers.rs` y a fixtures de prueba locales cuando hagan
falta. Se reutilizarán `WorkerPool`, `WorkerRequest`, `TenantConfig` y
`WorkerPoolStats`; cambios en `crates/runtime/src/workers.rs` solo se admitirán
si una prueba descubre una garantía que el contrato actual no puede expresar.

Las pruebas usarán límites finitos de fuel y esperas con deadline breve para
evitar bloqueos. No dependerán de orden de planificación del sistema operativo
ni de mediciones de tiempo para demostrar throughput.

## Matriz de aceptación

| Criterio | Prueba propuesta | Evidencia esperada |
| --- | --- | --- |
| AC-01 | Enviar solicitudes alternadas y desde dos hilos a un pool de dos workers contra fixtures con global mutable y memoria lineal. Alternar tenants con fuel, límite de memoria y policy distintos. | Cada lectura empieza en cero; el tenant restringido falla solo con su error tipado; el tenant permitido y el de límites holgados conservan sus resultados. |
| AC-02 | Saturar un pool de un worker y una cola pequeña con una invocación limitada por fuel. Cancelar una solicitud en cola y completar las admitidas restantes. | `QueueFull` contiene la capacidad configurada, el ticket cancelado recibe `Cancelled`, los trabajos admitidos terminan y los contadores concilian aceptación, rechazo, cancelación y finalización. |
| AC-03 | Ejecutar por separado trap, export inexistente, fuel agotado, límite de memoria, cancelación en cola y panic Rust controlado; después de cada caso enviar una solicitud sana de otro tenant. | El error queda en su ticket, la solicitud posterior termina y la capacidad vuelve a estar disponible. |
| AC-04 | Con una ejecución activa y trabajos pendientes, invocar `close(Drain)` y `close(RejectQueued)` desde un hilo de cierre. | `Drain` entrega resultados de la cola aceptada; `RejectQueued` entrega `Cancelled` solo a pendientes; ambos devuelven `running == 0` y cierran admisión. |
| AC-05 | Mantener `workers.rs` tras `#![cfg(feature = "experimental-workers")]` y validar el workspace con configuración predeterminada y `--all-features` en CI. | CI verde en Linux, Windows y macOS para formato, Clippy, tests y build en ambas configuraciones. |

## Detalle de fixtures

1. Ampliar el fixture Core WASM de `workers.rs` con escritura y lectura de
   memoria, además del global mutable existente. Cada trabajo debe comprobar
   que sus valores iniciales son cero.
2. Reutilizar los patrones de `warm_pool_contract.rs` y
   `tenant_lifecycle.rs` para un trap posterior a escritura, fuel insuficiente
   y límite de memoria. Esto evita inventar clasificaciones de `RuntimeError`.
3. Reutilizar una fixture con import del host ya cubierta por las pruebas de
   policy para contrastar tenants con política permitida y deny-by-default.
4. Usar la inyección privada de panic de AN-065 únicamente para comprobar la
   recuperación del worker; no convertirla en API pública.

## Secuencia de implementación

1. Crear helpers de fixture y espera acotada en `workers.rs` para evitar
   duplicar sondeo de `pool.stats().running`.
2. Añadir los escenarios de aislamiento alternado y concurrente para global,
   memoria, fuel, límites y policy.
3. Añadir la matriz de fallos y verificar una solicitud sana posterior a cada
   fallo.
4. Añadir las carreras de cierre desde un hilo y comprobar tickets, reporte y
   contadores.
5. Ejecutar formato, Clippy, tests predeterminados y con todas las features;
   confirmar la matriz completa en CI de tres sistemas operativos.

## Decisiones y límites

- El orden FIFO observado de `VecDeque` no se promoverá a garantía pública;
  AN-066 solo prueba que los trabajos admitidos no quedan silenciosamente sin
  resultado bajo una carga acotada.
- No se usan sleeps como mecanismo de sincronización. Los tests coordinan con
  barreras del host, sondeo de estado y deadlines explícitos.
- AN-067 sigue siendo la única fase que podrá afirmar mejoras de throughput,
  latencia, saturación o estabilidad prolongada.

## Criterios para detenerse

Pausar y revisar antes de cambiar el runtime si una prueba requiere compartir
host context, reutilizar `Store`/instancia, establecer una interrupción por
reloj o prometer equidad de scheduling que el contrato actual no declara.
