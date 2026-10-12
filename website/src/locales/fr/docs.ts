// Docs in French, by section id (src/data/docs.ts).
import { macBeta } from "../../data/release";

const docs: Record<string, { title: string; html: string }> = {
  install: {
    title: "Installation",
    html: `<p>Le plus rapide, sur n’importe quelle distribution :</p>
<pre><code>curl -fsSL https://quickdesk.click/install.sh | bash</code></pre>
<p>Il choisit le .deb, le .rpm ou l’AppImage pour votre système, vérifie son SHA-256 et l’installe (<a href="/install.sh">lire le script</a>). Ou récupérez un paquet sur la <a href="/download">page de téléchargement</a> et installez le fichier téléchargé avec :</p>
<ul>
<li><strong>Ubuntu, Debian, Mint, Pop!_OS :</strong> <code>sudo apt install ~/Downloads/QuickDesk_&lt;version&gt;_amd64.deb</code></li>
<li><strong>Fedora, openSUSE :</strong> <code>sudo dnf install ~/Downloads/QuickDesk-&lt;version&gt;-1.x86_64.rpm</code></li>
<li><strong>Toute autre :</strong> rendez l’AppImage exécutable et lancez-la. Elle nécessite FUSE 2 (<code>libfuse2</code>, sur Ubuntu 24.04 <code>libfuse2t64</code>).</li>
</ul>
<p>QuickDesk a besoin d’un système x86_64 de ces dernières années (glibc 2.34 ou plus récent). Il fonctionne au mieux sous GNOME, avec Wayland comme avec X11.</p>
<p><strong>Windows 10 et 11 (64 bits) :</strong> dans PowerShell, lancez <code>irm https://quickdesk.click/install.ps1 | iex</code>, ou téléchargez l’installeur depuis la <a href="/download">page de téléchargement</a>. Il s’installe pour votre utilisateur seulement, sans droits d’administrateur. L’installeur n’est pas encore signé, Windows peut donc indiquer qu’il a protégé votre PC : choisissez <em>Informations complémentaires → Exécuter quand même</em>. Sous Windows, les raccourcis sont <kbd>Win</kbd> <kbd>Alt</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd>, QuickDesk vit dans la zone de notification et le collage automatique ne demande aucun réglage. L’onglet Runtimes y fonctionne via mise, rustup et uv (voir <em>Versions des langages</em>).</p>
<p>Au premier lancement, un écran de bienvenue présente les raccourcis et propose deux options : <strong>démarrer à l’ouverture de session</strong> (nécessaire pour que l’historique du presse-papiers et l’anneau d’IA fonctionnent en permanence) et <strong>collage instantané</strong> (voir <a href="#auto-paste">Collage automatique</a>). Ensuite, QuickDesk vit dans la barre supérieure.</p>`,
  },
  macos: {
    title: "macOS",
    html: `${macBeta ? '<p class="note"><strong>Encore en développement.</strong> La version Mac fonctionne, mais elle est récente : tout n’a pas encore été essayé sur chaque Mac, et le collage automatique et les raccourcis sont encore en cours de mise au point. Merci de <a href="https://github.com/TankGum/quickdesk/issues">signaler</a> ce qui ne marche pas.</p>\n' : ""}<p>QuickDesk fonctionne sur macOS 11 ou plus récent sur Apple Silicon (M1 et plus récent). Dans le Terminal, lancez <code>curl -fsSL https://quickdesk.click/install.sh | bash</code> : il place QuickDesk.app dans Applications et l’ouvre. Ou téléchargez le <code>.dmg</code> depuis la <a href="/download#macos">page de téléchargement</a> et faites glisser QuickDesk dans Applications.</p>
<p><strong>Premier lancement.</strong> QuickDesk n’est pas encore notarié par Apple : si le <code>.dmg</code> a été téléchargé dans un navigateur, macOS arrête le premier lancement en indiquant qu’« Apple ne peut pas vérifier qu’il ne contient pas de logiciel malveillant ». L’app n’a aucun problème ; faites l’une de ces deux choses une fois (installé avec la commande, aucun avertissement) :</p>
<ol>
<li>Ouvrez QuickDesk depuis Applications et fermez l’avertissement de macOS. Ouvrez ensuite Réglages Système → Confidentialité et sécurité, faites défiler jusqu’à Sécurité, cliquez sur <em>Ouvrir quand même</em> à côté de « “QuickDesk” a été bloqué pour protéger votre Mac », confirmez avec votre mot de passe ou Touch ID et cliquez sur <em>Ouvrir</em>. Sur macOS 14 ou antérieur, Contrôle-clic sur QuickDesk → <em>Ouvrir</em> → <em>Ouvrir</em> fonctionne aussi.</li>
<li>Ou lancez <code>xattr -dr com.apple.quarantine /Applications/QuickDesk.app</code> dans le Terminal, puis ouvrez QuickDesk comme d’habitude.</li>
</ol>
<p>La <a href="/download#macos-open">page de téléchargement</a> montre les mêmes étapes.</p>
<ul>
<li>QuickDesk vit dans la barre des menus, sans icône dans le Dock. Ses fenêtres surgissantes s’ouvrent sur l’Espace où vous êtes, y compris par-dessus une app en plein écran.</li>
<li>Les raccourcis sont <kbd>⌃</kbd> <kbd>⌥</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd> (Contrôle + Option) : ⌘⌥V et ⌘⌥N appartiennent au Finder.</li>
<li>Le <strong>collage automatique</strong> appuie sur ⌘V dans l’app que vous utilisiez, ce que macOS n’autorise qu’après avoir ajouté QuickDesk dans Réglages Système → Confidentialité et sécurité → <strong>Accessibilité</strong>. macOS le demande la première fois que vous choisissez une entrée ; d’ici là, l’entrée est copiée et c’est vous qui appuyez sur ⌘V. Réglages → Collage automatique indique si c’est autorisé.</li>
<li>Les versions récentes de macOS peuvent demander si QuickDesk peut lire ce que copient les autres apps. Choisissez Autoriser, sinon l’historique du presse-papiers reste vide.</li>
<li><strong>Utilisation de Claude :</strong> sur Mac, Claude Code garde sa connexion dans le Trousseau. L’onglet AI propose de l’y lire ; macOS demande alors une fois (choisissez <em>Toujours autoriser</em>). Comme sur les autres systèmes, la connexion est seulement lue et n’est envoyée qu’à <code>api.anthropic.com</code>.</li>
<li>Comme l’app est signée ad hoc, macOS peut redemander l’accès à l’Accessibilité et au Trousseau après une mise à jour. Si le collage automatique ne fonctionne plus, retirez QuickDesk de la liste Accessibilité et ajoutez-le de nouveau.</li>
<li>L’onglet Runtimes fonctionne comme sous Linux, avec votre shell de connexion (zsh : <code>~/.zshrc</code>). Les copies installées avec Homebrew sont affichées mais pas modifiées.</li>
</ul>
<p><strong>Désinstaller :</strong> quittez QuickDesk depuis la barre des menus et placez-le d’Applications dans la Corbeille. Vos données restent dans <code>~/Library/Application Support/click.quickdesk</code> jusqu’à ce que vous supprimiez ce dossier.</p>`,
  },
  shortcuts: {
    title: "Raccourcis",
    html: `<table>
<tr><th>Raccourci</th><th>Ouvre</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>Notes</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>Fenêtre de l’historique du presse-papiers</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>Ports</td></tr>
<tr><td>non défini</td><td>Fenêtre de note rapide (à définir dans Réglages)</td></tr>
</table>
<p>Appuyer de nouveau sur un raccourci quand sa fenêtre est ouverte la ferme. Changez-les dans <strong>Réglages → Raccourcis</strong> ; QuickDesk prévient quand une combinaison est déjà prise.</p>
<p>Sous GNOME, les raccourcis sont ajoutés à <em>Paramètres → Clavier → Raccourcis personnalisés</em> et retirés quand QuickDesk se ferme, ils fonctionnent donc aussi sous Wayland. Sur les autres bureaux, associez <code>quickdesk toggle notes</code>, <code>quickdesk toggle clipboard</code>, <code>quickdesk toggle ports</code> ou <code>quickdesk toggle quick-note</code> à des touches dans les réglages du clavier de votre bureau.</p>
<p><strong>Pourquoi Super + Alt ?</strong> Les méthodes de saisie vietnamiennes comme Unikey et Bamboo avalent Super + Maj + lettre pendant que vous tapez dans un champ de texte ; les combinaisons avec Alt continuent de fonctionner.</p>`,
  },
  notes: {
    title: "Notes",
    html: `<p>Les notes sont en Markdown, avec un éditeur enrichi : titres, gras et italique, listes, listes de tâches, citations et blocs de code. <kbd>Ctrl</kbd> <kbd>N</kbd> crée une note, <kbd>Ctrl</kbd> <kbd>S</kbd> enregistre, <kbd>Ctrl</kbd> <kbd>F</kbd> cherche. La recherche ignore les accents : <code>tieng viet</code> trouve « tiếng Việt ».</p>
<p>Épinglez une note avec l’étoile pour la garder en haut. La suppression demande une confirmation et peut être annulée pendant quelques secondes.</p>`,
  },
  sync: {
    title: "Synchroniser les notes entre ordinateurs",
    html: `<p>Les notes se synchronisent via <strong>QuickDesk Cloud</strong>. Ni compte ni e-mail : un ordinateur reçoit un <strong>code de synchronisation</strong>, et vos autres ordinateurs le rejoignent avec ce code. Seules les notes se synchronisent ; l’historique du presse-papiers et les réglages restent sur chaque ordinateur.</p>
<h3>La mettre en place</h3>
<ol>
<li>Sur le premier ordinateur : <strong>Réglages → Synchronisation → Activer la synchronisation</strong>, puis choisissez une phrase secrète (au moins 8 caractères).</li>
<li>QuickDesk affiche votre <strong>code de synchronisation</strong> (<code>QD1-…</code>) et une <strong>clé de récupération</strong>. Gardez la clé de récupération en lieu sûr ; elle n’est affichée qu’une fois.</li>
<li>Sur chaque autre ordinateur : <strong>Réglages → Synchronisation → J’ai un code de synchronisation</strong>, puis saisissez le code et la même phrase secrète (ou la clé de récupération).</li>
</ol>
<p>Le code de synchronisation peut être réaffiché à tout moment avec <strong>Afficher le code de synchronisation</strong> dans Réglages → Synchronisation.</p>
<h3>Quand elle a lieu</h3>
<ul>
<li>Quelques secondes après la modification d’une note, et à l’ouverture d’une fenêtre de QuickDesk.</li>
<li>Sinon, QuickDesk cherche des changements chaque minute tant qu’une fenêtre est ouverte, et toutes les 5 minutes quand il est dans la barre. Une vérification est une seule petite requête ; les notes ne transitent que si quelque chose a changé.</li>
<li><strong>Synchroniser</strong> vérifie tout de suite. Les modifications faites hors ligne sont envoyées à votre retour en ligne.</li>
<li>Si une note a changé sur deux ordinateurs en même temps, les deux versions sont conservées : la plus récente reste, l’autre devient une copie à côté. Une modification l’emporte sur une suppression.</li>
</ul>
<h3>Confidentialité et clés</h3>
<ul>
<li>Les notes sont chiffrées sur votre ordinateur avant l’envoi (XChaCha20-Poly1305, clé dérivée de votre phrase secrète par Argon2id). QuickDesk Cloud ne conserve que du texte chiffré sous un identifiant de compte aléatoire et ne peut pas lire vos notes.</li>
<li>Il faut <strong>à la fois</strong> le code de synchronisation (pour atteindre les données) et la phrase secrète (pour les déchiffrer). Traitez le code comme un mot de passe : avec lui seul, quelqu’un pourrait quand même supprimer vos notes synchronisées.</li>
<li>Phrase secrète oubliée ? Saisissez plutôt la clé de récupération. Sans l’une ni l’autre, la copie dans QuickDesk Cloud ne peut pas être ouverte, mais les notes de chaque ordinateur restent intactes.</li>
<li>Si QuickDesk demande de <strong>Déverrouiller la synchronisation</strong> (par exemple après une réinitialisation du trousseau du système), saisissez de nouveau la phrase secrète.</li>
</ul>
<h3>La désactiver</h3>
<p><strong>Désactiver la synchronisation…</strong> arrête la synchronisation sur un ordinateur et garde ses notes. Cochez <em>Supprimer aussi mes notes de QuickDesk Cloud</em> pour supprimer la copie synchronisée pour tous les ordinateurs.</p>
<p>Limites par code de synchronisation : 256 Mo et 20 000 objets stockés, bien plus que ce dont des notes ont besoin.</p>`,
  },
  clipboard: {
    title: "Historique du presse-papiers",
    html: `<p>Tout ce que vous copiez est enregistré : texte, images (captures d’écran) et fichiers copiés dans votre gestionnaire de fichiers. Ouvrez la fenêtre avec <kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd>, tapez pour chercher, filtrez par texte, images ou fichiers, et appuyez sur <kbd>Entrée</kbd> pour coller dans l’app que vous utilisiez. <kbd>Ctrl</kbd> <kbd>Entrée</kbd> copie seulement, <kbd>Ctrl</kbd> <kbd>P</kbd> épingle, <kbd>Suppr</kbd> supprime.</p>
<ul>
<li>L’historique reste sur cet ordinateur et ne se synchronise jamais.</li>
<li>Les copies que les gestionnaires de mots de passe marquent comme secrètes sont ignorées. Avant de copier quelque chose de sensible, <strong>suspendez l’enregistrement</strong> 15 minutes, une heure ou jusqu’à ce que vous le réactiviez (dans l’onglet Presse-papiers ou le menu de la barre).</li>
<li>QuickDesk garde les 1 000 dernières entrées ; les épinglées pour toujours. Texte jusqu’à 1 Mo, images jusqu’à 10 Mo chacune et 200 Mo au total.</li>
<li><strong>Effacer…</strong> dans l’onglet Presse-papiers supprime tout sauf les entrées épinglées.</li>
</ul>`,
  },
  "auto-paste": {
    title: "Collage automatique",
    html: `<p>Quand vous choisissez une entrée, QuickDesk revient à l’app que vous utilisiez et appuie sur <kbd>Maj</kbd> <kbd>Inser</kbd>, qui fonctionne aussi dans les terminaux. Choisissez la méthode dans <strong>Réglages → Collage automatique</strong> :</p>
<ul>
<li><strong>Portail du bureau</strong> (par défaut) : GNOME demande la permission une fois et affiche un indicateur de contrôle à distance pendant le collage.</li>
<li><strong>Instantané (clavier virtuel)</strong> : ni demande ni indicateur. L’activer demande une fois votre mot de passe administrateur pour autoriser les programmes lancés en votre nom à créer un clavier virtuel (la même autorisation que celle utilisée par Steam pour les manettes).</li>
</ul>
<p>Désactivez le collage automatique dans l’onglet Presse-papiers pour seulement copier, puis appuyez vous-même sur <kbd>Ctrl</kbd> <kbd>V</kbd>.</p>`,
  },
  ports: {
    title: "Ports",
    html: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> liste chaque port TCP en écoute (IPv4 et IPv6) avec le processus, sa ligne de commande complète et son utilisateur, actualisés toutes les 2 secondes. Filtrez par port, PID, processus ou nom de conteneur.</p>
<ul>
<li><strong>Ouvrir</strong> ouvre <code>http://localhost:&lt;port&gt;</code> dans votre navigateur ; <strong>Copier le PID</strong> et <strong>Copier la commande</strong> les copient.</li>
<li><strong>Tuer</strong> demande d’abord, puis arrête le processus poliment et ne propose de le forcer que s’il ne s’arrête pas.</li>
<li>Les ports publiés par Docker affichent le conteneur et l’image, et <strong>Arrêter</strong> arrête le conteneur au lieu de tuer <code>docker-proxy</code>.</li>
<li>Les processus d’autres utilisateurs (services du système) apparaissent masqués ; les examiner demande les droits root.</li>
</ul>`,
  },
  versions: {
    title: "Versions des langages (Runtimes)",
    html: `<p>L’onglet <strong>Runtimes</strong> gère les versions des langages de programmation de cet ordinateur (Node.js, Python, Rust, Go, Java, Ruby, Deno, Bun, PHP). Il montre la version qu’exécute un <em>nouveau terminal</em> et d’où elle vient. QuickDesk interroge votre shell de connexion : c’est donc ce que vous obtenez après <code>~/.bashrc</code> ou <code>~/.zshrc</code>, et non ce que voit le bureau.</p>
<ul>
<li>Les langages installés avec <strong>nvm</strong> (Node.js), <strong>rustup</strong> (Rust), <strong>uv</strong> (Python) ou <strong>mise</strong> (Go, Java, Ruby, Deno, Bun, PHP et les autres) peuvent être modifiés : <strong>Installer une version…</strong> choisit parmi les versions que propose l’outil (rustup accepte un canal ou une version que vous tapez), <strong>Définir par défaut</strong> définit ce qu’utilisent les nouveaux terminaux, <strong>Désinstaller</strong> en supprime une et libère sa place. La version par défaut ne peut pas être supprimée ; choisissez d’abord une autre version par défaut.</li>
<li>Les installations tournent en arrière-plan avec une barre de progression et peuvent être annulées. Un seul changement s’exécute à la fois. La liste des versions montre par défaut la dernière version de chaque branche ; passez aux LTS ou à tout, ou tapez pour filtrer.</li>
<li>Quand une version plus récente d’une branche installée sort (Node 22.24.1 pour 22.23.3, Python 3.13.9 pour 3.13.7), la version affiche <strong>↑ 22.24.1</strong> : un clic l’installe et, si l’ancienne était la version par défaut, la rend par défaut. L’ancienne reste jusqu’à ce que vous la désinstalliez. Les canaux Rust sont mis à jour sur place : <strong>stable</strong> affiche <strong>↑ 1.93.0</strong> quand rustup a un stable plus récent, et le clic lance <code>rustup update stable</code>.</li>
<li><strong>Appliquer tout de suite dans les terminaux ouverts</strong> (en haut de l’onglet) fait arriver les changements dans les terminaux déjà ouverts, dès leur prochaine invite : QuickDesk ajoute une ligne à votre fichier de shell qui charge un petit hook d’invite (<code>~/.config/quickdesk/shell/hook.sh</code>). Le hook fait rechercher les commandes au shell après que QuickDesk a changé une version, et fait suivre à Node le <code>.nvmrc</code> le plus proche quand vous faites <code>cd</code> dans un projet, sinon la version par défaut que vous choisissez (un <code>nvm use</code> manuel tient jusqu’au changement de dossier ou de version par défaut). Sans lui, les changements de Rust s’appliquent quand même tout de suite, ceux de Python et de mise arrivent dans les nouveaux terminaux (ou dans un terminal ouvert après <code>hash -r</code>), et ceux de nvm dans les nouveaux terminaux seulement.</li>
<li><strong>Définir pour un projet…</strong> écrit le fichier propre à l’outil dans le dossier de votre choix : <code>.nvmrc</code>, <code>rust-toolchain.toml</code>, <code>.python-version</code> ou <code>mise.toml</code> (mise garde les autres outils déjà listés).</li>
<li>Un langage sans gestionnaire de versions (Go installé par apt, par exemple) propose <strong>Installer mise…</strong> : QuickDesk télécharge le dernier mise depuis github.com/jdx/mise, le vérifie avec les sommes SHA-256 de la version et le place dans <code>~/.local/bin</code>. Une fois une version rendue par défaut, <strong>Corriger le PATH…</strong> propose la ligne qui active mise dans votre shell.</li>
<li>Si les nouveaux terminaux devaient exécuter une autre copie que celle par défaut (par exemple le <code>python3</code> du système arrive en premier dans le PATH), la carte explique pourquoi et <strong>Corriger le PATH…</strong> montre exactement les lignes qu’il ajouterait. Ce n’est qu’après votre accord que QuickDesk les ajoute sous forme de bloc balisé à la fin de votre fichier de shell, avec une sauvegarde à côté ; le bloc peut être retiré au même endroit.</li>
<li>Les corrections du PATH et le hook d’invite sont écrits pour bash (<code>~/.bashrc</code>) et zsh (<code>~/.zshrc</code>) ; fish n’est pas encore pris en charge.</li>
<li>Les langages qui ne sont pas encore sur l’ordinateur sont listés sous <strong>Ajouter un langage</strong> : choisissez-en un pour installer mise si besoin, puis une version.</li>
<li><strong>Windows :</strong> QuickDesk demande à PowerShell ce qu’exécute un nouveau terminal et passe par <strong>mise</strong> (installé dans <code>%LOCALAPPDATA%\\mise\\bin</code>), <strong>rustup</strong> et <strong>uv</strong> ; nvm-windows et fnm sont affichés mais pas modifiés, et le <code>python.exe</code> factice du Store est ignoré. <strong>Corriger le PATH…</strong> met les dossiers de l’outil (pour mise, ses shims) en tête du PATH de votre compte, celui de Paramètres → Variables d’environnement, sans droits d’administrateur ; l’ancienne valeur est gardée dans <code>~\\.config\\quickdesk\\path</code> et le même panneau retire de nouveau les dossiers. Les nouveaux terminaux la prennent en compte ; les changements de version par défaut s’appliquent ensuite tout de suite. « Appliquer tout de suite dans les terminaux ouverts » n’est pas nécessaire sous Windows et n’y est pas affiché.</li>
<li>Les copies installées par le système (apt, dnf) ne sont jamais modifiées ni supprimées ; QuickDesk ne lance jamais <code>sudo</code>. Les langages gérés par des outils qu’il ne prend pas en charge (fnm, volta, pyenv, SDKMAN…) sont affichés en lecture seule.</li>
</ul>`,
  },
  ai: {
    title: "Utilisation de l’IA",
    html: `<p>Un anneau dans la barre supérieure montre quelle part de vos limites d’IA vous avez utilisée, en vert, orange ou rouge. Son menu montre l’outil que vous suivez, avec une barre par limite et l’heure de réinitialisation ; l’onglet AI montre tous les outils.</p>
<ul>
<li><strong>Claude Code :</strong> QuickDesk lit la connexion que Claude Code a déjà enregistrée sur cet ordinateur et demande votre utilisation à Anthropic, au plus toutes les 5 minutes. Le jeton est seulement lu : jamais stocké, renouvelé ni envoyé ailleurs qu’à Anthropic.</li>
<li><strong>Codex :</strong> lu dans les journaux de session que Codex garde dans <code>~/.codex</code>.</li>
<li><strong>Gemini (Antigravity) :</strong> il n’indique pas de pourcentage, QuickDesk montre donc, d’après ses journaux, quand vous avez épuisé votre quota pour la dernière fois.</li>
</ul>
<p>Choisissez l’outil et la limite à suivre dans <em>Suivre</em>, ou masquez l’anneau dans l’onglet AI.</p>`,
  },
  updates: {
    title: "Mises à jour",
    html: `<p>QuickDesk cherche une nouvelle version 30 secondes après son démarrage, puis toutes les 6 heures. Quand il y en a une, un bandeau dans la fenêtre principale et une entrée en haut du menu de la barre le signalent, avec <strong>Nouveautés</strong>. Rien n’est installé tant que vous ne cliquez pas sur <strong>Mettre à jour</strong>.</p>
<p>Chaque mise à jour est vérifiée avec la clé de signature de QuickDesk avant d’être installée. Un .deb ou un .rpm demande votre mot de passe administrateur ; une AppImage se remplace elle-même. QuickDesk redémarre ensuite et montre ce qui a changé. Désactivez les vérifications automatiques, ou vérifiez à la main, dans <strong>Réglages → Mises à jour</strong>.</p>`,
  },
  uninstall: {
    title: "Désinstallation",
    html: `<ol>
<li>Désactivez <em>Lancer QuickDesk à l’ouverture de session</em> dans Réglages, puis quittez depuis la barre.</li>
<li>Supprimez le paquet : <code>sudo apt remove quick-desk</code> (Debian/Ubuntu) ou <code>sudo dnf remove QuickDesk</code> (Fedora), ou supprimez l’AppImage.</li>
<li>Vos données restent dans <code>~/.local/share/click.quickdesk</code> jusqu’à ce que vous supprimiez ce dossier.</li>
</ol>
<p><strong>Windows :</strong> Paramètres → Applications → Applications installées → QuickDesk → Désinstaller. Vos données restent dans <code>%APPDATA%\\click.quickdesk</code> jusqu’à ce que vous supprimiez ce dossier.</p>
<p>Supprimer le paquet retire aussi l’autorisation de collage instantané, si vous l’aviez activée.</p>`,
  },
  troubleshooting: {
    title: "Dépannage",
    html: `<ul>
<li><strong>Un raccourci ne fait rien :</strong> vérifiez s’il y a un conflit dans <em>Paramètres → Clavier → Raccourcis personnalisés</em>, ou choisissez une autre combinaison dans Réglages → Raccourcis.</li>
<li><strong>Le collage automatique n’écrit rien :</strong> vérifiez que l’app où vous étiez a toujours le focus ; changez de méthode dans Réglages → Collage automatique. L’entrée est toujours copiée, donc <kbd>Ctrl</kbd> <kbd>V</kbd> fonctionne.</li>
<li><strong>Journaux :</strong> <code>~/.local/share/click.quickdesk/logs/</code>. Joignez-les quand vous <a href="https://github.com/TankGum/quickdesk/issues">signalez un problème</a>.</li>
</ul>`,
  },
};

export default docs;
