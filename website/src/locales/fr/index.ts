// Français (French). See ../en.ts for what each part is. The app itself is in
// English and Vietnamese, so its menu and button names stay in English.
import type { Locale } from "..";
import changelog from "./CHANGELOG.md?raw";
import docs from "./docs";
import home from "./home";
import privacy from "./privacy";

const fr: Locale = {
  strings: {
    ui: {
      home: "Accueil",
      download: "Télécharger",
      docs: "Documentation",
      changelog: "Nouveautés",
      privacy: "Confidentialité",
      github: "GitHub",
      language: "Langue",
      footer: "QuickDesk · Licence MIT.",
      tagline: "Notes, historique du presse-papiers, ports et limites d’IA. À un raccourci près.",
      systems: "Systèmes",
      soon: "bientôt",
      product: "Produit",
      project: "Projet",
      issues: "Signaler un problème",
      sitemap: "Plan du site",
      skip: "Aller au contenu",
      menu: "Menu",
      close: "Fermer",
      faq: "FAQ",
      theme: "Basculer entre clair et sombre",
      themeShort: "Clair / sombre",
    },
    pages: {
      home: {
        title: "QuickDesk — Notes, presse-papiers et ports pour Linux, Windows et macOS",
        description:
          "QuickDesk est un assistant de bureau gratuit et open source pour Linux, Windows et macOS : des notes chiffrées de bout en bout, l’historique du presse-papiers, un gestionnaire de ports en direct et vos limites d’utilisation de l’IA, à un seul raccourci.",
      },
      download: {
        title: "Télécharger QuickDesk pour Linux, Windows et macOS (gratuit)",
        description:
          "Téléchargez QuickDesk pour Windows 10 et 11, macOS sur Apple Silicon, Ubuntu, Debian, Fedora et les autres distributions Linux : l’installeur Windows, .dmg, .deb, .rpm et AppImage, avec sommes de contrôle.",
      },
      docs: {
        title: "Documentation de QuickDesk : installation, raccourcis, synchronisation, presse-papiers, ports",
        description:
          "Comment installer QuickDesk sur Linux, Windows et macOS, utiliser ses raccourcis, synchroniser les notes, l’historique du presse-papiers, les ports, les versions de langages, l’utilisation de l’IA et les mises à jour.",
      },
      changelog: {
        title: "Nouveautés de QuickDesk : ce qui change à chaque version",
        description: "Ce qui est nouveau, amélioré et corrigé dans chaque version de QuickDesk.",
      },
      privacy: {
        title: "Confidentialité · QuickDesk",
        description: "QuickDesk n’a ni compte, ni statistiques, ni télémétrie. Ce qui reste sur votre ordinateur et ce qui est envoyé, et où.",
      },
      sitemap: {
        title: "Plan du site · QuickDesk",
        description:
          "Toutes les pages de quickdesk.click : téléchargements pour Linux, Windows et macOS, la documentation section par section, chaque version des nouveautés et la confidentialité.",
      },
    },
    seo: {
      app: "Un assistant de bureau gratuit et open source : des notes synchronisées et chiffrées de bout en bout, l’historique du presse-papiers, un gestionnaire de ports, les versions des langages de programmation et vos limites d’utilisation de l’IA, chacun à un raccourci près.",
      ogAlt: "QuickDesk : tout votre bureau, à un raccourci près.",
    },
    hero: {
      pick: "Votre système",
      linux: "Télécharger pour Linux",
      windows: "Télécharger pour Windows",
      macos: "Télécharger pour macOS{beta}",
      macSoon: "macOS : bientôt",
    },
    download: {
      title: "Télécharger QuickDesk {v}",
      leadMac: "Pour Linux et Windows (x86_64) et macOS (Apple Silicon{beta}) · publié le {date}. Gratuit et open source (MIT).",
      leadSoon: "Pour Linux et Windows sur x86_64, macOS bientôt · publié le {date}. Gratuit et open source (MIT).",
      pick: "Votre système",
      devTitle: "La version Mac est encore en développement",
      devText:
        "QuickDesk pour macOS fonctionne, mais il est récent : tout n’a pas encore été essayé sur chaque Mac, et le collage automatique et les raccourcis sont encore en cours de mise au point. Si quelque chose ne marche pas, dites-le-nous.",
      devLink: "Signaler un problème sur GitHub",
      deb: ["Ubuntu, Debian, Linux Mint, Pop!_OS", "Ubuntu 22.04 et plus récent, Debian 12 et plus récent."],
      rpm: ["Fedora et autres distributions basées sur RPM", "Fedora 36 et plus récent, openSUSE Tumbleweed."],
      appimage: ["Toute autre distribution", "Portable : aucune installation. Nécessite FUSE 2 (libfuse2)."],
      windows: ["Windows 10 et 11", "64 bits. S’installe pour votre utilisateur seulement : pas besoin de droits d’administrateur."],
      macos: ["macOS 11 ou plus récent", "Apple Silicon (M1 et plus récent). Ouvrez l’image disque et faites glisser QuickDesk dans Applications."],
      oneMac:
        "Dans le Terminal. Télécharge QuickDesk.app, vérifie son SHA-256, le place dans Applications et l’ouvre. Installé ainsi, macOS n’affiche aucun avertissement.",
      afterMac:
        "QuickDesk vit dans la barre des menus et se met à jour tout seul désormais. Pour le collage automatique, autorisez-le dans Confidentialité et sécurité → Accessibilité quand macOS le demande.",
      gatekeeper: "Téléchargé dans un navigateur, macOS bloque le premier lancement : QuickDesk n’est pas encore notarié par Apple.",
      gatekeeperHow: "Comment l’ouvrir",
      openTitle: "Ouvrir QuickDesk la première fois",
      openLead:
        "QuickDesk n’est pas encore notarié par Apple (il faut pour cela un Apple Developer ID payant). Si le .dmg a été téléchargé dans un navigateur, macOS arrête le premier lancement en indiquant qu’« Apple ne peut pas vérifier qu’il ne contient pas de logiciel malveillant ». L’app n’a aucun problème : faites l’une de ces deux choses une seule fois. Si vous l’avez installée avec la commande ci-dessus, vous pouvez passer cette étape.",
      openA: "Dans Réglages Système",
      openASteps: [
        "Faites glisser QuickDesk dans Applications et ouvrez-le. Quand macOS affiche l’avertissement, fermez-le.",
        "Ouvrez Réglages Système → Confidentialité et sécurité et faites défiler jusqu’à Sécurité.",
        "À côté de « “QuickDesk” a été bloqué pour protéger votre Mac », cliquez sur Ouvrir quand même, confirmez avec votre mot de passe ou Touch ID, puis cliquez sur Ouvrir.",
      ],
      openOld: "Sur macOS 14 ou antérieur, un Contrôle-clic sur QuickDesk dans Applications, puis Ouvrir, et encore Ouvrir, fonctionne aussi.",
      openB: "Ou dans le Terminal",
      openBLead: "Après avoir fait glisser QuickDesk dans Applications, lancez ceci, puis ouvrez QuickDesk comme d’habitude :",
      get: "Télécharger",
      sha: "SHA-256",
      all: "Toutes les sommes de contrôle (SHA256SUMS)",
      notes: "Nouveautés de la version {v}",
      allChanges: "Tous les changements",
      afterLinux: "Ouvrez ensuite QuickDesk depuis le menu des applications. Il vit dans la barre supérieure et se met à jour tout seul désormais.",
      afterWindows: "Ouvrez ensuite QuickDesk depuis le menu Démarrer. Il vit dans la zone de notification et se met à jour tout seul désormais.",
      copy: "Copier",
      oneTitle: "Installer en une commande",
      oneLinux:
        "Dans un terminal. Choisit le bon paquet pour votre distribution (.deb, .rpm ou AppImage), vérifie son SHA-256 et l’installe. Il ne demande votre mot de passe que pour l’étape d’installation.",
      oneWindows: "Dans PowerShell. Télécharge l’installeur, vérifie son SHA-256 et installe QuickDesk pour votre utilisateur, sans droits d’administrateur.",
      oneScript: "Lire le script",
      orPick: "Ou choisissez un paquet",
      smartscreen: "Windows peut indiquer qu’il a protégé votre PC : l’installeur n’est pas encore signé. Choisissez Informations complémentaires → Exécuter quand même.",
      macTitle: "QuickDesk pour macOS arrive",
      macLead: "Notes, historique du presse-papiers, ports et limites d’IA sur le Mac. Ce sera annoncé dans les nouveautés et dans les versions publiées sur GitHub.",
      macWatch: "Suivre les versions sur GitHub",
    },
    cta: {
      title: "Prêt quand vous l’êtes.",
      lead: "Installez-le une fois et tout est à un raccourci près : vos notes, ce que vous avez copié, les ports utilisés et vos limites d’IA.",
      download: "Télécharger",
      changes: "Nouveautés",
      meta: "Version {v} · {date} · Gratuit et open source (MIT)",
      downloading: "Téléchargement de",
      verified: "SHA-256 vérifié",
      installed: "QuickDesk {v} installé",
      running: "QuickDesk est lancé",
      open: "Ouvrez-le avec",
    },
    intro: {
      tagline: "Tout votre bureau. À un raccourci près.",
      skip: "Passer",
    },
    docs: {
      title: "Documentation",
      lead: "Installation, raccourcis, synchronisation, presse-papiers, ports, utilisation de l’IA et mises à jour.",
      contents: "Sommaire",
      groups: ["Premiers pas", "Fonctionnalités", "Aide"],
      sync: "Synchronisation",
      clipboard: "Presse-papiers",
      versions: "Runtimes",
      stuck: "Toujours bloqué ?",
      ask: "Demandez sur GitHub",
    },
    sitemap: {
      title: "Plan du site",
      lead: "Toutes les pages et sections de quickdesk.click.",
      home: "Accueil",
      demo: "Essayer les raccourcis",
      getIt: "Obtenir QuickDesk",
      oneLine: "Scripts d’installation",
      other: "Ailleurs",
      machine: "sitemap.xml (pour les moteurs de recherche)",
      privacyNote: "Ni compte, ni statistiques, ni télémétrie : ce qui reste sur votre ordinateur et ce qui est envoyé, et où.",
    },
    changelog: {
      title: "Nouveautés",
      lead: "Ce qui est nouveau, amélioré et corrigé dans chaque version de QuickDesk.",
      latest: "Dernière",
      source: "Source : le journal des modifications sur GitHub",
      english: "Pas encore traduit ; affiché en anglais.",
    },
    notFound: {
      title: "Page introuvable · QuickDesk",
      description: "Cette page n’existe pas.",
      heading: "Cette page n’existe pas.",
    },
    demo: {
      refreshed: "Utilisation actualisée",
      paused: "Historique du presse-papiers en pause",
      resumed: "Historique du presse-papiers repris",
      quit: "Quitter — dans la vraie app, QuickDesk se fermerait maintenant",
      encrypting: "Chiffrement…",
      synced: "Chiffré et synchronisé",
      copied: "Copié",
      pressCopy: "Appuyez sur Ctrl+C",
      copy: "Copier",
      note: "Point du matin — mardi\n• Images du presse-papiers livrées\n• Relecture du processus de publication\n• Aucun blocage",
    },
  },
  content: { home, docs, privacy, changelog },
};

export default fr;
