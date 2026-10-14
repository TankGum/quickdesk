// Privacy page in Spanish. Keep it in step with src/data/privacy.ts.
const privacy = {
  title: "Privacidad",
  updated: "Última actualización: 14 de octubre de 2026 (imágenes en las notas)",
  html: `<p class="lead">QuickDesk no tiene cuentas, ni analítica, ni telemetría. Tus notas y tu historial del portapapeles viven en tu ordenador. Esta página enumera todo lo que sale de él.</p>
<h2>Lo que se queda en tu ordenador</h2>
<ul>
<li><strong>Notas, historial del portapapeles (texto, imágenes, listas de archivos), configuración y registros</strong>, en <code>~/.local/share/click.quickdesk</code>.</li>
<li><strong>El historial del portapapeles nunca sale de tu ordenador</strong>, ni siquiera con la sincronización activada. Lo que los gestores de contraseñas marcan como secreto no se guarda, y puedes pausar el guardado cuando quieras.</li>
<li>Los <strong>puertos</strong> se leen de <code>/proc</code> y de Docker en tu máquina y no se guardan.</li>
<li>Tu código de sincronización y tu clave de cifrado se guardan en el llavero del sistema.</li>
</ul>
<h2>Qué envía QuickDesk, y adónde</h2>
<table>
<tr><th>Cuándo</th><th>Adónde</th><th>Qué</th></tr>
<tr><td data-label="Cuándo">Al buscar actualizaciones (30 s después de arrancar y luego cada 6 horas; se puede desactivar)</td><td data-label="Adónde"><code>dl.quickdesk.click</code> (Cloudflare)</td><td data-label="Qué">Una petición de <code>update.json</code>, y el paquete cuando eliges actualizar. Tu dirección IP es visible para Cloudflare, como en cualquier descarga.</td></tr>
<tr><td data-label="Cuándo">Sincronización de notas, solo si la activas</td><td data-label="Adónde">QuickDesk Cloud, <code>sync.quickdesk.click</code> (Cloudflare)</td><td data-label="Qué">Tus notas y las imágenes que contienen, cifradas en tu ordenador (XChaCha20-Poly1305) con una clave derivada de tu frase de contraseña; QuickDesk Cloud no puede leerlas. Las guarda bajo un identificador aleatorio de cuenta de sincronización, con su tamaño y la hora de subida, y ve tu dirección IP. Ni correo, ni nombre, ni otros datos personales. Puedes borrarlo todo desde Ajustes → Sincronización.</td></tr>
<tr><td data-label="Cuándo">Uso de IA, solo si Claude Code tiene la sesión iniciada en este equipo (en un Mac, solo después de que permitas leer su inicio de sesión del Llavero)</td><td data-label="Adónde"><code>api.anthropic.com</code></td><td data-label="Qué">Una petición de uso autorizada con el inicio de sesión que ya tiene Claude Code, como mucho cada 5 minutos. El token se lee; nunca se guarda, se renueva ni se envía a ningún otro sitio.</td></tr>
<tr><td data-label="Cuándo">Pestaña Runtimes, cuando busca versiones nuevas (como mucho una vez por hora), cuando abres una lista de versiones o instalas una</td><td data-label="Adónde">Los servidores de nvm, uv, rustup y mise: <code>nodejs.org</code>, <code>github.com</code> / <code>releases.astral.sh</code> (compilaciones de Python), <code>static.rust-lang.org</code> y, para mise, el sitio de descarga de cada lenguaje (por ejemplo <code>go.dev</code>)</td><td data-label="Qué">Esas herramientas hacen las peticiones ellas mismas, como cuando las ejecutas en una terminal. QuickDesk no les añade nada.</td></tr>
<tr><td data-label="Cuándo">Pestaña Runtimes, solo si haces clic en «Instalar mise»</td><td data-label="Adónde"><code>github.com</code> (versiones de jdx/mise); en Windows también <code>api.github.com</code></td><td data-label="Qué">Una descarga del último mise y su archivo de sumas de verificación. En Windows, QuickDesk pregunta antes a la API de GitHub cuál es la última versión (User-Agent «QuickDesk»).</td></tr>
<tr><td data-label="Cuándo">Pegado automático mediante el portal de escritorio</td><td data-label="Adónde">Tu propio escritorio (GNOME), no la red</td><td data-label="Qué">Una pulsación de Shift + Insert.</td></tr>
</table>
<p>El uso de Codex y Antigravity se lee de archivos de registro de tu ordenador; no se envía nada.</p>
<h2>Esta web</h2>
<p>quickdesk.click es un sitio estático en Cloudflare Pages, sin cookies, sin analítica y sin seguimiento. Carga fuentes de Google Fonts, así que Google ve tu dirección IP. La demo se ejecuta entera en tu navegador con datos de ejemplo. El sistema que eliges (Linux, Windows o macOS) y el tema claro u oscuro, si lo cambias, se recuerdan en el almacenamiento local de tu navegador y no se envían a ningún sitio.</p>
<h2>Recuento de descargas</h2>
<p>Para saber cuántas personas instalan y actualizan QuickDesk, <code>dl.quickdesk.click</code> cuenta cada descarga y cada búsqueda de actualizaciones: qué archivo (su versión y sistema), el tipo de cliente que lo pidió (un navegador que viene de esta web o de otra parte, una terminal, PowerShell o el actualizador de la app) y el país que Cloudflare indica para la petición. Solo se guardan totales diarios (en una base de datos Cloudflare D1): ni dirección IP, ni cadena del navegador, ni cookie, ni otro identificador, ni registro por petición, así que los recuentos no se pueden vincular a una persona ni a un ordenador.</p>
<h2>Preguntas</h2>
<p>QuickDesk es de código abierto: todo lo que se dice aquí se puede comprobar en el <a href="https://github.com/TankGum/quickdesk">código fuente</a>. Pregunta o informa de un problema en <a href="https://github.com/TankGum/quickdesk/issues">GitHub issues</a>.</p>`,
};

export default privacy;
