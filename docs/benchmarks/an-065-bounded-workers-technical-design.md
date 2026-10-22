# AN-065 — Diseño técnico de workers host-side acotados

## Decisión y límite del prototipo

Este documento refina AN-065 para revisión humana. Propone un mecanismo
**interno y experimental** de concurrencia host-side; no cambia las rutas
productivas de `Engine`, `Component` o `Tenant`, no habilita una opción por
defecto y no mide ni promete una mejora de rendimiento.

El worker es reutilizable únicamente como hilo de ejecución. Cada trabajo crea
y destruye su propio `wasmtime::Store` e instancia mediante la ruta existente
`Tenant::invoke`. Esta es una consecuencia directa de ADR-005: no se conserva
una instancia viva, memoria lineal, tabla, global, recurso Canonical ABI, fuel,
límite ni host context para el siguiente trabajo.

El primer prototipo excluye intencionadamente `invoke_with_wasi` y las variantes
`*_with_host_ctx`. Ambos admiten recursos o fábricas de host que necesitan un
contrato propio de propiedad, envío entre hilos y limpieza. AN-065 solo admite
la invocación ordinaria de `Tenant::invoke`; ampliar el alcance requerirá un
Work Item y revisión de seguridad separados. Esta exclusión tampoco cambia el
estado no certificado de WASI.

## Superficie propuesta

La API viviría en un módulo experimental, por ejemplo
`agent_native_runtime::workers`, y no se conectaría a la ruta de invocación
existente. Los nombres son propuestos; se validarán con las convenciones del
crate durante la implementación.

```rust
pub struct WorkerPoolConfig {
    pub workers: usize,
    pub queue_capacity: usize,
}

pub struct WorkerPool { /* estado privado */ }

pub struct WorkerRequest {
    // Referencia a un artefacto ya cargado e inmutable; nunca a una instancia.
    component: Arc<Component>,
    tenant: Tenant,
    function: String,
    arguments: Vec<Val>,
}

pub struct WorkerTicket { /* id, estado y receptor privados */ }

pub enum SubmitError {
    QueueFull { queue_capacity: usize },
    Closed,
    UnsupportedRequest,
}

pub enum WorkerPoolInitError {
    Config(WorkerPoolConfigError),
    Spawn { worker_index: usize, workers: usize, source: std::io::Error },
}

pub enum WorkerError {
    Runtime(RuntimeError),
    Cancelled,
    WorkerPanicked,
    WorkerUnavailable,
    WorkerFaulted,
}

pub enum CancelError {
    AlreadyRunning,
    Finished,
}

pub enum ShutdownMode { Drain, RejectQueued }

impl WorkerPool {
    pub fn new(config: WorkerPoolConfig) -> Result<Self, WorkerPoolInitError>;
    pub fn submit(&self, request: WorkerRequest) -> Result<WorkerTicket, SubmitError>;
    pub fn stats(&self) -> WorkerPoolStats;
    pub fn close(self, mode: ShutdownMode) -> ShutdownReport;
}

impl WorkerTicket {
    pub fn try_cancel(&self) -> Result<(), CancelError>;
    pub fn recv(self) -> Result<InvocationReport, WorkerError>;
}
```

`WorkerRequest` toma posesión de sus argumentos y de su `Tenant`. El `Tenant`
contiene identidad, policy y límites inmutables por trabajo. El `Component` se
comparte solo si la implementación demuestra por compilación que es
`Send + Sync`; sigue siendo un artefacto parseado e inmutable. Si Wasmtime o la
versión soportada no cumplen esa condición para `Component` o `Val`, AN-065 no
introducirá conversiones `unsafe`: el prototipo se detendrá y documentará la
incompatibilidad antes de elegir otra representación.

No se entrega un `Resolver` al pool. El resolver actual contiene un `HashMap`
mutable y su contrato no es `Send + Sync`. La solicitud recibe un artefacto
pre-cargado; por tanto no se duplica ni se sincroniza implícitamente la caché
content-addressed. La carga y la política de caché siguen siendo responsabilidad
del llamador.

## Modelo de concurrencia y backpressure

`WorkerPool::new` crea exactamente `workers` hilos de sistema con
`thread::Builder::spawn`. Si el host rechaza la creación de un hilo, devuelve
un error tipado y detiene/espera los workers creados parcialmente. El estado
compartido será una cola `VecDeque` protegida por `Mutex` y una `Condvar`, no una
cola sin límite. Esta elección permite quitar un trabajo cancelado de la cola y
liberar su cupo inmediatamente.

1. `submit` toma el lock y solo acepta trabajo cuando el pool está en
   `Accepting`, `queued < queue_capacity` y el tenant declara un presupuesto
   finito de fuel.
2. Cuando la cola está llena devuelve inmediatamente
   `SubmitError::QueueFull`; no espera, no descarta silenciosamente y no crea
   un hilo adicional.
3. Un worker espera en la `Condvar`, extrae exactamente un trabajo y marca su
   ticket como `Running` antes de liberar el lock.
4. Ejecuta `tenant.invoke(&component, &function, &arguments)`. Esa ruta crea
   Store e instancia nuevas para el trabajo y las descarta al retornar.
5. Registra un resultado, error de runtime o panic tipado en el ticket, libera
   el slot y despierta a quien espere el cierre.

`WorkerPoolStats` solo expondrá contadores agregados (`queued`, `running`,
`accepted`, `rejected`, `cancelled`, `completed`, `failed`, `panicked`) y la
configuración. No incluirá IDs de tenant, nombres de funciones, argumentos ni
salidas.

## Cancelación, fallos y cierre

La cancelación es deliberadamente cooperativa y solo está garantizada antes de
que el trabajo comience:

- si el ticket sigue `Queued`, `try_cancel` lo elimina de la cola, responde
  `WorkerError::Cancelled` y libera capacidad;
- si ya está `Running`, devuelve `CancelError::AlreadyRunning` y no intenta
  terminar el hilo ni interrumpir Wasmtime de forma insegura;
- si terminó, devuelve `CancelError::Finished`.

Fuel, límites de memoria y límites de instancia son el mecanismo de contención
de una ejecución ya iniciada. En consecuencia, el prototipo rechaza antes de
encolar un `Tenant` sin presupuesto de fuel finito. Un deadline o interrupción
cooperativa de Wasmtime no se añade de forma implícita: necesitaría semántica de
reloj y pruebas propias.

Cada invocación se ejecuta dentro de `catch_unwind`; el bucle del worker también
contiene pánicos inesperados de mantenimiento. Una guarda RAII libera el slot
durante el desenrollado. Un panic de la invocación se comunica como
`WorkerError::WorkerPanicked`; un fallo inesperado de mantenimiento se comunica
como `WorkerError::WorkerFaulted`; y una desconexión del canal como
`WorkerError::WorkerUnavailable`. El trabajo no deja estado de guest persistente
y el hilo continúa atendiendo la siguiente solicitud. Un `RuntimeError` (trap,
fuel, memoria, policy o argumento inválido) se conserva dentro de
`WorkerError::Runtime`; no se transforma en éxito ni afecta a otro ticket.

`close(self, ShutdownMode::Drain)` pasa el pool a `Closing`, rechaza nuevos
trabajos y permite que termine la cola aceptada. `RejectQueued` responde
`Cancelled` a todo trabajo que aún no inició y espera solo los que ya están en
ejecución. Ambos modos realizan `join` de los hilos antes de devolver
`ShutdownReport`. No habrá un `Drop` que bloquee de manera sorprendente; el
prototipo exigirá cierre explícito en sus pruebas y documentación. El resultado
del cierre es determinista, pero no existe un máximo de tiempo garantizado: una
ejecución activa debe retornar mediante sus propios límites antes de su `join`.

## Aislamiento y precondiciones de implementación

Antes de escribir la ruta funcional, AN-065 debe demostrar en una prueba de
compilación que las fronteras del mensaje cumplen `Send + Sync` sin `unsafe`:
`Component`, `Val`, `Tenant`, `InvocationReport` y los errores que cruzan el
ticket. La prueba valida la representación, no una promesa teórica sobre
Wasmtime.

Por cada solicitud, la implementación deberá:

- usar exclusivamente `Tenant::invoke`, con un `Tenant` propiedad del trabajo;
- validar imports contra la policy del tenant antes de ejecutar, igual que la
  ruta actual;
- crear Store, linker e instancia durante esa invocación y descartarlos antes
  de que el worker tome otro trabajo;
- conservar los límites de fuel, memoria, tablas e instancias del tenant;
- no compartir `Resolver`, `Store`, `Instance`, `Linker` mutable, host context,
  WASI I/O ni estado mutable entre trabajos.

El objeto `Arc<Component>` no equivale a una caché nueva ni a un pool de
instancias: solo evita copiar un artefacto parseado que el llamador ya posee.
La policy efectiva siempre procede del `Tenant` de cada solicitud y vuelve a
validarse en su frontera.

## Pruebas que acompañarán el prototipo

AN-065 implementará solo el contrato mecánico y sus pruebas inmediatas:

| Caso | Evidencia esperada |
| --- | --- |
| Configuración inválida | cero workers o cero cola se rechazan tipadamente. |
| Ejecución no acotada | un tenant sin fuel finito se rechaza antes de ocupar capacidad. |
| Capacidad | con workers bloqueados y cola llena, el siguiente `submit` devuelve `QueueFull` sin bloquear. |
| Cancelación en cola | quita exactamente ese trabajo, entrega `Cancelled` y permite una nueva admisión. |
| Cancelación en ejecución | devuelve `AlreadyRunning`; fuel/límites siguen siendo la contención. |
| Cierre | `Drain` completa la cola; `RejectQueued` cancela solo las pendientes; ambos rechazan nuevos envíos. |
| Error y panic | un `RuntimeError`, un panic Rust controlado y un fallo de canal se clasifican sin perder el siguiente slot ni convertir el resultado en éxito. |
| Instancia fresca | dos invocaciones del fixture de global mutable devuelven el estado inicial en cada trabajo. |

AN-066 ampliará estas pruebas a tenants alternados y simultáneos, policy, fuel,
memoria, traps, saturación y cierres concurrentes en Linux, Windows y macOS.
AN-067 realizará throughput y soak; AN-065 no publicará cifras.

## Criterio para pasar a código

La implementación queda autorizable solo si se aprueba este diseño y se
mantienen estas decisiones: API experimental sin integración productiva,
trabajos ordinarios sin WASI/host context, rechazo inmediato ante saturación,
cancelación solo en cola y Store/instancia fresca por solicitud. Cualquier
cambio de esas fronteras exige una nueva revisión humana.
