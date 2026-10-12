// Privacy page in Brazilian Portuguese. Keep it in step with src/data/privacy.ts.
const privacy = {
  title: "Privacidade",
  updated: "Última atualização: 11 de outubro de 2026 (contagem de downloads)",
  html: `<p class="lead">O QuickDesk não tem contas, nem análises, nem telemetria. Suas notas e seu histórico da área de transferência ficam no seu computador. Esta página lista tudo o que sai dele.</p>
<h2>O que fica no seu computador</h2>
<ul>
<li><strong>Notas, histórico da área de transferência (texto, imagens, listas de arquivos), configurações e logs</strong>, em <code>~/.local/share/click.quickdesk</code>.</li>
<li><strong>O histórico da área de transferência nunca sai do seu computador</strong>, nem mesmo com a sincronização ativada. O que os gerenciadores de senhas marcam como secreto não é salvo, e você pode pausar o salvamento quando quiser.</li>
<li>As <strong>portas</strong> são lidas de <code>/proc</code> e do Docker na sua máquina e não são salvas.</li>
<li>Seu código de sincronização e sua chave de criptografia ficam no chaveiro do sistema.</li>
</ul>
<h2>O que o QuickDesk envia, e para onde</h2>
<table>
<tr><th>Quando</th><th>Para onde</th><th>O quê</th></tr>
<tr><td data-label="Quando">Ao procurar atualizações (30 s depois de iniciar e depois a cada 6 horas; pode ser desativado)</td><td data-label="Para onde"><code>dl.quickdesk.click</code> (Cloudflare)</td><td data-label="O quê">Uma requisição de <code>update.json</code>, e o pacote quando você escolhe atualizar. Seu endereço IP fica visível para a Cloudflare, como em qualquer download.</td></tr>
<tr><td data-label="Quando">Sincronização de notas, só se você ativar</td><td data-label="Para onde">QuickDesk Cloud, <code>sync.quickdesk.click</code> (Cloudflare)</td><td data-label="O quê">Suas notas, criptografadas no seu computador (XChaCha20-Poly1305) com uma chave derivada da sua frase secreta; o QuickDesk Cloud não consegue lê-las. Ele as guarda sob um identificador aleatório de conta de sincronização, com o tamanho e a hora do envio, e vê seu endereço IP. Nada de e-mail, nome ou outros dados pessoais. Você pode apagar tudo em Configurações → Sincronização.</td></tr>
<tr><td data-label="Quando">Uso de IA, só se o Claude Code estiver conectado neste computador (no Mac, só depois que você permitir ler o login dele no Chaveiro)</td><td data-label="Para onde"><code>api.anthropic.com</code></td><td data-label="O quê">Uma requisição de uso autorizada com o login que o Claude Code já tem, no máximo a cada 5 minutos. O token é lido; nunca é salvo, renovado nem enviado a nenhum outro lugar.</td></tr>
<tr><td data-label="Quando">Aba Runtimes, quando procura versões novas (no máximo uma vez por hora), quando você abre uma lista de versões ou instala uma</td><td data-label="Para onde">Os servidores do nvm, uv, rustup e mise: <code>nodejs.org</code>, <code>github.com</code> / <code>releases.astral.sh</code> (builds do Python), <code>static.rust-lang.org</code> e, para o mise, o site de download de cada linguagem (por exemplo <code>go.dev</code>)</td><td data-label="O quê">Essas ferramentas fazem as requisições elas mesmas, como quando você as executa em um terminal. O QuickDesk não acrescenta nada.</td></tr>
<tr><td data-label="Quando">Aba Runtimes, só se você clicar em “Instalar o mise”</td><td data-label="Para onde"><code>github.com</code> (versões de jdx/mise); no Windows também <code>api.github.com</code></td><td data-label="O quê">Um download do mise mais recente e do arquivo de somas de verificação. No Windows, o QuickDesk antes pergunta à API do GitHub qual é a versão mais recente (User-Agent “QuickDesk”).</td></tr>
<tr><td data-label="Quando">Colagem automática pelo portal da área de trabalho</td><td data-label="Para onde">Sua própria área de trabalho (GNOME), não a rede</td><td data-label="O quê">Um pressionamento de Shift + Insert.</td></tr>
</table>
<p>O uso do Codex e do Antigravity é lido de arquivos de log no seu computador; nada é enviado.</p>
<h2>Este site</h2>
<p>quickdesk.click é um site estático no Cloudflare Pages, sem cookies, sem análises e sem rastreamento. Ele carrega fontes do Google Fonts, então o Google vê seu endereço IP. A demonstração roda inteira no seu navegador com dados de exemplo. O sistema que você escolhe (Linux, Windows ou macOS) e o tema claro ou escuro, se você mudar, ficam guardados no armazenamento local do seu navegador e não são enviados a lugar nenhum.</p>
<h2>Contagem de downloads</h2>
<p>Para saber quantas pessoas instalam e atualizam o QuickDesk, <code>dl.quickdesk.click</code> conta cada download e cada busca por atualizações: qual arquivo (versão e sistema), o tipo de cliente que pediu (um navegador vindo deste site ou de outro lugar, um terminal, o PowerShell ou o atualizador do app) e o país que a Cloudflare informa para a requisição. Só são guardados totais diários (em um banco de dados Cloudflare D1): nada de endereço IP, string do navegador, cookie, outro identificador ou registro por requisição, então as contagens não podem ser ligadas a uma pessoa nem a um computador.</p>
<h2>Perguntas</h2>
<p>O QuickDesk é de código aberto: tudo o que está dito aqui pode ser conferido no <a href="https://github.com/TankGum/quickdesk">código-fonte</a>. Pergunte ou relate um problema nas <a href="https://github.com/TankGum/quickdesk/issues">issues do GitHub</a>.</p>`,
};

export default privacy;
