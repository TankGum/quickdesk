// Español (Spanish). See ../en.ts for what each part is. The app's menu and
// button names are written as the app itself shows them
// (src/shared/locales/es.ts). The system's own labels are named as the system
// shows them.
import type { Locale } from "..";
import changelog from "./CHANGELOG.md?raw";
import docs from "./docs";
import home from "./home";
import privacy from "./privacy";

const es: Locale = {
  strings: {
    ui: {
      home: "Inicio",
      download: "Descargar",
      docs: "Documentación",
      changelog: "Novedades",
      privacy: "Privacidad",
      github: "GitHub",
      language: "Idioma",
      footer: "QuickDesk · Licencia MIT.",
      tagline: "Notas, historial del portapapeles, puertos y límites de IA. A un atajo de distancia.",
      systems: "Sistemas",
      soon: "pronto",
      product: "Producto",
      project: "Proyecto",
      issues: "Informar de un problema",
      sitemap: "Mapa del sitio",
      skip: "Saltar al contenido",
      menu: "Menú",
      close: "Cerrar",
      faq: "Preguntas",
      theme: "Cambiar entre claro y oscuro",
      themeShort: "Claro / oscuro",
    },
    pages: {
      home: {
        title: "QuickDesk — Notas, portapapeles y puertos para Linux, Windows y macOS",
        description:
          "QuickDesk es un asistente de escritorio gratuito y de código abierto para Linux, Windows y macOS: notas cifradas de extremo a extremo, historial del portapapeles, un gestor de puertos en vivo y tus límites de uso de IA, con un solo atajo.",
      },
      download: {
        title: "Descargar QuickDesk para Linux, Windows y macOS (gratis)",
        description:
          "Descarga QuickDesk para Windows 10 y 11, macOS en Apple Silicon, Ubuntu, Debian, Fedora y otras distribuciones Linux: el instalador de Windows, .dmg, .deb, .rpm y AppImage, con sumas de verificación.",
      },
      docs: {
        title: "Documentación de QuickDesk: instalación, atajos, sincronización, portapapeles, puertos",
        description:
          "Cómo instalar QuickDesk en Linux, Windows y macOS y usar sus atajos, la sincronización de notas, el historial del portapapeles, los puertos, las versiones de lenguajes, el uso de IA y las actualizaciones.",
      },
      changelog: {
        title: "Novedades de QuickDesk: qué hay de nuevo en cada versión",
        description: "Lo nuevo, lo mejorado y lo corregido en cada versión de QuickDesk.",
      },
      privacy: {
        title: "Privacidad · QuickDesk",
        description: "QuickDesk no tiene cuentas, analítica ni telemetría. Qué se queda en tu ordenador y qué se envía y adónde.",
      },
      sitemap: {
        title: "Mapa del sitio · QuickDesk",
        description:
          "Todas las páginas de quickdesk.click: descargas para Linux, Windows y macOS, la documentación sección por sección, cada versión de las novedades y la privacidad.",
      },
    },
    seo: {
      app: "Un asistente de escritorio gratuito y de código abierto: notas sincronizadas y cifradas de extremo a extremo, historial del portapapeles, un gestor de puertos, versiones de lenguajes de programación y límites de uso de IA, cada uno a un atajo de distancia.",
      ogAlt: "QuickDesk: todo tu escritorio, a un atajo de distancia.",
    },
    hero: {
      pick: "Tu sistema",
      linux: "Descargar para Linux",
      windows: "Descargar para Windows",
      macos: "Descargar para macOS{beta}",
      macSoon: "macOS: muy pronto",
    },
    download: {
      title: "Descargar QuickDesk {v}",
      leadMac: "Para Linux y Windows (x86_64) y macOS (Apple Silicon{beta}) · publicado el {date}. Gratis y de código abierto (MIT).",
      leadSoon: "Para Linux y Windows en x86_64, macOS muy pronto · publicado el {date}. Gratis y de código abierto (MIT).",
      pick: "Tu sistema",
      devTitle: "La versión para Mac aún está en desarrollo",
      devText:
        "QuickDesk para macOS funciona, pero es nuevo: todavía no se ha probado todo en todos los Mac, y el pegado automático y los atajos se siguen puliendo. Si algo no funciona, cuéntanoslo.",
      devLink: "Informar de un problema en GitHub",
      deb: ["Ubuntu, Debian, Linux Mint, Pop!_OS", "Ubuntu 22.04 o posterior, Debian 12 o posterior."],
      rpm: ["Fedora y otras distribuciones basadas en RPM", "Fedora 36 o posterior, openSUSE Tumbleweed."],
      appimage: ["Cualquier otra distribución", "Portátil: no necesita instalación. Requiere FUSE 2 (libfuse2)."],
      windows: ["Windows 10 y 11", "64 bits. Se instala solo para tu usuario: sin permisos de administrador."],
      macos: ["macOS 11 o posterior", "Apple Silicon (M1 o posterior). Abre la imagen de disco y arrastra QuickDesk a Aplicaciones."],
      oneMac:
        "En Terminal. Descarga QuickDesk.app, comprueba su SHA-256, lo coloca en Aplicaciones y lo abre. Instalado así, macOS no muestra ninguna advertencia.",
      afterMac:
        "QuickDesk vive en la barra de menús y a partir de ahora se actualiza solo. Para el pegado automático, permítelo en Privacidad y seguridad → Accesibilidad cuando macOS lo pida.",
      gatekeeper: "Si lo descargas desde el navegador, macOS bloquea el primer arranque: QuickDesk aún no está notarizado por Apple.",
      gatekeeperHow: "Cómo abrirlo",
      openTitle: "Abrir QuickDesk por primera vez",
      openLead:
        "QuickDesk aún no está notarizado por Apple (para eso hace falta un Apple Developer ID de pago). Si el .dmg se descargó con el navegador, macOS detiene el primer arranque con el aviso de que «Apple no puede comprobar si contiene software malicioso». A la app no le pasa nada: haz una de estas dos cosas una sola vez. Si lo instalaste con el comando de arriba, puedes saltarte esto.",
      openA: "En Ajustes del Sistema",
      openASteps: [
        "Arrastra QuickDesk a Aplicaciones y ábrelo. Cuando macOS muestre el aviso, ciérralo.",
        "Abre Ajustes del Sistema → Privacidad y seguridad y baja hasta Seguridad.",
        "Junto a «Se ha bloqueado “QuickDesk” para proteger tu Mac», haz clic en Abrir igualmente, confirma con tu contraseña o Touch ID y luego haz clic en Abrir.",
      ],
      openOld: "En macOS 14 o anterior también sirve hacer Control-clic en QuickDesk dentro de Aplicaciones, elegir Abrir y volver a pulsar Abrir.",
      openB: "O en Terminal",
      openBLead: "Después de arrastrar QuickDesk a Aplicaciones, ejecuta esto y luego abre QuickDesk como siempre:",
      get: "Descargar",
      sha: "SHA-256",
      all: "Todas las sumas de verificación (SHA256SUMS)",
      notes: "Novedades de la versión {v}",
      allChanges: "Todos los cambios",
      afterLinux: "Después abre QuickDesk desde el menú de aplicaciones. Vive en la barra superior y a partir de ahora se actualiza solo.",
      afterWindows: "Después abre QuickDesk desde el menú Inicio. Vive en la bandeja del sistema y a partir de ahora se actualiza solo.",
      copy: "Copiar",
      oneTitle: "Instalar con un solo comando",
      oneLinux:
        "En una terminal. Elige el paquete adecuado para tu distribución (.deb, .rpm o AppImage), comprueba su SHA-256 y lo instala. Solo pide tu contraseña en el paso de instalación.",
      oneWindows: "En PowerShell. Descarga el instalador, comprueba su SHA-256 e instala QuickDesk para tu usuario, sin permisos de administrador.",
      oneScript: "Leer el script",
      orPick: "O elige un paquete",
      smartscreen: "Windows puede decir que protegió tu PC: el instalador aún no está firmado. Elige Más información → Ejecutar de todas formas.",
      macTitle: "QuickDesk para macOS está en camino",
      macLead: "Notas, historial del portapapeles, puertos y límites de IA en el Mac. Se anunciará en las novedades y en los lanzamientos de GitHub.",
      macWatch: "Seguir los lanzamientos en GitHub",
    },
    cta: {
      title: "Listo cuando tú lo estés.",
      lead: "Instálalo una vez y todo queda a un atajo de distancia: tus notas, lo que copiaste, los puertos en uso y tus límites de IA.",
      download: "Descargar",
      changes: "Novedades",
      meta: "Versión {v} · {date} · Gratis y de código abierto (MIT)",
      downloading: "Descargando",
      verified: "SHA-256 verificado",
      installed: "QuickDesk {v} instalado",
      running: "QuickDesk se está ejecutando",
      open: "Ábrelo con",
    },
    intro: {
      tagline: "Todo tu escritorio. A un atajo de distancia.",
      skip: "Saltar",
    },
    docs: {
      title: "Documentación",
      lead: "Instalación, atajos, sincronización, portapapeles, puertos, uso de IA y actualizaciones.",
      contents: "Contenido",
      groups: ["Primeros pasos", "Funciones", "Ayuda"],
      sync: "Sincronización",
      clipboard: "Portapapeles",
      versions: "Runtimes",
      stuck: "¿Sigues atascado?",
      ask: "Pregunta en GitHub",
    },
    sitemap: {
      title: "Mapa del sitio",
      lead: "Todas las páginas y secciones de quickdesk.click.",
      home: "Inicio",
      demo: "Prueba los atajos",
      getIt: "Consigue QuickDesk",
      oneLine: "Scripts de instalación",
      other: "En otros sitios",
      machine: "sitemap.xml (para buscadores)",
      privacyNote: "Sin cuentas, analítica ni telemetría: qué se queda en tu ordenador y qué se envía y adónde.",
    },
    changelog: {
      title: "Novedades",
      lead: "Lo nuevo, lo mejorado y lo corregido en cada versión de QuickDesk.",
      latest: "Última",
      source: "Fuente: el registro de cambios en GitHub",
      english: "Aún sin traducir; se muestra en inglés.",
    },
    notFound: {
      title: "Página no encontrada · QuickDesk",
      description: "Esta página no existe.",
      heading: "Esta página no existe.",
    },
    demo: {
      refreshed: "Uso actualizado",
      paused: "Historial del portapapeles en pausa",
      resumed: "Historial del portapapeles reanudado",
      quit: "Salir — en la app real, QuickDesk se cerraría ahora",
      encrypting: "Cifrando…",
      synced: "Cifrado y sincronizado",
      copied: "Copiado",
      pressCopy: "Pulsa Ctrl+C",
      copy: "Copiar",
      note: "Reunión diaria — martes\n• Publicadas las imágenes del portapapeles\n• Revisando el proceso de publicación\n• Sin bloqueos",
    },
  },
  content: { home, docs, privacy, changelog },
};

export default es;
