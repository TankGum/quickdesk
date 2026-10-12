// Português do Brasil (Brazilian Portuguese). See ../en.ts for what each part
// is. The app itself is in English and Vietnamese, so its menu and button
// names stay in English.
import type { Locale } from "..";
import changelog from "./CHANGELOG.md?raw";
import docs from "./docs";
import home from "./home";
import privacy from "./privacy";

const pt: Locale = {
  strings: {
    ui: {
      home: "Início",
      download: "Baixar",
      docs: "Documentação",
      changelog: "Novidades",
      privacy: "Privacidade",
      github: "GitHub",
      language: "Idioma",
      footer: "QuickDesk · Licença MIT.",
      tagline: "Notas, histórico da área de transferência, portas e limites de IA. A um atalho de distância.",
      systems: "Sistemas",
      soon: "em breve",
      product: "Produto",
      project: "Projeto",
      issues: "Relatar um problema",
      sitemap: "Mapa do site",
      skip: "Pular para o conteúdo",
      menu: "Menu",
      close: "Fechar",
      faq: "Perguntas",
      theme: "Alternar entre claro e escuro",
      themeShort: "Claro / escuro",
    },
    pages: {
      home: {
        title: "QuickDesk — Notas, área de transferência e portas para Linux, Windows e macOS",
        description:
          "QuickDesk é um assistente de desktop gratuito e de código aberto para Linux, Windows e macOS: notas com criptografia de ponta a ponta, histórico da área de transferência, um gerenciador de portas ao vivo e seus limites de uso de IA, com um único atalho.",
      },
      download: {
        title: "Baixar o QuickDesk para Linux, Windows e macOS (grátis)",
        description:
          "Baixe o QuickDesk para Windows 10 e 11, macOS no Apple Silicon, Ubuntu, Debian, Fedora e outras distribuições Linux: o instalador do Windows, .dmg, .deb, .rpm e AppImage, com somas de verificação.",
      },
      docs: {
        title: "Documentação do QuickDesk: instalação, atalhos, sincronização, área de transferência, portas",
        description:
          "Como instalar o QuickDesk no Linux, Windows e macOS e usar os atalhos, a sincronização de notas, o histórico da área de transferência, as portas, as versões de linguagens, o uso de IA e as atualizações.",
      },
      changelog: {
        title: "Novidades do QuickDesk: o que há de novo em cada versão",
        description: "O que há de novo, o que melhorou e o que foi corrigido em cada versão do QuickDesk.",
      },
      privacy: {
        title: "Privacidade · QuickDesk",
        description: "O QuickDesk não tem contas, análises nem telemetria. O que fica no seu computador e o que é enviado, e para onde.",
      },
      sitemap: {
        title: "Mapa do site · QuickDesk",
        description:
          "Todas as páginas de quickdesk.click: downloads para Linux, Windows e macOS, a documentação seção por seção, cada versão das novidades e a privacidade.",
      },
    },
    seo: {
      app: "Um assistente de desktop gratuito e de código aberto: notas sincronizadas com criptografia de ponta a ponta, histórico da área de transferência, um gerenciador de portas, versões de linguagens de programação e limites de uso de IA, cada um a um atalho de distância.",
      ogAlt: "QuickDesk: todo o seu desktop, a um atalho de distância.",
    },
    hero: {
      pick: "Seu sistema",
      linux: "Baixar para Linux",
      windows: "Baixar para Windows",
      macos: "Baixar para macOS{beta}",
      macSoon: "macOS: em breve",
    },
    download: {
      title: "Baixar o QuickDesk {v}",
      leadMac: "Para Linux e Windows (x86_64) e macOS (Apple Silicon{beta}) · lançado em {date}. Grátis e de código aberto (MIT).",
      leadSoon: "Para Linux e Windows em x86_64, macOS em breve · lançado em {date}. Grátis e de código aberto (MIT).",
      pick: "Seu sistema",
      devTitle: "A versão para Mac ainda está em desenvolvimento",
      devText:
        "O QuickDesk para macOS funciona, mas é novo: ainda não foi tudo testado em todos os Macs, e a colagem automática e os atalhos continuam sendo aprimorados. Se algo não funcionar, conte para a gente.",
      devLink: "Relatar um problema no GitHub",
      deb: ["Ubuntu, Debian, Linux Mint, Pop!_OS", "Ubuntu 22.04 ou mais recente, Debian 12 ou mais recente."],
      rpm: ["Fedora e outras distribuições baseadas em RPM", "Fedora 36 ou mais recente, openSUSE Tumbleweed."],
      appimage: ["Qualquer outra distribuição", "Portátil: não precisa de instalação. Requer FUSE 2 (libfuse2)."],
      windows: ["Windows 10 e 11", "64 bits. Instala só para o seu usuário: sem permissão de administrador."],
      macos: ["macOS 11 ou mais recente", "Apple Silicon (M1 ou mais recente). Abra a imagem de disco e arraste o QuickDesk para Aplicativos."],
      oneMac:
        "No Terminal. Baixa o QuickDesk.app, confere o SHA-256, coloca em Aplicativos e abre. Instalado assim, o macOS não mostra nenhum aviso.",
      afterMac:
        "O QuickDesk fica na barra de menus e, a partir de agora, se atualiza sozinho. Para a colagem automática, permita em Privacidade e Segurança → Acessibilidade quando o macOS pedir.",
      gatekeeper: "Se você baixar pelo navegador, o macOS bloqueia a primeira abertura: o QuickDesk ainda não é notarizado pela Apple.",
      gatekeeperHow: "Como abrir",
      openTitle: "Abrir o QuickDesk pela primeira vez",
      openLead:
        "O QuickDesk ainda não é notarizado pela Apple (isso exige um Apple Developer ID pago). Se o .dmg foi baixado pelo navegador, o macOS interrompe a primeira abertura com o aviso de que “a Apple não pôde verificar se ele está livre de malware”. Não há nada de errado com o app: faça uma destas duas coisas, uma única vez. Se você instalou com o comando acima, pode pular esta parte.",
      openA: "Nos Ajustes do Sistema",
      openASteps: [
        "Arraste o QuickDesk para Aplicativos e abra. Quando o macOS mostrar o aviso, feche-o.",
        "Abra Ajustes do Sistema → Privacidade e Segurança e role até Segurança.",
        "Ao lado de “O “QuickDesk” foi bloqueado para proteger o seu Mac”, clique em Abrir Mesmo Assim, confirme com sua senha ou Touch ID e depois clique em Abrir.",
      ],
      openOld: "No macOS 14 ou anterior, também funciona clicar com Control no QuickDesk em Aplicativos, escolher Abrir e clicar em Abrir de novo.",
      openB: "Ou no Terminal",
      openBLead: "Depois de arrastar o QuickDesk para Aplicativos, execute isto e abra o QuickDesk normalmente:",
      get: "Baixar",
      sha: "SHA-256",
      all: "Todas as somas de verificação (SHA256SUMS)",
      notes: "Novidades da versão {v}",
      allChanges: "Todas as mudanças",
      afterLinux: "Depois abra o QuickDesk pelo menu de aplicativos. Ele fica na barra superior e, a partir de agora, se atualiza sozinho.",
      afterWindows: "Depois abra o QuickDesk pelo menu Iniciar. Ele fica na bandeja do sistema e, a partir de agora, se atualiza sozinho.",
      copy: "Copiar",
      oneTitle: "Instalar com um único comando",
      oneLinux:
        "Em um terminal. Escolhe o pacote certo para a sua distribuição (.deb, .rpm ou AppImage), confere o SHA-256 e instala. Só pede sua senha na etapa de instalação.",
      oneWindows: "No PowerShell. Baixa o instalador, confere o SHA-256 e instala o QuickDesk para o seu usuário, sem permissão de administrador.",
      oneScript: "Ler o script",
      orPick: "Ou escolha um pacote",
      smartscreen: "O Windows pode dizer que protegeu seu PC: o instalador ainda não é assinado. Escolha Mais informações → Executar assim mesmo.",
      macTitle: "O QuickDesk para macOS está a caminho",
      macLead: "Notas, histórico da área de transferência, portas e limites de IA no Mac. Será anunciado nas novidades e nos lançamentos do GitHub.",
      macWatch: "Acompanhar os lançamentos no GitHub",
    },
    cta: {
      title: "Pronto quando você estiver.",
      lead: "Instale uma vez e tudo fica a um atalho de distância: suas notas, o que você copiou, as portas em uso e seus limites de IA.",
      download: "Baixar",
      changes: "Novidades",
      meta: "Versão {v} · {date} · Grátis e de código aberto (MIT)",
      downloading: "Baixando",
      verified: "SHA-256 verificado",
      installed: "QuickDesk {v} instalado",
      running: "O QuickDesk está em execução",
      open: "Abra com",
    },
    intro: {
      tagline: "Todo o seu desktop. A um atalho de distância.",
      skip: "Pular",
    },
    docs: {
      title: "Documentação",
      lead: "Instalação, atalhos, sincronização, área de transferência, portas, uso de IA e atualizações.",
      contents: "Conteúdo",
      groups: ["Primeiros passos", "Recursos", "Ajuda"],
      sync: "Sincronização",
      clipboard: "Área de transferência",
      versions: "Runtimes",
      stuck: "Ainda com dificuldade?",
      ask: "Pergunte no GitHub",
    },
    sitemap: {
      title: "Mapa do site",
      lead: "Todas as páginas e seções de quickdesk.click.",
      home: "Início",
      demo: "Experimente os atalhos",
      getIt: "Obtenha o QuickDesk",
      oneLine: "Scripts de instalação",
      other: "Em outros sites",
      machine: "sitemap.xml (para buscadores)",
      privacyNote: "Sem contas, análises nem telemetria: o que fica no seu computador e o que é enviado, e para onde.",
    },
    changelog: {
      title: "Novidades",
      lead: "O que há de novo, o que melhorou e o que foi corrigido em cada versão do QuickDesk.",
      latest: "Mais recente",
      source: "Fonte: o registro de alterações no GitHub",
      english: "Ainda não traduzido; exibido em inglês.",
    },
    notFound: {
      title: "Página não encontrada · QuickDesk",
      description: "Esta página não existe.",
      heading: "Esta página não existe.",
    },
    demo: {
      refreshed: "Uso atualizado",
      paused: "Histórico da área de transferência pausado",
      resumed: "Histórico da área de transferência retomado",
      quit: "Sair — no app de verdade, o QuickDesk fecharia agora",
      encrypting: "Criptografando…",
      synced: "Criptografado e sincronizado",
      copied: "Copiado",
      pressCopy: "Pressione Ctrl+C",
      copy: "Copiar",
      note: "Daily — terça-feira\n• Imagens na área de transferência lançadas\n• Revisando o processo de lançamento\n• Sem bloqueios",
    },
  },
  content: { home, docs, privacy, changelog },
};

export default pt;
