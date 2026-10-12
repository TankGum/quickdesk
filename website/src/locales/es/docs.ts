// Docs in Spanish, by section id (src/data/docs.ts).
import { macBeta } from "../../data/release";

const docs: Record<string, { title: string; html: string }> = {
  install: {
    title: "Instalación",
    html: `<p>La forma más rápida, en cualquier distribución:</p>
<pre><code>curl -fsSL https://quickdesk.click/install.sh | bash</code></pre>
<p>Elige el .deb, .rpm o AppImage para tu sistema, comprueba su SHA-256 y lo instala (<a href="/install.sh">leer el script</a>). O consigue un paquete en la <a href="/download">página de descarga</a> e instala el archivo descargado con:</p>
<ul>
<li><strong>Ubuntu, Debian, Mint, Pop!_OS:</strong> <code>sudo apt install ~/Downloads/QuickDesk_&lt;version&gt;_amd64.deb</code></li>
<li><strong>Fedora, openSUSE:</strong> <code>sudo dnf install ~/Downloads/QuickDesk-&lt;version&gt;-1.x86_64.rpm</code></li>
<li><strong>Cualquier otra:</strong> haz ejecutable el AppImage y ábrelo. Necesita FUSE 2 (<code>libfuse2</code>; en Ubuntu 24.04, <code>libfuse2t64</code>).</li>
</ul>
<p>QuickDesk necesita un sistema x86_64 de los últimos años (glibc 2.34 o posterior). Funciona mejor en GNOME, tanto con Wayland como con X11.</p>
<p><strong>Windows 10 y 11 (64 bits):</strong> en PowerShell, ejecuta <code>irm https://quickdesk.click/install.ps1 | iex</code>, o descarga el instalador desde la <a href="/download">página de descarga</a>. Se instala solo para tu usuario, sin permisos de administrador. El instalador aún no está firmado, así que Windows puede decir que protegió tu PC: elige <em>Más información → Ejecutar de todas formas</em>. En Windows los atajos son <kbd>Win</kbd> <kbd>Alt</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd>, QuickDesk vive en la bandeja del sistema y el pegado automático no necesita configuración. Allí la pestaña Runtimes funciona con mise, rustup y uv (consulta <em>Versiones de lenguajes</em>).</p>
<p>La primera vez, una pantalla de bienvenida muestra los atajos y ofrece dos opciones: <strong>iniciar al entrar en la sesión</strong> (necesario para que el historial del portapapeles y el anillo de IA funcionen siempre) y <strong>pegado instantáneo</strong> (consulta <a href="#auto-paste">Pegado automático</a>). Después, QuickDesk vive en la barra superior.</p>`,
  },
  macos: {
    title: "macOS",
    html: `${macBeta ? '<p class="note"><strong>Aún en desarrollo.</strong> La versión para Mac funciona, pero es nueva: todavía no se ha probado todo en todos los Mac, y el pegado automático y los atajos se siguen puliendo. Por favor, <a href="https://github.com/TankGum/quickdesk/issues">cuéntanos</a> lo que no funcione.</p>\n' : ""}<p>QuickDesk funciona en macOS 11 o posterior en Apple Silicon (M1 o posterior). En Terminal, ejecuta <code>curl -fsSL https://quickdesk.click/install.sh | bash</code>: coloca QuickDesk.app en Aplicaciones y lo abre. O descarga el <code>.dmg</code> desde la <a href="/download#macos">página de descarga</a> y arrastra QuickDesk a Aplicaciones.</p>
<p><strong>Abrirlo por primera vez.</strong> QuickDesk aún no está notarizado por Apple, así que, si el <code>.dmg</code> se descargó con el navegador, macOS detiene el primer arranque con el aviso de que «Apple no puede comprobar si contiene software malicioso». A la app no le pasa nada; haz una de estas dos cosas una vez (si lo instalaste con el comando, no aparece ningún aviso):</p>
<ol>
<li>Abre QuickDesk desde Aplicaciones y cierra el aviso de macOS. Luego abre Ajustes del Sistema → Privacidad y seguridad, baja hasta Seguridad, haz clic en <em>Abrir igualmente</em> junto a «Se ha bloqueado “QuickDesk” para proteger tu Mac», confirma con tu contraseña o Touch ID y haz clic en <em>Abrir</em>. En macOS 14 o anterior también sirve Control-clic en QuickDesk → <em>Abrir</em> → <em>Abrir</em>.</li>
<li>O ejecuta <code>xattr -dr com.apple.quarantine /Applications/QuickDesk.app</code> en Terminal y abre QuickDesk como siempre.</li>
</ol>
<p>La <a href="/download#macos-open">página de descarga</a> muestra los mismos pasos.</p>
<ul>
<li>QuickDesk vive en la barra de menús, sin icono en el Dock. Sus ventanas emergentes se abren en el Espacio en el que estés, también encima de una app a pantalla completa.</li>
<li>Los atajos son <kbd>⌃</kbd> <kbd>⌥</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd> (Control + Opción): ⌘⌥V y ⌘⌥N pertenecen al Finder.</li>
<li>El <strong>pegado automático</strong> pulsa ⌘V en la app que estabas usando, y macOS solo lo permite después de que añadas QuickDesk en Ajustes del Sistema → Privacidad y seguridad → <strong>Accesibilidad</strong>. macOS lo pregunta la primera vez que eliges una entrada; hasta entonces la entrada se copia y tú pulsas ⌘V. Settings → Auto-paste muestra si está permitido.</li>
<li>Las versiones recientes de macOS pueden preguntar si QuickDesk puede leer lo que copian otras apps. Elige Permitir, o el historial del portapapeles se quedará vacío.</li>
<li><strong>Uso de Claude:</strong> en un Mac, Claude Code guarda su inicio de sesión en el Llavero. La pestaña AI ofrece leerlo de ahí; macOS pregunta entonces una vez (elige <em>Permitir siempre</em>). Como en los demás sistemas, el inicio de sesión solo se lee y solo se envía a <code>api.anthropic.com</code>.</li>
<li>Como la app está firmada de forma ad hoc, macOS puede volver a pedir los permisos de Accesibilidad y del Llavero tras una actualización. Si el pegado automático deja de funcionar, quita QuickDesk de la lista de Accesibilidad y vuelve a añadirlo.</li>
<li>La pestaña Runtimes funciona como en Linux, con tu shell de inicio de sesión (zsh: <code>~/.zshrc</code>). Las copias instaladas con Homebrew se muestran, pero no se modifican.</li>
</ul>
<p><strong>Desinstalar:</strong> sal de QuickDesk desde la barra de menús y muévelo de Aplicaciones a la Papelera. Tus datos se quedan en <code>~/Library/Application Support/click.quickdesk</code> hasta que borres esa carpeta.</p>`,
  },
  shortcuts: {
    title: "Atajos",
    html: `<table>
<tr><th>Atajo</th><th>Abre</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>Notas</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>Ventana del historial del portapapeles</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>Puertos</td></tr>
<tr><td>sin asignar</td><td>Ventana de nota rápida (asígnale uno en Settings)</td></tr>
</table>
<p>Pulsar de nuevo un atajo con su ventana abierta la cierra. Cambia cualquiera en <strong>Settings → Hotkeys</strong>; QuickDesk avisa cuando una combinación ya está en uso.</p>
<p>En GNOME los atajos se añaden a <em>Configuración → Teclado → Atajos personalizados</em> y se quitan cuando QuickDesk se cierra, así que también funcionan en Wayland. En otros escritorios, asigna <code>quickdesk toggle notes</code>, <code>quickdesk toggle clipboard</code>, <code>quickdesk toggle ports</code> o <code>quickdesk toggle quick-note</code> a teclas en la configuración del teclado de tu escritorio.</p>
<p><strong>¿Por qué Super + Alt?</strong> Los métodos de entrada vietnamitas, como Unikey y Bamboo, se tragan Super + Shift + letra mientras escribes en un campo de texto; las combinaciones con Alt siguen funcionando.</p>`,
  },
  notes: {
    title: "Notas",
    html: `<p>Las notas son Markdown con un editor enriquecido: títulos, negrita y cursiva, listas, listas de tareas, citas y bloques de código. <kbd>Ctrl</kbd> <kbd>N</kbd> crea una nota nueva, <kbd>Ctrl</kbd> <kbd>S</kbd> guarda, <kbd>Ctrl</kbd> <kbd>F</kbd> busca. La búsqueda ignora los acentos, así que <code>tieng viet</code> encuentra «tiếng Việt».</p>
<p>Fija una nota con la estrella para mantenerla arriba. Al borrar se pide confirmación, y se puede deshacer durante unos segundos.</p>`,
  },
  sync: {
    title: "Sincronizar notas entre equipos",
    html: `<p>Las notas se sincronizan a través de <strong>QuickDesk Cloud</strong>. No hay cuenta ni correo: un equipo recibe un <strong>código de sincronización</strong> y tus otros equipos se unen con él. Solo se sincronizan las notas; el historial del portapapeles y la configuración se quedan en cada equipo.</p>
<h3>Configurarlo</h3>
<ol>
<li>En el primer equipo: <strong>Settings → Sync → Turn on sync</strong>, y elige una frase de contraseña (al menos 8 caracteres).</li>
<li>QuickDesk muestra tu <strong>código de sincronización</strong> (<code>QD1-…</code>) y una <strong>clave de recuperación</strong>. Guarda la clave de recuperación en un lugar seguro; solo se muestra una vez.</li>
<li>En cada uno de los otros equipos: <strong>Settings → Sync → I have a sync code</strong>, e introduce el código y la misma frase de contraseña (o la clave de recuperación).</li>
</ol>
<p>Puedes volver a ver el código en cualquier momento con <strong>Show sync code</strong> en Settings → Sync.</p>
<h3>Cuándo se sincroniza</h3>
<ul>
<li>Un par de segundos después de editar una nota, y cuando abres una ventana de QuickDesk.</li>
<li>Por lo demás, QuickDesk busca cambios cada minuto mientras hay una ventana abierta, y cada 5 minutos mientras está en la bandeja. Cada comprobación es una sola petición pequeña; las notas solo viajan cuando algo cambió.</li>
<li><strong>Sync now</strong> comprueba en el acto. Los cambios hechos sin conexión se envían cuando vuelves a estar en línea.</li>
<li>Si una nota cambió en dos equipos a la vez, se conservan ambas versiones: la más reciente se queda y la otra pasa a ser una copia a su lado. Una edición gana a un borrado.</li>
</ul>
<h3>Privacidad y claves</h3>
<ul>
<li>Las notas se cifran en tu equipo antes de subirse (XChaCha20-Poly1305, clave derivada de tu frase de contraseña con Argon2id). QuickDesk Cloud solo guarda texto cifrado bajo un identificador de cuenta aleatorio y no puede leer tus notas.</li>
<li>Hacen falta <strong>las dos cosas</strong>: el código de sincronización (para llegar a los datos) y la frase de contraseña (para descifrarlos). Trata el código como una contraseña: con él solo, alguien podría borrar tus notas sincronizadas.</li>
<li>¿Olvidaste la frase de contraseña? Introduce la clave de recuperación. Sin ninguna de las dos, la copia de QuickDesk Cloud no se puede abrir, pero las notas de cada equipo no se tocan.</li>
<li>Si QuickDesk pide <strong>Unlock sync</strong> (por ejemplo, tras restablecer el llavero del sistema), vuelve a introducir la frase de contraseña.</li>
</ul>
<h3>Desactivarla</h3>
<p><strong>Turn off sync…</strong> deja de sincronizar en un equipo y conserva sus notas. Marca <em>Also delete my notes from QuickDesk Cloud</em> para borrar la copia sincronizada para todos los equipos.</p>
<p>Límites por código de sincronización: 256 MB y 20.000 objetos guardados, mucho más de lo que necesitan unas notas.</p>`,
  },
  clipboard: {
    title: "Historial del portapapeles",
    html: `<p>Todo lo que copias se guarda: texto, imágenes (capturas de pantalla) y archivos copiados en tu gestor de archivos. Abre la ventana con <kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd>, escribe para buscar, filtra por texto, imágenes o archivos y pulsa <kbd>Enter</kbd> para pegar en la app que estabas usando. <kbd>Ctrl</kbd> <kbd>Enter</kbd> solo copia, <kbd>Ctrl</kbd> <kbd>P</kbd> fija, <kbd>Supr</kbd> borra.</p>
<ul>
<li>El historial se queda en este equipo y nunca se sincroniza.</li>
<li>Lo que los gestores de contraseñas marcan como secreto se omite. Antes de copiar algo delicado, <strong>pausa el guardado</strong> durante 15 minutos, una hora o hasta que lo reactives (en la pestaña Clipboard o en el menú de la bandeja).</li>
<li>QuickDesk conserva las 1.000 entradas más recientes; las fijadas, para siempre. Texto de hasta 1 MB, imágenes de hasta 10 MB cada una y 200 MB en total.</li>
<li><strong>Clear…</strong> en la pestaña Clipboard borra todo salvo las entradas fijadas.</li>
</ul>`,
  },
  "auto-paste": {
    title: "Pegado automático",
    html: `<p>Cuando eliges una entrada, QuickDesk vuelve a la app que estabas usando y pulsa <kbd>Shift</kbd> <kbd>Insert</kbd>, que también funciona en terminales. Elige cómo en <strong>Settings → Auto-paste</strong>:</p>
<ul>
<li><strong>Desktop portal</strong> (predeterminado): GNOME pide permiso una vez y muestra un indicador de control remoto mientras pega.</li>
<li><strong>Instant (virtual keyboard)</strong>: sin avisos ni indicador. Al activarlo, pide tu contraseña de administrador una vez para permitir que los programas que ejecutas tú creen un teclado virtual (el mismo permiso que usa Steam para los mandos).</li>
</ul>
<p>Desactiva el pegado automático en la pestaña Clipboard para limitarte a copiar, y luego pulsa tú <kbd>Ctrl</kbd> <kbd>V</kbd>.</p>`,
  },
  ports: {
    title: "Puertos",
    html: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> muestra cada puerto TCP en escucha (IPv4 e IPv6) con su proceso, su línea de órdenes completa y su usuario, actualizado cada 2 segundos. Filtra por puerto, PID, proceso o nombre de contenedor.</p>
<ul>
<li><strong>Open</strong> abre <code>http://localhost:&lt;port&gt;</code> en tu navegador; <strong>Copy PID</strong> y <strong>Copy cmd</strong> los copian.</li>
<li><strong>Kill</strong> pregunta primero, luego detiene el proceso con educación y solo ofrece forzar el cierre si no termina.</li>
<li>Los puertos publicados por Docker muestran el contenedor y la imagen, y <strong>Stop</strong> detiene el contenedor en lugar de matar <code>docker-proxy</code>.</li>
<li>Los procesos de otros usuarios (servicios del sistema) aparecen ocultos; examinarlos requiere root.</li>
</ul>`,
  },
  versions: {
    title: "Versiones de lenguajes (Runtimes)",
    html: `<p>La pestaña <strong>Runtimes</strong> gestiona las versiones de los lenguajes de programación de este equipo (Node.js, Python, Rust, Go, Java, Ruby, Deno, Bun, PHP). Muestra la versión que ejecuta una <em>terminal nueva</em> y de dónde viene. QuickDesk pregunta a tu shell de inicio de sesión, así que esto es lo que obtienes después de <code>~/.bashrc</code> o <code>~/.zshrc</code>, no lo que ve el escritorio.</p>
<ul>
<li>Los lenguajes instalados con <strong>nvm</strong> (Node.js), <strong>rustup</strong> (Rust), <strong>uv</strong> (Python) o <strong>mise</strong> (Go, Java, Ruby, Deno, Bun, PHP y los demás) se pueden cambiar: <strong>Install version…</strong> elige entre las versiones que ofrece la herramienta (rustup acepta un canal o una versión que escribas), <strong>Make default</strong> fija lo que usan las terminales nuevas, <strong>Uninstall</strong> quita una y libera su espacio. La predeterminada no se puede quitar; elige antes otra predeterminada.</li>
<li>Las instalaciones se ejecutan en segundo plano con una barra de progreso y se pueden cancelar. Solo se ejecuta un cambio a la vez. La lista de versiones muestra por defecto la última versión de cada rama; cambia a LTS o a todas, o escribe para filtrar.</li>
<li>Cuando sale una versión más nueva de una rama instalada (Node 22.24.1 para 22.23.3, Python 3.13.9 para 3.13.7), la versión muestra <strong>↑ 22.24.1</strong>: un clic la instala y, si la antigua era la predeterminada, la convierte en predeterminada. La antigua se queda hasta que la desinstales. Los canales de Rust se actualizan en su sitio: <strong>stable</strong> muestra <strong>↑ 1.93.0</strong> cuando rustup tiene un stable más nuevo, y el clic ejecuta <code>rustup update stable</code>.</li>
<li><strong>Apply right away in open terminals</strong> (arriba en la pestaña) hace que los cambios lleguen a las terminales ya abiertas, desde su siguiente prompt: QuickDesk añade una línea a tu archivo de shell que carga un pequeño hook del prompt (<code>~/.config/quickdesk/shell/hook.sh</code>). El hook hace que el shell vuelva a buscar las órdenes después de que QuickDesk cambie una versión, y que Node siga el <code>.nvmrc</code> más cercano cuando entras con <code>cd</code> en un proyecto, o si no la predeterminada que elijas (un <code>nvm use</code> manual se mantiene hasta que cambies de carpeta o de predeterminada). Sin él, los cambios de Rust se aplican igualmente al momento, los de Python y mise llegan a las terminales nuevas (o a una abierta tras <code>hash -r</code>) y los de nvm solo a las terminales nuevas.</li>
<li><strong>Set for a project…</strong> escribe el archivo propio de la herramienta en la carpeta que elijas: <code>.nvmrc</code>, <code>rust-toolchain.toml</code>, <code>.python-version</code> o <code>mise.toml</code> (mise conserva las demás herramientas que ya figuren ahí).</li>
<li>Un lenguaje sin gestor de versiones (por ejemplo, Go instalado con apt) ofrece <strong>Install mise…</strong>: QuickDesk descarga el último mise de github.com/jdx/mise, lo comprueba con las sumas SHA-256 de la versión y lo coloca en <code>~/.local/bin</code>. Después de que hagas predeterminada una versión, <strong>Fix PATH…</strong> ofrece la línea que activa mise en tu shell.</li>
<li>Si las terminales nuevas fueran a ejecutar otra copia distinta de la predeterminada (por ejemplo, el <code>python3</code> del sistema va primero en el PATH), la tarjeta explica por qué y <strong>Fix PATH…</strong> muestra exactamente las líneas que añadiría. Solo cuando aceptas, QuickDesk las añade como un bloque marcado al final de tu archivo de shell, con una copia de seguridad al lado; el bloque se puede quitar desde el mismo sitio.</li>
<li>Los arreglos del PATH y el hook del prompt se escriben para bash (<code>~/.bashrc</code>) y zsh (<code>~/.zshrc</code>); fish aún no está soportado.</li>
<li>Los lenguajes que todavía no están en el equipo aparecen en <strong>Add a language</strong>: elige uno para instalar mise si hace falta y luego una versión.</li>
<li><strong>Windows:</strong> QuickDesk pregunta a PowerShell qué ejecuta una terminal nueva y trabaja con <strong>mise</strong> (instalado en <code>%LOCALAPPDATA%\\mise\\bin</code>), <strong>rustup</strong> y <strong>uv</strong>; nvm-windows y fnm se muestran pero no se modifican, y el <code>python.exe</code> de marcador de posición de la Tienda se ignora. <strong>Fix PATH…</strong> pone las carpetas de la herramienta (para mise, sus shims) al principio del PATH de tu cuenta, el de Configuración → Variables de entorno, sin permisos de administrador; el valor anterior se guarda en <code>~\\.config\\quickdesk\\path</code> y el mismo panel vuelve a quitar las carpetas. Las terminales nuevas lo recogen; a partir de ahí, los cambios de predeterminada se aplican al momento. En Windows «Apply right away» no hace falta y no se muestra.</li>
<li>Las copias instaladas por el sistema (apt, dnf) nunca se cambian ni se quitan; QuickDesk nunca ejecuta <code>sudo</code>. Los lenguajes gestionados por herramientas que no soporta (fnm, volta, pyenv, SDKMAN…) se muestran en solo lectura.</li>
</ul>`,
  },
  ai: {
    title: "Uso de IA",
    html: `<p>Un anillo en la barra superior muestra cuánto has usado de tus límites de IA, en verde, ámbar o rojo. Su menú muestra la herramienta que sigues, con una barra por cada límite y cuándo se reinicia; la pestaña AI muestra todas las herramientas.</p>
<ul>
<li><strong>Claude Code:</strong> QuickDesk lee el inicio de sesión que Claude Code ya guardó en este equipo y pregunta a Anthropic por tu uso, como mucho cada 5 minutos. El token solo se lee: nunca se guarda, se renueva ni se envía a ningún sitio salvo a Anthropic.</li>
<li><strong>Codex:</strong> se lee de los registros de sesión que Codex guarda en <code>~/.codex</code>.</li>
<li><strong>Gemini (Antigravity):</strong> no informa de un porcentaje, así que QuickDesk muestra, a partir de sus registros, cuándo agotaste la cuota por última vez.</li>
</ul>
<p>Elige qué herramienta y qué límite seguir en <em>Track</em>, u oculta el anillo en la pestaña AI.</p>`,
  },
  updates: {
    title: "Actualizaciones",
    html: `<p>QuickDesk busca una versión nueva 30 segundos después de arrancar y luego cada 6 horas. Cuando sale una, un aviso en la ventana principal y una entrada al principio del menú de la bandeja lo indican, con <strong>What’s New</strong>. No se instala nada hasta que haces clic en <strong>Update Now</strong>.</p>
<p>Cada actualización se verifica con la clave de firma de QuickDesk antes de instalarse. Un .deb o .rpm pide tu contraseña de administrador; un AppImage se reemplaza a sí mismo. Después QuickDesk se reinicia y muestra qué ha cambiado. Desactiva las comprobaciones automáticas, o comprueba a mano, en <strong>Settings → Updates</strong>.</p>`,
  },
  uninstall: {
    title: "Desinstalar",
    html: `<ol>
<li>Desactiva <em>Start QuickDesk when I log in</em> en Settings y sal desde la bandeja.</li>
<li>Quita el paquete: <code>sudo apt remove quick-desk</code> (Debian/Ubuntu) o <code>sudo dnf remove QuickDesk</code> (Fedora), o borra el AppImage.</li>
<li>Tus datos se quedan en <code>~/.local/share/click.quickdesk</code> hasta que borres esa carpeta.</li>
</ol>
<p><strong>Windows:</strong> Configuración → Aplicaciones → Aplicaciones instaladas → QuickDesk → Desinstalar. Tus datos se quedan en <code>%APPDATA%\\click.quickdesk</code> hasta que borres esa carpeta.</p>
<p>Quitar el paquete también retira el permiso de pegado instantáneo, si lo activaste.</p>`,
  },
  troubleshooting: {
    title: "Solución de problemas",
    html: `<ul>
<li><strong>Un atajo no hace nada:</strong> revisa si hay un conflicto en <em>Configuración → Teclado → Atajos personalizados</em>, o elige otra combinación en Settings → Hotkeys.</li>
<li><strong>El pegado automático no escribe nada:</strong> comprueba que la app en la que estabas sigue teniendo el foco; cambia el método en Settings → Auto-paste. La entrada siempre se copia, así que <kbd>Ctrl</kbd> <kbd>V</kbd> funciona.</li>
<li><strong>Registros:</strong> <code>~/.local/share/click.quickdesk/logs/</code>. Adjúntalos cuando <a href="https://github.com/TankGum/quickdesk/issues">informes de un problema</a>.</li>
</ul>`,
  },
};

export default docs;
