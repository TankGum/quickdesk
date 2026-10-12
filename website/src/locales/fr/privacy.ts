// Privacy page in French. Keep it in step with src/data/privacy.ts.
const privacy = {
  title: "Confidentialité",
  updated: "Dernière mise à jour : 11 octobre 2026 (comptage des téléchargements)",
  html: `<p class="lead">QuickDesk n’a ni compte, ni statistiques, ni télémétrie. Vos notes et votre historique du presse-papiers vivent sur votre ordinateur. Cette page liste tout ce qui en sort.</p>
<h2>Ce qui reste sur votre ordinateur</h2>
<ul>
<li><strong>Notes, historique du presse-papiers (texte, images, listes de fichiers), réglages et journaux</strong>, dans <code>~/.local/share/click.quickdesk</code>.</li>
<li><strong>L’historique du presse-papiers ne quitte jamais votre ordinateur</strong>, même avec la synchronisation activée. Les copies que les gestionnaires de mots de passe marquent comme secrètes ne sont pas enregistrées, et vous pouvez suspendre l’enregistrement à tout moment.</li>
<li>Les <strong>ports</strong> sont lus dans <code>/proc</code> et Docker sur votre machine et ne sont pas stockés.</li>
<li>Votre code de synchronisation et votre clé de chiffrement sont conservés dans le trousseau du système.</li>
</ul>
<h2>Ce que QuickDesk envoie, et où</h2>
<table>
<tr><th>Quand</th><th>Vers</th><th>Quoi</th></tr>
<tr><td data-label="Quand">Recherche de mises à jour (30 s après le démarrage, puis toutes les 6 heures ; désactivable)</td><td data-label="Vers"><code>dl.quickdesk.click</code> (Cloudflare)</td><td data-label="Quoi">Une requête pour <code>update.json</code>, et le paquet quand vous choisissez de mettre à jour. Votre adresse IP est visible par Cloudflare, comme pour tout téléchargement.</td></tr>
<tr><td data-label="Quand">Synchronisation des notes, seulement si vous l’activez</td><td data-label="Vers">QuickDesk Cloud, <code>sync.quickdesk.click</code> (Cloudflare)</td><td data-label="Quoi">Vos notes, chiffrées sur votre ordinateur (XChaCha20-Poly1305) avec une clé dérivée de votre phrase secrète ; QuickDesk Cloud ne peut pas les lire. Il les stocke sous un identifiant de compte de synchronisation aléatoire, avec leur taille et leur heure d’envoi, et voit votre adresse IP. Ni e-mail, ni nom, ni autre donnée personnelle. Vous pouvez tout supprimer depuis Réglages → Synchronisation.</td></tr>
<tr><td data-label="Quand">Utilisation de l’IA, seulement si Claude Code est connecté sur cet ordinateur (sur Mac, seulement après avoir autorisé la lecture de sa connexion dans le Trousseau)</td><td data-label="Vers"><code>api.anthropic.com</code></td><td data-label="Quoi">Une requête d’utilisation autorisée avec la connexion existante de Claude Code, au plus toutes les 5 minutes. Le jeton est lu, jamais stocké, renouvelé ni envoyé ailleurs.</td></tr>
<tr><td data-label="Quand">Onglet Runtimes, quand il cherche de nouvelles versions (au plus une fois par heure), quand vous ouvrez une liste de versions ou en installez une</td><td data-label="Vers">Les serveurs de nvm, uv, rustup et mise : <code>nodejs.org</code>, <code>github.com</code> / <code>releases.astral.sh</code> (compilations de Python), <code>static.rust-lang.org</code> et, pour mise, le site de téléchargement de chaque langage (par exemple <code>go.dev</code>)</td><td data-label="Quoi">Ces outils font eux-mêmes les requêtes, comme quand vous les lancez dans un terminal. QuickDesk n’y ajoute rien.</td></tr>
<tr><td data-label="Quand">Onglet Runtimes, seulement si vous cliquez sur « Installer mise »</td><td data-label="Vers"><code>github.com</code> (versions de jdx/mise) ; sous Windows aussi <code>api.github.com</code></td><td data-label="Quoi">Un téléchargement du dernier mise et de son fichier de sommes de contrôle. Sous Windows, QuickDesk demande d’abord à l’API de GitHub quelle est la dernière version (User-Agent « QuickDesk »).</td></tr>
<tr><td data-label="Quand">Collage automatique via le portail de bureau</td><td data-label="Vers">Votre propre bureau (GNOME), pas le réseau</td><td data-label="Quoi">Un appui sur Maj + Inser.</td></tr>
</table>
<p>L’utilisation de Codex et d’Antigravity est lue dans des fichiers journaux de votre ordinateur ; rien n’est envoyé.</p>
<h2>Ce site</h2>
<p>quickdesk.click est un site statique sur Cloudflare Pages, sans cookies, sans statistiques et sans pistage. Il charge des polices depuis Google Fonts, Google voit donc votre adresse IP. La démo tourne entièrement dans votre navigateur, sur des données d’exemple. Le système que vous choisissez (Linux, Windows ou macOS) et le thème clair ou sombre, si vous le changez, sont mémorisés dans le stockage local de votre navigateur et ne sont envoyés nulle part.</p>
<h2>Comptage des téléchargements</h2>
<p>Pour savoir combien de personnes installent et mettent à jour QuickDesk, <code>dl.quickdesk.click</code> compte chaque téléchargement et chaque recherche de mise à jour : quel fichier (sa version et son système), le type de client qui l’a demandé (un navigateur venant de ce site ou d’ailleurs, un terminal, PowerShell ou le système de mise à jour de l’app) et le pays que Cloudflare indique pour la requête. Seuls des totaux quotidiens sont conservés (dans une base de données Cloudflare D1) : ni adresse IP, ni chaîne de navigateur, ni cookie, ni autre identifiant, ni enregistrement par requête, les comptages ne peuvent donc pas être reliés à une personne ou à un ordinateur.</p>
<h2>Questions</h2>
<p>QuickDesk est open source : tout ce qui est écrit ici peut être vérifié dans le <a href="https://github.com/TankGum/quickdesk">code source</a>. Posez vos questions ou signalez un problème dans les <a href="https://github.com/TankGum/quickdesk/issues">GitHub issues</a>.</p>`,
};

export default privacy;
