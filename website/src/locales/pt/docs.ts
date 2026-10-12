// Docs in Brazilian Portuguese, by section id (src/data/docs.ts).
import { macBeta } from "../../data/release";

const docs: Record<string, { title: string; html: string }> = {
  install: {
    title: "Instalação",
    html: `<p>O jeito mais rápido, em qualquer distribuição:</p>
<pre><code>curl -fsSL https://quickdesk.click/install.sh | bash</code></pre>
<p>Ele escolhe o .deb, .rpm ou AppImage para o seu sistema, confere o SHA-256 e instala (<a href="/install.sh">ler o script</a>). Ou pegue um pacote na <a href="/download">página de download</a> e instale o arquivo baixado com:</p>
<ul>
<li><strong>Ubuntu, Debian, Mint, Pop!_OS:</strong> <code>sudo apt install ~/Downloads/QuickDesk_&lt;version&gt;_amd64.deb</code></li>
<li><strong>Fedora, openSUSE:</strong> <code>sudo dnf install ~/Downloads/QuickDesk-&lt;version&gt;-1.x86_64.rpm</code></li>
<li><strong>Qualquer outra:</strong> torne o AppImage executável e abra. Ele precisa do FUSE 2 (<code>libfuse2</code>; no Ubuntu 24.04, <code>libfuse2t64</code>).</li>
</ul>
<p>O QuickDesk precisa de um sistema x86_64 dos últimos anos (glibc 2.34 ou mais recente). Funciona melhor no GNOME, tanto no Wayland quanto no X11.</p>
<p><strong>Windows 10 e 11 (64 bits):</strong> no PowerShell, execute <code>irm https://quickdesk.click/install.ps1 | iex</code>, ou baixe o instalador na <a href="/download">página de download</a>. Ele instala só para o seu usuário, sem permissão de administrador. O instalador ainda não é assinado, então o Windows pode dizer que protegeu seu PC: escolha <em>Mais informações → Executar assim mesmo</em>. No Windows os atalhos são <kbd>Win</kbd> <kbd>Alt</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd>, o QuickDesk fica na bandeja do sistema e a colagem automática não precisa de configuração. Lá a aba Runtimes funciona com mise, rustup e uv (veja <em>Versões de linguagens</em>).</p>
<p>Na primeira vez, uma tela de boas-vindas mostra os atalhos e oferece duas opções: <strong>iniciar ao entrar na sessão</strong> (necessário para que o histórico da área de transferência e o anel de IA funcionem sempre) e <strong>colagem instantânea</strong> (veja <a href="#auto-paste">Colagem automática</a>). Depois disso, o QuickDesk fica na barra superior.</p>`,
  },
  macos: {
    title: "macOS",
    html: `${macBeta ? '<p class="note"><strong>Ainda em desenvolvimento.</strong> A versão para Mac funciona, mas é nova: ainda não foi tudo testado em todos os Macs, e a colagem automática e os atalhos continuam sendo aprimorados. Por favor, <a href="https://github.com/TankGum/quickdesk/issues">conte para a gente</a> o que não funcionar.</p>\n' : ""}<p>O QuickDesk funciona no macOS 11 ou mais recente no Apple Silicon (M1 ou mais recente). No Terminal, execute <code>curl -fsSL https://quickdesk.click/install.sh | bash</code>: ele coloca o QuickDesk.app em Aplicativos e o abre. Ou baixe o <code>.dmg</code> na <a href="/download#macos">página de download</a> e arraste o QuickDesk para Aplicativos.</p>
<p><strong>Abrir pela primeira vez.</strong> O QuickDesk ainda não é notarizado pela Apple, então, se o <code>.dmg</code> foi baixado pelo navegador, o macOS interrompe a primeira abertura com o aviso de que “a Apple não pôde verificar se ele está livre de malware”. Não há nada de errado com o app; faça uma destas duas coisas uma vez (se você instalou com o comando, nenhum aviso aparece):</p>
<ol>
<li>Abra o QuickDesk em Aplicativos e feche o aviso do macOS. Depois abra Ajustes do Sistema → Privacidade e Segurança, role até Segurança, clique em <em>Abrir Mesmo Assim</em> ao lado de “O “QuickDesk” foi bloqueado para proteger o seu Mac”, confirme com sua senha ou Touch ID e clique em <em>Abrir</em>. No macOS 14 ou anterior, também funciona clicar com Control no QuickDesk → <em>Abrir</em> → <em>Abrir</em>.</li>
<li>Ou execute <code>xattr -dr com.apple.quarantine /Applications/QuickDesk.app</code> no Terminal e abra o QuickDesk normalmente.</li>
</ol>
<p>A <a href="/download#macos-open">página de download</a> mostra os mesmos passos.</p>
<ul>
<li>O QuickDesk fica na barra de menus, sem ícone no Dock. As janelas pop-up abrem no Espaço em que você está, inclusive por cima de um app em tela cheia.</li>
<li>Os atalhos são <kbd>⌃</kbd> <kbd>⌥</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd> (Control + Opção): ⌘⌥V e ⌘⌥N pertencem ao Finder.</li>
<li>A <strong>colagem automática</strong> pressiona ⌘V no app que você estava usando, e o macOS só permite isso depois que você adicionar o QuickDesk em Ajustes do Sistema → Privacidade e Segurança → <strong>Acessibilidade</strong>. O macOS pergunta na primeira vez que você escolhe um item; até lá o item é copiado e você pressiona ⌘V. Configurações → Colar automaticamente mostra se está permitido.</li>
<li>Versões recentes do macOS podem perguntar se o QuickDesk pode ler o que outros apps copiam. Escolha Permitir, ou o histórico da área de transferência ficará vazio.</li>
<li><strong>Uso do Claude:</strong> no Mac, o Claude Code guarda o login no Chaveiro. A aba AI oferece lê-lo de lá; o macOS então pergunta uma vez (escolha <em>Sempre Permitir</em>). Como nos outros sistemas, o login só é lido e só é enviado para <code>api.anthropic.com</code>.</li>
<li>Como o app tem assinatura ad hoc, o macOS pode pedir de novo as permissões de Acessibilidade e do Chaveiro depois de uma atualização. Se a colagem automática parar de funcionar, remova o QuickDesk da lista de Acessibilidade e adicione de novo.</li>
<li>A aba Runtimes funciona como no Linux, com o seu shell de login (zsh: <code>~/.zshrc</code>). Cópias instaladas pelo Homebrew aparecem, mas não são alteradas.</li>
</ul>
<p><strong>Desinstalar:</strong> saia do QuickDesk pela barra de menus e mova-o de Aplicativos para a Lixeira. Seus dados ficam em <code>~/Library/Application Support/click.quickdesk</code> até você apagar essa pasta.</p>`,
  },
  shortcuts: {
    title: "Atalhos",
    html: `<table>
<tr><th>Atalho</th><th>Abre</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>Notas</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>Janela do histórico da área de transferência</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>Portas</td></tr>
<tr><td>sem atalho</td><td>Janela de nota rápida (defina um em Configurações)</td></tr>
</table>
<p>Pressionar de novo um atalho com a janela dele aberta fecha a janela. Mude qualquer um em <strong>Configurações → Atalhos</strong>; o QuickDesk avisa quando uma combinação já está em uso.</p>
<p>No GNOME os atalhos são adicionados em <em>Configurações → Teclado → Atalhos personalizados</em> e removidos quando o QuickDesk fecha, então também funcionam no Wayland. Em outros ambientes, associe <code>quickdesk toggle notes</code>, <code>quickdesk toggle clipboard</code>, <code>quickdesk toggle ports</code> ou <code>quickdesk toggle quick-note</code> a teclas nas configurações de teclado do seu ambiente.</p>
<p><strong>Por que Super + Alt?</strong> Métodos de entrada vietnamitas, como Unikey e Bamboo, engolem Super + Shift + letra enquanto você digita em um campo de texto; as combinações com Alt continuam funcionando.</p>`,
  },
  notes: {
    title: "Notas",
    html: `<p>As notas são Markdown com um editor rico: títulos, negrito e itálico, listas, listas de tarefas, citações e blocos de código. <kbd>Ctrl</kbd> <kbd>N</kbd> cria uma nota nova, <kbd>Ctrl</kbd> <kbd>S</kbd> salva, <kbd>Ctrl</kbd> <kbd>F</kbd> pesquisa. A pesquisa ignora acentos, então <code>tieng viet</code> encontra “tiếng Việt”.</p>
<p>Fixe uma nota com a estrela para mantê-la no topo. Apagar pede confirmação e pode ser desfeito por alguns segundos.</p>`,
  },
  sync: {
    title: "Sincronizar notas entre computadores",
    html: `<p>As notas sincronizam pelo <strong>QuickDesk Cloud</strong>. Não há conta nem e-mail: um computador recebe um <strong>código de sincronização</strong> e seus outros computadores entram com ele. Só as notas sincronizam; o histórico da área de transferência e as configurações ficam em cada computador.</p>
<h3>Configurar</h3>
<ol>
<li>No primeiro computador: <strong>Configurações → Sincronização → Ativar a sincronização</strong>, e escolha uma frase secreta (pelo menos 8 caracteres).</li>
<li>O QuickDesk mostra seu <strong>código de sincronização</strong> (<code>QD1-…</code>) e uma <strong>chave de recuperação</strong>. Guarde a chave de recuperação em um lugar seguro; ela só aparece uma vez.</li>
<li>Em cada um dos outros computadores: <strong>Configurações → Sincronização → Tenho um código de sincronização</strong>, e digite o código e a mesma frase secreta (ou a chave de recuperação).</li>
</ol>
<p>Você pode ver o código de novo a qualquer momento com <strong>Mostrar o código de sincronização</strong> em Configurações → Sincronização.</p>
<h3>Quando sincroniza</h3>
<ul>
<li>Alguns segundos depois de você editar uma nota, e quando você abre uma janela do QuickDesk.</li>
<li>Fora isso, o QuickDesk procura mudanças a cada minuto enquanto uma janela está aberta, e a cada 5 minutos enquanto está na bandeja. Cada verificação é uma única requisição pequena; as notas só trafegam quando algo mudou.</li>
<li><strong>Sincronizar agora</strong> verifica na hora. Mudanças feitas offline são enviadas quando você volta a ficar online.</li>
<li>Se uma nota mudou em dois computadores ao mesmo tempo, as duas versões são mantidas: a mais recente fica e a outra vira uma cópia ao lado. Uma edição vence uma exclusão.</li>
</ul>
<h3>Privacidade e chaves</h3>
<ul>
<li>As notas são criptografadas no seu computador antes do envio (XChaCha20-Poly1305, chave derivada da sua frase secreta com Argon2id). O QuickDesk Cloud só guarda texto cifrado sob um identificador de conta aleatório e não consegue ler suas notas.</li>
<li>São necessárias <strong>as duas coisas</strong>: o código de sincronização (para chegar aos dados) e a frase secreta (para decifrá-los). Trate o código como uma senha: só com ele, alguém poderia apagar suas notas sincronizadas.</li>
<li>Esqueceu a frase secreta? Digite a chave de recuperação. Sem nenhuma das duas, a cópia no QuickDesk Cloud não pode ser aberta, mas as notas em cada computador não são afetadas.</li>
<li>Se o QuickDesk pedir <strong>Desbloquear a sincronização</strong> (por exemplo, depois de redefinir o chaveiro do sistema), digite a frase secreta de novo.</li>
</ul>
<h3>Desativar</h3>
<p><strong>Desativar a sincronização…</strong> para de sincronizar em um computador e mantém as notas dele. Marque <em>Excluir também minhas notas do QuickDesk Cloud</em> para apagar a cópia sincronizada para todos os computadores.</p>
<p>Limites por código de sincronização: 256 MB e 20.000 objetos armazenados, muito mais do que notas precisam.</p>`,
  },
  clipboard: {
    title: "Histórico da área de transferência",
    html: `<p>Tudo o que você copia é guardado: texto, imagens (capturas de tela) e arquivos copiados no seu gerenciador de arquivos. Abra a janela com <kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd>, digite para pesquisar, filtre por texto, imagens ou arquivos e pressione <kbd>Enter</kbd> para colar no app que você estava usando. <kbd>Ctrl</kbd> <kbd>Enter</kbd> só copia, <kbd>Ctrl</kbd> <kbd>P</kbd> fixa, <kbd>Delete</kbd> apaga.</p>
<ul>
<li>O histórico fica neste computador e nunca é sincronizado.</li>
<li>O que os gerenciadores de senhas marcam como secreto é ignorado. Antes de copiar algo sensível, <strong>pause o salvamento</strong> por 15 minutos, uma hora ou até reativar (na aba Área de transferência ou no menu da bandeja).</li>
<li>O QuickDesk guarda os 1.000 itens mais recentes; os fixados, para sempre. Texto de até 1 MB, imagens de até 10 MB cada e 200 MB no total.</li>
<li><strong>Limpar…</strong> na aba Área de transferência apaga tudo, menos os itens fixados.</li>
</ul>`,
  },
  "auto-paste": {
    title: "Colagem automática",
    html: `<p>Quando você escolhe um item, o QuickDesk volta para o app que você estava usando e pressiona <kbd>Shift</kbd> <kbd>Insert</kbd>, que também funciona em terminais. Escolha como em <strong>Configurações → Colar automaticamente</strong>:</p>
<ul>
<li><strong>Portal da área de trabalho</strong> (padrão): o GNOME pede permissão uma vez e mostra um indicador de controle remoto enquanto cola.</li>
<li><strong>Instantâneo (teclado virtual)</strong>: sem avisos nem indicador. Ao ativar, pede sua senha de administrador uma vez para permitir que programas executados por você criem um teclado virtual (a mesma permissão que o Steam usa para controles).</li>
</ul>
<p>Desative a colagem automática na aba Área de transferência para apenas copiar, e então pressione <kbd>Ctrl</kbd> <kbd>V</kbd> você mesmo.</p>`,
  },
  ports: {
    title: "Portas",
    html: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> mostra cada porta TCP escutando (IPv4 e IPv6) com o processo, a linha de comando completa e o usuário, atualizada a cada 2 segundos. Filtre por porta, PID, processo ou nome de contêiner.</p>
<ul>
<li><strong>Abrir</strong> abre <code>http://localhost:&lt;port&gt;</code> no seu navegador; <strong>Copiar PID</strong> e <strong>Copiar comando</strong> copiam esses dados.</li>
<li><strong>Encerrar</strong> pergunta antes, depois encerra o processo com educação e só oferece forçar se ele não terminar.</li>
<li>Portas publicadas pelo Docker mostram o contêiner e a imagem, e <strong>Parar</strong> para o contêiner em vez de matar o <code>docker-proxy</code>.</li>
<li>Processos de outros usuários (serviços do sistema) aparecem ocultos; inspecioná-los exige root.</li>
</ul>`,
  },
  versions: {
    title: "Versões de linguagens (Runtimes)",
    html: `<p>A aba <strong>Runtimes</strong> gerencia as versões das linguagens de programação deste computador (Node.js, Python, Rust, Go, Java, Ruby, Deno, Bun, PHP). Ela mostra a versão que um <em>terminal novo</em> executa e de onde vem. O QuickDesk pergunta ao seu shell de login, então isso é o que você obtém depois do <code>~/.bashrc</code> ou <code>~/.zshrc</code>, não o que a área de trabalho vê.</p>
<ul>
<li>As linguagens instaladas com <strong>nvm</strong> (Node.js), <strong>rustup</strong> (Rust), <strong>uv</strong> (Python) ou <strong>mise</strong> (Go, Java, Ruby, Deno, Bun, PHP e as demais) podem ser alteradas: <strong>Instalar versão…</strong> escolhe entre as versões que a ferramenta oferece (o rustup aceita um canal ou uma versão digitada), <strong>Tornar padrão</strong> define o que os terminais novos usam, <strong>Desinstalar</strong> remove uma e libera o espaço. A padrão não pode ser removida; escolha outra padrão antes.</li>
<li>As instalações rodam em segundo plano com uma barra de progresso e podem ser canceladas. Só uma mudança roda por vez. A lista de versões mostra por padrão a versão mais recente de cada série; troque para LTS ou todas, ou digite para filtrar.</li>
<li>Quando sai uma versão mais nova de uma série instalada (Node 22.24.1 para 22.23.3, Python 3.13.9 para 3.13.7), a versão mostra <strong>↑ 22.24.1</strong>: um clique instala e, se a antiga era a padrão, torna a nova a padrão. A antiga fica até você desinstalá-la. Os canais do Rust são atualizados no lugar: <strong>stable</strong> mostra <strong>↑ 1.93.0</strong> quando o rustup tem um stable mais novo, e o clique executa <code>rustup update stable</code>.</li>
<li><strong>Aplicar na hora nos terminais abertos</strong> (no topo da aba) faz as mudanças chegarem aos terminais já abertos, a partir do próximo prompt: o QuickDesk adiciona uma linha ao seu arquivo de shell que carrega um pequeno hook do prompt (<code>~/.config/quickdesk/shell/hook.sh</code>). O hook faz o shell procurar os comandos de novo depois que o QuickDesk muda uma versão, e faz o Node seguir o <code>.nvmrc</code> mais próximo quando você entra com <code>cd</code> em um projeto, ou senão a padrão que você escolher (um <code>nvm use</code> manual vale até você mudar de pasta ou de padrão). Sem ele, as mudanças do Rust ainda valem na hora, as do Python e do mise chegam aos terminais novos (ou a um aberto depois de <code>hash -r</code>) e as do nvm só aos terminais novos.</li>
<li><strong>Definir para um projeto…</strong> escreve o arquivo próprio da ferramenta na pasta que você escolher: <code>.nvmrc</code>, <code>rust-toolchain.toml</code>, <code>.python-version</code> ou <code>mise.toml</code> (o mise mantém as outras ferramentas que já estiverem lá).</li>
<li>Uma linguagem sem gerenciador de versões (por exemplo, Go instalado com apt) oferece <strong>Instalar o mise…</strong>: o QuickDesk baixa o mise mais recente de github.com/jdx/mise, confere com as somas SHA-256 da versão e o coloca em <code>~/.local/bin</code>. Depois que você tornar uma versão padrão, <strong>Corrigir o PATH…</strong> oferece a linha que ativa o mise no seu shell.</li>
<li>Se os terminais novos fossem executar uma cópia diferente da padrão (por exemplo, o <code>python3</code> do sistema vem primeiro no PATH), o cartão explica por quê e <strong>Corrigir o PATH…</strong> mostra exatamente as linhas que adicionaria. Só quando você aceita, o QuickDesk as adiciona como um bloco marcado no fim do seu arquivo de shell, com um backup ao lado; o bloco pode ser removido no mesmo lugar.</li>
<li>Os ajustes de PATH e o hook do prompt são escritos para bash (<code>~/.bashrc</code>) e zsh (<code>~/.zshrc</code>); o fish ainda não é suportado.</li>
<li>As linguagens que ainda não estão no computador aparecem em <strong>Adicionar uma linguagem</strong>: escolha uma para instalar o mise, se preciso, e depois uma versão.</li>
<li><strong>Windows:</strong> o QuickDesk pergunta ao PowerShell o que um terminal novo executa e trabalha com <strong>mise</strong> (instalado em <code>%LOCALAPPDATA%\\mise\\bin</code>), <strong>rustup</strong> e <strong>uv</strong>; o nvm-windows e o fnm aparecem, mas não são alterados, e o <code>python.exe</code> de espaço reservado da Microsoft Store é ignorado. <strong>Corrigir o PATH…</strong> coloca as pastas da ferramenta (para o mise, os shims) no início do PATH da sua conta, o de Configurações → Variáveis de ambiente, sem permissão de administrador; o valor anterior é salvo em <code>~\\.config\\quickdesk\\path</code> e o mesmo painel remove as pastas de novo. Os terminais novos passam a usá-lo; a partir daí, mudanças de padrão valem na hora. No Windows o “Aplicar na hora nos terminais abertos” não é necessário e não aparece.</li>
<li>Cópias instaladas pelo sistema (apt, dnf) nunca são alteradas nem removidas; o QuickDesk nunca executa <code>sudo</code>. Linguagens gerenciadas por ferramentas que ele não suporta (fnm, volta, pyenv, SDKMAN…) aparecem só para leitura.</li>
</ul>`,
  },
  ai: {
    title: "Uso de IA",
    html: `<p>Um anel na barra superior mostra quanto você já usou dos seus limites de IA, em verde, âmbar ou vermelho. O menu dele mostra a ferramenta que você acompanha, com uma barra para cada limite e quando ele reinicia; a aba AI mostra todas as ferramentas.</p>
<ul>
<li><strong>Claude Code:</strong> o QuickDesk lê o login que o Claude Code já salvou neste computador e pergunta à Anthropic sobre o seu uso, no máximo a cada 5 minutos. O token só é lido: nunca é salvo, renovado nem enviado a lugar nenhum além da Anthropic.</li>
<li><strong>Codex:</strong> lido dos logs de sessão que o Codex guarda em <code>~/.codex</code>.</li>
<li><strong>Gemini (Antigravity):</strong> não informa uma porcentagem, então o QuickDesk mostra, a partir dos logs dele, quando você esgotou a cota pela última vez.</li>
</ul>
<p>Escolha qual ferramenta e qual limite acompanhar em <em>Acompanhar</em>, ou esconda o anel na aba AI.</p>`,
  },
  updates: {
    title: "Atualizações",
    html: `<p>O QuickDesk procura uma versão nova 30 segundos depois de iniciar e depois a cada 6 horas. Quando sai uma, um aviso na janela principal e um item no topo do menu da bandeja informam, com <strong>Novidades</strong>. Nada é instalado até você clicar em <strong>Atualizar agora</strong>.</p>
<p>Cada atualização é verificada com a chave de assinatura do QuickDesk antes de ser instalada. Um .deb ou .rpm pede sua senha de administrador; um AppImage se substitui sozinho. Depois o QuickDesk reinicia e mostra o que mudou. Desative as verificações automáticas, ou verifique manualmente, em <strong>Configurações → Atualizações</strong>.</p>`,
  },
  uninstall: {
    title: "Desinstalar",
    html: `<ol>
<li>Desative <em>Iniciar o QuickDesk ao entrar na sessão</em> em Configurações e saia pela bandeja.</li>
<li>Remova o pacote: <code>sudo apt remove quick-desk</code> (Debian/Ubuntu) ou <code>sudo dnf remove QuickDesk</code> (Fedora), ou apague o AppImage.</li>
<li>Seus dados ficam em <code>~/.local/share/click.quickdesk</code> até você apagar essa pasta.</li>
</ol>
<p><strong>Windows:</strong> Configurações → Aplicativos → Aplicativos instalados → QuickDesk → Desinstalar. Seus dados ficam em <code>%APPDATA%\\click.quickdesk</code> até você apagar essa pasta.</p>
<p>Remover o pacote também retira a permissão de colagem instantânea, se você a ativou.</p>`,
  },
  troubleshooting: {
    title: "Solução de problemas",
    html: `<ul>
<li><strong>Um atalho não faz nada:</strong> veja se há conflito em <em>Configurações → Teclado → Atalhos personalizados</em>, ou escolha outra combinação em Configurações → Atalhos.</li>
<li><strong>A colagem automática não digita nada:</strong> confira se o app em que você estava ainda tem o foco; mude o método em Configurações → Colar automaticamente. O item sempre é copiado, então <kbd>Ctrl</kbd> <kbd>V</kbd> funciona.</li>
<li><strong>Logs:</strong> <code>~/.local/share/click.quickdesk/logs/</code>. Anexe-os quando <a href="https://github.com/TankGum/quickdesk/issues">relatar um problema</a>.</li>
</ul>`,
  },
};

export default docs;
