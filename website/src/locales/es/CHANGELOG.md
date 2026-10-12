# Novedades

Lo que cambió en cada versión de QuickDesk (traducción al español). El original en inglés está en CHANGELOG.md, en la raíz del repositorio; las versiones que falten aquí se muestran en inglés.

## 0.7.0 — 2026-10-12

### Nuevo
- La app habla los idiomas del sitio web: además de inglés y vietnamita, chino simplificado, japonés, hindi, español, francés, árabe (de derecha a izquierda), bengalí, ruso, portugués (Brasil) e indonesio. Elige uno en Ajustes → Idioma; «Igual que el sistema» ahora también sigue el idioma del sistema en macOS y Windows. Los menús de la bandeja y las novedades usan el mismo idioma.

## 0.6.1 — 2026-10-12

### Corregido
- macOS: elegir una entrada en la ventana del portapapeles vuelve a pegarla en el campo de texto en el que estabas escribiendo. Ahora QuickDesk se aparta para que macOS devuelva el foco a esa app y a ese campo, en lugar de pedir que se active la app, algo que las versiones recientes de macOS pueden cumplir solo a medias.

## 0.6.0 — 2026-10-11

### Nuevo
- QuickDesk para macOS (Apple Silicon, macOS 11 o posterior): notas con sincronización, historial del portapapeles con texto, imágenes y archivos, pegado automático con ⌘V en la app que estabas usando, puertos, versiones de lenguajes y uso de IA, desde la barra de menús. Los atajos son ⌃⌥N / V / P. Instálalo desde Terminal con `curl -fsSL https://quickdesk.click/install.sh | bash` o como .dmg desde la página de descarga; se actualiza solo, como en los demás sistemas. La app aún no está notarizada por Apple: si el .dmg se descargó con el navegador, macOS bloquea el primer arranque, y la página de descarga explica cómo abrirla (Ajustes del Sistema → Privacidad y seguridad → Abrir igualmente).
- En un Mac, la pestaña AI también puede mostrar tus límites de Claude: allí Claude Code guarda su inicio de sesión en el Llavero, y QuickDesk solo lo lee después de que lo permitas.

### Corregido
- El script de instalación dice qué paquete le falta a una versión en lugar de detenerse sin decir nada.

## 0.5.0 — 2026-10-11

### Nuevo
- QuickDesk para Windows 10 y 11 (64 bits): notas con sincronización, historial del portapapeles con texto, imágenes y archivos, pegado automático en la app que estabas usando, puertos y uso de IA, con los mismos atajos Win+Alt+N / V / P. Instálalo desde PowerShell con `irm https://quickdesk.click/install.ps1 | iex` o desde la página de descarga; se instala para tu usuario sin permisos de administrador y se actualiza solo. La pestaña Runtimes funciona con mise, rustup y uv: instala mise con un clic, añade Node, Python, Go y los demás, y «Arreglar PATH…» pone sus carpetas en el PATH de tu cuenta sin permisos de administrador.
- Runtimes: los lenguajes que aún no están en el equipo aparecen en «Añadir un lenguaje», listos para instalarse con mise.

### Mejorado
- Runtimes: los canales de Rust también tienen botón de actualización. Cuando rustup tiene un stable (o beta, nightly) más nuevo, el canal muestra ↑ y un clic ejecuta `rustup update` para él.

## 0.4.0 — 2026-10-10

### Nuevo
- Una pestaña Runtimes para las versiones de tus lenguajes de programación: mira qué Node.js, Python, Rust, Go… ejecuta de verdad una terminal nueva y de dónde viene, instala y desinstala versiones, elige la predeterminada y fija una versión por proyecto. QuickDesk trabaja con nvm, uv, rustup y mise, y nunca toca los paquetes del sistema.
- Cuando sale una versión más nueva de una rama instalada (por ejemplo Node 22.24.1 para 22.23.3), la versión muestra un botón de actualización; la antigua se queda hasta que la quites.
- Los lenguajes sin gestor de versiones (por ejemplo Go instalado con apt) pueden obtener mise con un clic, descargado de sus versiones en GitHub y comprobado con sus sumas SHA-256.
- «Aplicar al momento en las terminales abiertas»: los cambios llegan a las terminales ya abiertas en su siguiente prompt, y Node sigue el `.nvmrc` más cercano cuando haces `cd`. Cuando algo en el PATH estorba, la pestaña explica por qué y ofrece las líneas exactas que añadir a tu archivo de shell, con una copia de seguridad.

### Mejorado
- La página de descarga muestra la orden completa para descargar e instalar desde una terminal.
- La lista de versiones puede mostrar la última versión de cada rama, solo las LTS o todas.

## 0.3.1 — 2026-10-09

### Nuevo
- La sincronización ya viene integrada: actívala en Ajustes → Sincronización y une tus otros equipos con un código de sincronización. Sin cuenta, correo ni configuración de almacenamiento. Las notas se siguen cifrando en tu ordenador antes de salir de él; QuickDesk Cloud solo guarda texto cifrado.

### Corregido
- Han vuelto las notas, el historial del portapapeles y la configuración de versiones anteriores. Al actualizar a 0.2.0 y a 0.3.0 se empezaba con una carpeta vacía en lugar de trasladar tus datos; ahora QuickDesk los trae por sí mismo y conserva las carpetas antiguas como copia de seguridad.
- Abrir Ajustes ya no vuelve grises los puntos de color de las demás pestañas.

### Mejorado
- Ya no se ofrece la sincronización a través de tu propio bucket de S3 o R2. Si la usabas, la sincronización se desactiva una vez, con una nota en Ajustes → Sincronización; tus notas se quedan en tu ordenador.
- El menú de uso de IA de la barra superior solo muestra la herramienta que sigues, con una barra verde, amarilla o roja por cada límite y la hora de reinicio debajo. Elige otra herramienta en Track.

## 0.3.0 — 2026-10-09

### Nuevo
- QuickDesk tiene su propia dirección: [quickdesk.click](https://quickdesk.click). Las descargas y actualizaciones llegan desde ahí.
- Una web de verdad: una página de descarga que sugiere el paquete adecuado, documentación, novedades y página de privacidad, en inglés y vietnamita.

### Mejorado
- Tus datos ahora viven en `~/.local/share/click.quickdesk`. QuickDesk los traslada allí por sí mismo la primera vez que arranca; las notas, el historial del portapapeles y la configuración se quedan como estaban.
- El reinicio tras una actualización es más fiable.

## 0.2.2 — 2026-10-09

### Nuevo
- QuickDesk ahora se actualiza solo. Busca versiones nuevas en segundo plano, te cuenta qué ha cambiado y solo las instala cuando tú lo eliges. Cada actualización se verifica con la clave de firma de QuickDesk antes de instalarse.
- Ajustes → Actualizaciones: mira tu versión, comprueba ahora o desactiva las comprobaciones automáticas.
- Tras una actualización, QuickDesk muestra las novedades una vez.
- La página de descarga está disponible en vietnamita y enlaza al código fuente en GitHub.

### Mejorado
- Un solo icono en todas partes: el dock, el menú de aplicaciones, las ventanas y la bandeja comparten ahora el icono de QuickDesk.

## 0.2.1 — 2026-10-08

### Nuevo
- Un aspecto nuevo, compartido por la app y su página de descarga: un color por sección (notas, portapapeles, puertos, IA), pistas de teclado en las ventanas emergentes y una barra de pestañas al estilo de macOS.
- Iconos de línea en lugar de emojis en toda la app.
- Menús de la bandeja con un estilo macOS más limpio: etiquetas cortas, una marca de verificación para «Save Portapapeles History» y los límites de IA agrupados por herramienta.

### Mejorado
- La demo de la página de descarga es la app real funcionando con datos de ejemplo.

## 0.2.0 — 2026-10-08

### Nuevo
- Paquetes para Ubuntu/Debian (.deb), Fedora (.rpm) y otras distribuciones (AppImage), publicados en el sitio de descargas de QuickDesk.

### Mejorado
- Tus datos se trasladan automáticamente a la nueva carpeta de datos de QuickDesk (`~/.local/share/io.github.tankgum.quickdesk`).
