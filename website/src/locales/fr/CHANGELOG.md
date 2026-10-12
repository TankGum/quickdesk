# Nouveautés

Ce qui a changé dans chaque version de QuickDesk (traduction française). L’original anglais est dans CHANGELOG.md à la racine du dépôt ; les versions absentes ici s’affichent en anglais.

## Unreleased

### Nouveau
- L’application parle les langues du site : en plus de l’anglais et du vietnamien, le chinois simplifié, le japonais, l’hindi, l’espagnol, le français, l’arabe (de droite à gauche), le bengali, le russe, le portugais (Brésil) et l’indonésien. Choisissez-en une dans Réglages → Langue ; « Comme le système » suit désormais aussi la langue du système sur macOS et Windows. Les menus de la zone de notification et les nouveautés suivent la même langue.

## 0.6.1 — 2026-10-12

### Corrigé
- macOS : choisir une entrée dans la fenêtre du presse-papiers la colle de nouveau dans le champ de texte où vous écriviez. QuickDesk se met désormais en retrait pour que macOS rende le focus à cette app et à ce champ, au lieu de demander l’activation de l’app, ce que les versions récentes de macOS peuvent ne respecter qu’à moitié.

## 0.6.0 — 2026-10-11

### Nouveau
- QuickDesk pour macOS (Apple Silicon, macOS 11 ou plus récent) : notes synchronisées, historique du presse-papiers avec texte, images et fichiers, collage automatique avec ⌘V dans l’app que vous utilisiez, ports, versions des langages et utilisation de l’IA, depuis la barre des menus. Les raccourcis sont ⌃⌥N / V / P. Installez-le depuis le Terminal avec `curl -fsSL https://quickdesk.click/install.sh | bash` ou sous forme de .dmg depuis la page de téléchargement ; il se met à jour tout seul, comme sur les autres systèmes. L’app n’est pas encore notariée par Apple : si le .dmg a été téléchargé dans un navigateur, macOS bloque le premier lancement, et la page de téléchargement explique comment l’ouvrir (Réglages Système → Confidentialité et sécurité → Ouvrir quand même).
- Sur Mac, l’onglet AI peut aussi afficher vos limites Claude : Claude Code y garde sa connexion dans le Trousseau, et QuickDesk ne la lit qu’après votre autorisation.

### Corrigé
- Le script d’installation indique quel paquet manque à une version au lieu de s’arrêter sans un mot.

## 0.5.0 — 2026-10-11

### Nouveau
- QuickDesk pour Windows 10 et 11 (64 bits) : notes synchronisées, historique du presse-papiers avec texte, images et fichiers, collage automatique dans l’app que vous utilisiez, ports et utilisation de l’IA, avec les mêmes raccourcis Win+Alt+N / V / P. Installez-le depuis PowerShell avec `irm https://quickdesk.click/install.ps1 | iex` ou depuis la page de téléchargement ; il s’installe pour votre utilisateur sans droits d’administrateur et se met à jour tout seul. L’onglet Runtimes fonctionne via mise, rustup et uv : installez mise en un clic, ajoutez Node, Python, Go et les autres, et « Corriger le PATH… » met leurs dossiers dans le PATH de votre compte sans droits d’administrateur.
- Runtimes : les langages qui ne sont pas encore sur l’ordinateur sont listés sous « Ajouter un langage », prêts à être installés via mise.

### Amélioré
- Runtimes : les canaux Rust ont aussi un bouton de mise à jour. Quand rustup a un stable (ou beta, nightly) plus récent, le canal affiche ↑ et un clic lance `rustup update` pour lui.

## 0.4.0 — 2026-10-10

### Nouveau
- Un onglet Runtimes pour les versions de vos langages de programmation : voyez quel Node.js, Python, Rust, Go… un nouveau terminal exécute vraiment et d’où il vient, installez et désinstallez des versions, choisissez celle par défaut et fixez une version pour un projet. QuickDesk passe par nvm, uv, rustup et mise, et ne touche jamais aux paquets du système.
- Quand une version plus récente d’une branche installée sort (par exemple Node 22.24.1 pour 22.23.3), la version affiche un bouton de mise à jour ; l’ancienne reste jusqu’à ce que vous la supprimiez.
- Les langages sans gestionnaire de versions (Go installé par apt, par exemple) peuvent obtenir mise en un clic, téléchargé depuis ses versions GitHub et vérifié avec leurs sommes SHA-256.
- « Appliquer tout de suite dans les terminaux ouverts » : les changements arrivent dans les terminaux déjà ouverts à leur prochaine invite, et Node suit le `.nvmrc` le plus proche quand vous faites `cd`. Quand quelque chose dans le PATH gêne, l’onglet explique pourquoi et propose les lignes exactes à ajouter à votre fichier de shell, avec une sauvegarde.

### Amélioré
- La page de téléchargement montre la commande complète pour télécharger et installer depuis un terminal.
- La liste des versions peut montrer la dernière version de chaque branche, seulement les LTS, ou tout.

## 0.3.1 — 2026-10-09

### Nouveau
- La synchronisation est désormais intégrée : activez-la dans Réglages → Synchronisation et ajoutez vos autres ordinateurs avec un code de synchronisation. Ni compte, ni e-mail, ni stockage à configurer. Les notes sont toujours chiffrées sur votre ordinateur avant de le quitter ; QuickDesk Cloud ne stocke que du texte chiffré.

### Corrigé
- Les notes, l’historique du presse-papiers et les réglages des versions précédentes sont de retour. Les mises à jour vers 0.2.0 et 0.3.0 repartaient d’un dossier vide au lieu de déplacer vos données ; QuickDesk les rapatrie désormais tout seul et garde les anciens dossiers en sauvegarde.
- Ouvrir Réglages ne grise plus les points de couleur des autres onglets.

### Amélioré
- La synchronisation via votre propre bucket S3 ou R2 n’est plus proposée. Si vous l’utilisiez, la synchronisation est désactivée une fois, avec une note dans Réglages → Synchronisation ; vos notes restent sur votre ordinateur.
- Le menu d’utilisation de l’IA dans la barre supérieure ne montre que l’outil que vous suivez, avec une barre verte, jaune ou rouge par limite et l’heure de réinitialisation dessous. Choisissez un autre outil dans Track.

## 0.3.0 — 2026-10-09

### Nouveau
- QuickDesk a sa propre adresse : [quickdesk.click](https://quickdesk.click). Les téléchargements et les mises à jour viennent de là.
- Un vrai site web : une page de téléchargement qui suggère le bon paquet, la documentation, les nouveautés et la page de confidentialité, en anglais et en vietnamien.

### Amélioré
- Vos données vivent désormais dans `~/.local/share/click.quickdesk`. QuickDesk les y déplace tout seul au premier démarrage ; les notes, l’historique du presse-papiers et les réglages restent tels quels.
- Le redémarrage après une mise à jour est plus fiable.

## 0.2.2 — 2026-10-09

### Nouveau
- QuickDesk se met désormais à jour tout seul. Il cherche les nouvelles versions en arrière-plan, vous dit ce qui a changé et n’installe que quand vous le choisissez. Chaque mise à jour est vérifiée avec la clé de signature de QuickDesk avant d’être installée.
- Réglages → Mises à jour : voyez votre version, vérifiez maintenant ou désactivez les vérifications automatiques.
- Après une mise à jour, QuickDesk montre les nouveautés, une fois.
- La page de téléchargement existe en vietnamien et renvoie au code source sur GitHub.

### Amélioré
- Une seule icône partout : le dock, le menu des applications, les fenêtres et la barre partagent désormais l’icône de QuickDesk.

## 0.2.1 — 2026-10-08

### Nouveau
- Un nouveau look, partagé par l’app et sa page de téléchargement : une couleur par section (notes, presse-papiers, ports, IA), des indications clavier dans les fenêtres surgissantes et une barre d’onglets façon macOS.
- Des icônes au trait à la place des emojis dans toute l’app.
- Des menus de la barre dans un style macOS plus épuré : libellés courts, une coche pour « Save Presse-papiers History » et les limites d’IA regroupées par outil.

### Amélioré
- La démo de la page de téléchargement est la vraie app, qui tourne sur des données d’exemple.

## 0.2.0 — 2026-10-08

### Nouveau
- Des paquets pour Ubuntu/Debian (.deb), Fedora (.rpm) et les autres distributions (AppImage), publiés sur le site de téléchargement de QuickDesk.

### Amélioré
- Vos données sont déplacées automatiquement vers le nouveau dossier de données de QuickDesk (`~/.local/share/io.github.tankgum.quickdesk`).
