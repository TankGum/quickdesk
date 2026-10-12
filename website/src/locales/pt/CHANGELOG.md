# Novidades

O que mudou em cada versão do QuickDesk (tradução para o português do Brasil). O original em inglês está em CHANGELOG.md, na raiz do repositório; as versões que faltarem aqui aparecem em inglês.

## 0.7.0 — 2026-10-12

### Novo
- O app fala os idiomas do site: além de inglês e vietnamita, chinês simplificado, japonês, hindi, espanhol, francês, árabe (da direita para a esquerda), bengali, russo, português (Brasil) e indonésio. Escolha um em Configurações → Idioma; “Igual ao sistema” agora também segue o idioma do sistema no macOS e no Windows. Os menus da bandeja e as novidades usam o mesmo idioma.

## 0.6.1 — 2026-10-12

### Corrigido
- macOS: escolher um item na janela da área de transferência volta a colá-lo no campo de texto em que você estava digitando. Agora o QuickDesk sai da frente para que o macOS devolva o foco àquele app e àquele campo, em vez de pedir que o app seja ativado, algo que as versões recentes do macOS podem cumprir só pela metade.

## 0.6.0 — 2026-10-11

### Novo
- QuickDesk para macOS (Apple Silicon, macOS 11 ou mais recente): notas com sincronização, histórico da área de transferência com texto, imagens e arquivos, colagem automática com ⌘V no app que você estava usando, portas, versões de linguagens e uso de IA, pela barra de menus. Os atalhos são ⌃⌥N / V / P. Instale pelo Terminal com `curl -fsSL https://quickdesk.click/install.sh | bash` ou como .dmg pela página de download; ele se atualiza sozinho, como nos outros sistemas. O app ainda não é notarizado pela Apple: se o .dmg foi baixado pelo navegador, o macOS bloqueia a primeira abertura, e a página de download explica como abri-lo (Ajustes do Sistema → Privacidade e Segurança → Abrir Mesmo Assim).
- No Mac, a aba AI também pode mostrar seus limites do Claude: lá o Claude Code guarda o login no Chaveiro, e o QuickDesk só o lê depois que você permitir.

### Corrigido
- O script de instalação diz qual pacote está faltando em uma versão, em vez de parar sem dizer nada.

## 0.5.0 — 2026-10-11

### Novo
- QuickDesk para Windows 10 e 11 (64 bits): notas com sincronização, histórico da área de transferência com texto, imagens e arquivos, colagem automática no app que você estava usando, portas e uso de IA, com os mesmos atalhos Win+Alt+N / V / P. Instale pelo PowerShell com `irm https://quickdesk.click/install.ps1 | iex` ou pela página de download; ele instala para o seu usuário sem permissão de administrador e se atualiza sozinho. A aba Runtimes funciona com mise, rustup e uv: instale o mise com um clique, adicione Node, Python, Go e os demais, e “Corrigir o PATH…” coloca as pastas deles no PATH da sua conta sem permissão de administrador.
- Runtimes: as linguagens que ainda não estão no computador aparecem em “Adicionar uma linguagem”, prontas para instalar com o mise.

### Melhorado
- Runtimes: os canais do Rust também ganharam botão de atualização. Quando o rustup tem um stable (ou beta, nightly) mais novo, o canal mostra ↑ e um clique executa `rustup update` para ele.

## 0.4.0 — 2026-10-10

### Novo
- Uma aba Runtimes para as versões das suas linguagens de programação: veja qual Node.js, Python, Rust, Go… um terminal novo realmente executa e de onde vem, instale e desinstale versões, escolha a padrão e fixe uma versão por projeto. O QuickDesk trabalha com nvm, uv, rustup e mise, e nunca mexe nos pacotes do sistema.
- Quando sai uma versão mais nova de uma série instalada (por exemplo Node 22.24.1 para 22.23.3), a versão mostra um botão de atualização; a antiga fica até você removê-la.
- Linguagens sem gerenciador de versões (por exemplo Go instalado com apt) podem ganhar o mise com um clique, baixado das versões dele no GitHub e conferido com as somas SHA-256.
- “Aplicar na hora nos terminais abertos”: as mudanças chegam aos terminais já abertos no próximo prompt, e o Node segue o `.nvmrc` mais próximo quando você faz `cd`. Quando algo no PATH atrapalha, a aba explica por quê e oferece as linhas exatas para adicionar ao seu arquivo de shell, com um backup.

### Melhorado
- A página de download mostra o comando completo para baixar e instalar a partir de um terminal.
- A lista de versões pode mostrar a versão mais recente de cada série, só as LTS ou todas.

## 0.3.1 — 2026-10-09

### Novo
- A sincronização agora vem embutida: ative em Configurações → Sincronização e conecte seus outros computadores com um código de sincronização. Sem conta, e-mail ou configuração de armazenamento. As notas continuam sendo criptografadas no seu computador antes de saírem dele; o QuickDesk Cloud só guarda texto cifrado.

### Corrigido
- As notas, o histórico da área de transferência e as configurações de versões anteriores estão de volta. Ao atualizar para a 0.2.0 e a 0.3.0, o app começava com uma pasta vazia em vez de trazer seus dados; agora o QuickDesk os traz sozinho e mantém as pastas antigas como backup.
- Abrir Configurações não deixa mais cinzas os pontos coloridos das outras abas.

### Melhorado
- A sincronização pelo seu próprio bucket S3 ou R2 não é mais oferecida. Se você a usava, a sincronização é desativada uma vez, com um aviso em Configurações → Sincronização; suas notas ficam no seu computador.
- O menu de uso de IA na barra superior mostra só a ferramenta que você acompanha, com uma barra verde, amarela ou vermelha para cada limite e a hora de reinício embaixo. Escolha outra ferramenta em Track.

## 0.3.0 — 2026-10-09

### Novo
- O QuickDesk tem seu próprio endereço: [quickdesk.click](https://quickdesk.click). Downloads e atualizações vêm de lá.
- Um site de verdade: uma página de download que sugere o pacote certo, documentação, novidades e página de privacidade, em inglês e vietnamita.

### Melhorado
- Seus dados agora ficam em `~/.local/share/click.quickdesk`. O QuickDesk os move para lá sozinho na primeira vez que inicia; as notas, o histórico da área de transferência e as configurações continuam como estavam.
- O reinício depois de uma atualização está mais confiável.

## 0.2.2 — 2026-10-09

### Novo
- O QuickDesk agora se atualiza sozinho. Ele procura versões novas em segundo plano, conta o que mudou e só instala quando você escolhe. Cada atualização é verificada com a chave de assinatura do QuickDesk antes de ser instalada.
- Configurações → Atualizações: veja sua versão, verifique agora ou desative as verificações automáticas.
- Depois de uma atualização, o QuickDesk mostra as novidades uma vez.
- A página de download está disponível em vietnamita e tem link para o código-fonte no GitHub.

### Melhorado
- Um só ícone em todo lugar: o dock, o menu de aplicativos, as janelas e a bandeja agora compartilham o ícone do QuickDesk.

## 0.2.1 — 2026-10-08

### Novo
- Um visual novo, compartilhado pelo app e pela página de download: uma cor por seção (notas, área de transferência, portas, IA), dicas de teclado nas janelas pop-up e uma barra de abas no estilo do macOS.
- Ícones de linha no lugar de emojis em todo o app.
- Menus da bandeja com um estilo macOS mais limpo: rótulos curtos, uma marca de seleção para “Save Área de transferência History” e os limites de IA agrupados por ferramenta.

### Melhorado
- A demonstração na página de download é o app de verdade rodando com dados de exemplo.

## 0.2.0 — 2026-10-08

### Novo
- Pacotes para Ubuntu/Debian (.deb), Fedora (.rpm) e outras distribuições (AppImage), publicados no site de downloads do QuickDesk.

### Melhorado
- Seus dados são movidos automaticamente para a nova pasta de dados do QuickDesk (`~/.local/share/io.github.tankgum.quickdesk`).
