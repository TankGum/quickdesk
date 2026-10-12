// 简体中文 (Simplified Chinese). See ../en.ts for what each part is. The app
// itself is in English and Vietnamese, so its menu and button names stay in
// English (Settings → Sync …) for people to find them.
import type { Locale } from "..";
import changelog from "./CHANGELOG.md?raw";
import docs from "./docs";
import home from "./home";
import privacy from "./privacy";

const zh: Locale = {
  strings: {
    ui: {
      home: "首页",
      download: "下载",
      docs: "文档",
      changelog: "更新日志",
      privacy: "隐私",
      github: "GitHub",
      language: "语言",
      footer: "QuickDesk · MIT 许可证。",
      tagline: "笔记、剪贴板历史、端口和 AI 用量。一个快捷键即可打开。",
      systems: "系统",
      soon: "即将推出",
      product: "产品",
      project: "项目",
      issues: "报告问题",
      sitemap: "网站地图",
      skip: "跳到正文",
      menu: "菜单",
      close: "关闭",
      faq: "常见问题",
      theme: "切换浅色 / 深色",
      themeShort: "浅色 / 深色",
    },
    pages: {
      home: {
        title: "QuickDesk — 适用于 Linux、Windows 和 macOS 的笔记、剪贴板与端口工具",
        description:
          "QuickDesk 是一款免费、开源的桌面助手，支持 Linux、Windows 和 macOS：端到端加密的笔记、剪贴板历史、实时端口管理器以及 AI 用量限额，一个快捷键即可呼出。",
      },
      download: {
        title: "下载 QuickDesk（Linux、Windows 和 macOS，免费）",
        description:
          "下载适用于 Windows 10 和 11、Apple 芯片 Mac、Ubuntu、Debian、Fedora 及其他 Linux 发行版的 QuickDesk：Windows 安装程序、.dmg、.deb、.rpm 和 AppImage，并附校验和。",
      },
      docs: {
        title: "QuickDesk 文档：安装、快捷键、同步、剪贴板、端口",
        description: "如何在 Linux、Windows 和 macOS 上安装 QuickDesk，使用快捷键、同步笔记、剪贴板历史、端口、语言版本、AI 用量和更新。",
      },
      changelog: {
        title: "QuickDesk 更新日志：每个版本的新内容",
        description: "QuickDesk 每个版本的新功能、改进和修复。",
      },
      privacy: {
        title: "隐私 · QuickDesk",
        description: "QuickDesk 没有账号、没有分析、没有遥测。哪些数据留在你的电脑上，哪些会被发送到哪里。",
      },
      sitemap: {
        title: "网站地图 · QuickDesk",
        description: "quickdesk.click 的所有页面：Linux、Windows 和 macOS 下载，按章节列出的文档，更新日志中的每个版本，以及隐私说明。",
      },
    },
    seo: {
      app: "免费、开源的桌面助手：端到端加密并可同步的笔记、剪贴板历史、端口管理器、编程语言版本和 AI 用量限额，每一项都只需一个快捷键。",
      ogAlt: "QuickDesk：整个桌面，一个快捷键即达。",
    },
    hero: {
      pick: "你的系统",
      linux: "下载 Linux 版",
      windows: "下载 Windows 版",
      macos: "下载 macOS 版{beta}",
      macSoon: "macOS：即将推出",
    },
    download: {
      title: "下载 QuickDesk {v}",
      leadMac: "适用于 Linux 和 Windows（x86_64）以及 macOS（Apple 芯片{beta}）· 发布于 {date}。免费开源（MIT）。",
      leadSoon: "适用于 x86_64 的 Linux 和 Windows，macOS 即将推出 · 发布于 {date}。免费开源（MIT）。",
      pick: "你的系统",
      devTitle: "Mac 版仍在开发中",
      devText:
        "QuickDesk for macOS 已经可以使用，但它还很新：并非所有功能都在每台 Mac 上测试过，自动粘贴和快捷键仍在完善中。如果有什么不能用，请告诉我们。",
      devLink: "在 GitHub 上报告问题",
      deb: ["Ubuntu、Debian、Linux Mint、Pop!_OS", "Ubuntu 22.04 及以上，Debian 12 及以上。"],
      rpm: ["Fedora 及其他基于 RPM 的发行版", "Fedora 36 及以上，openSUSE Tumbleweed。"],
      appimage: ["其他任何发行版", "便携版：无需安装。需要 FUSE 2（libfuse2）。"],
      windows: ["Windows 10 和 11", "64 位。只为你的用户安装：不需要管理员权限。"],
      macos: ["macOS 11 或更高版本", "Apple 芯片（M1 及更新）。打开磁盘映像，把 QuickDesk 拖到“应用程序”文件夹。"],
      oneMac: "在终端中运行。下载 QuickDesk.app，校验其 SHA-256，放入“应用程序”文件夹并打开。用这种方式安装，macOS 不会发出警告。",
      afterMac: "QuickDesk 位于菜单栏，今后会自动更新。若要自动粘贴，请在 macOS 询问时于“隐私与安全性 → 辅助功能”中允许它。",
      gatekeeper: "如果用浏览器下载，macOS 会阻止首次启动：QuickDesk 尚未经过 Apple 公证。",
      gatekeeperHow: "如何打开",
      openTitle: "首次打开 QuickDesk",
      openLead:
        "QuickDesk 尚未经过 Apple 公证（这需要付费的 Apple Developer ID）。如果 .dmg 是用浏览器下载的，macOS 会在首次启动时提示“Apple 无法检查其是否包含恶意软件”。应用本身没有问题：只需执行以下任一方法一次。用上面的命令安装的话，可以跳过这一步。",
      openA: "在“系统设置”中",
      openASteps: [
        "把 QuickDesk 拖到“应用程序”并打开。macOS 发出警告时，点击“完成”。",
        "打开“系统设置 → 隐私与安全性”，向下滚动到“安全性”。",
        "在“已阻止 QuickDesk 以保护你的 Mac”旁边点击“仍要打开”，用密码或触控 ID 确认，然后点击“打开”。",
      ],
      openOld: "在 macOS 14 或更早版本上，也可以在“应用程序”中按住 Control 点按 QuickDesk，选择“打开”，再点一次“打开”。",
      openB: "或者在终端中",
      openBLead: "把 QuickDesk 拖到“应用程序”之后，运行下面的命令，然后照常打开 QuickDesk：",
      get: "下载",
      sha: "SHA-256",
      all: "所有校验和（SHA256SUMS）",
      notes: "{v} 的新内容",
      allChanges: "全部更新",
      afterLinux: "然后从应用菜单打开 QuickDesk。它位于顶部栏，今后会自动更新。",
      afterWindows: "然后从“开始”菜单打开 QuickDesk。它位于系统托盘，今后会自动更新。",
      copy: "复制",
      oneTitle: "一条命令安装",
      oneLinux: "在终端中运行。自动为你的发行版选择合适的软件包（.deb、.rpm 或 AppImage），校验 SHA-256 后安装。只有安装那一步会要求输入密码。",
      oneWindows: "在 PowerShell 中运行。下载安装程序，校验 SHA-256，并为你的用户安装 QuickDesk，无需管理员权限。",
      oneScript: "查看脚本",
      orPick: "或者选择一个软件包",
      smartscreen: "Windows 可能会提示“已保护你的电脑”：安装程序尚未签名。选择“更多信息 → 仍要运行”。",
      macTitle: "QuickDesk for macOS 即将到来",
      macLead: "在 Mac 上使用笔记、剪贴板历史、端口和 AI 用量。发布时会在更新日志和 GitHub Releases 中公布。",
      macWatch: "在 GitHub 上关注版本发布",
    },
    cta: {
      title: "随时为你准备好。",
      lead: "安装一次，一切都只需一个快捷键：你的笔记、复制过的内容、正在使用的端口和 AI 用量。",
      download: "下载",
      changes: "新内容",
      meta: "版本 {v} · {date} · 免费开源（MIT）",
      downloading: "正在下载",
      verified: "SHA-256 已校验",
      installed: "已安装 QuickDesk {v}",
      running: "QuickDesk 正在运行",
      open: "打开方式",
    },
    intro: {
      tagline: "整个桌面，一个快捷键即达。",
      skip: "跳过",
    },
    docs: {
      title: "文档",
      lead: "安装、快捷键、同步、剪贴板、端口、AI 用量和更新。",
      contents: "目录",
      groups: ["入门", "功能", "帮助"],
      sync: "同步",
      clipboard: "剪贴板",
      versions: "运行时",
      stuck: "还有疑问？",
      ask: "在 GitHub 上提问",
    },
    sitemap: {
      title: "网站地图",
      lead: "quickdesk.click 的每个页面和章节。",
      home: "首页",
      demo: "试试快捷键",
      getIt: "获取 QuickDesk",
      oneLine: "安装脚本",
      other: "其他",
      machine: "sitemap.xml（供搜索引擎使用）",
      privacyNote: "没有账号、分析或遥测：哪些数据留在你的电脑上，哪些会被发送到哪里。",
    },
    changelog: {
      title: "更新日志",
      lead: "QuickDesk 每个版本的新功能、改进和修复。",
      latest: "最新",
      source: "来源：GitHub 上的更新日志",
      english: "尚未翻译，以英文显示。",
    },
    notFound: {
      title: "页面未找到 · QuickDesk",
      description: "此页面不存在。",
      heading: "此页面不存在。",
    },
    demo: {
      refreshed: "用量已刷新",
      paused: "剪贴板历史已暂停",
      resumed: "剪贴板历史已恢复",
      quit: "退出 — 在真实应用中，QuickDesk 现在会关闭",
      encrypting: "正在加密…",
      synced: "已加密并同步",
      copied: "已复制",
      pressCopy: "按 Ctrl+C",
      copy: "复制",
      note: "站会 — 周二\n• 已发布剪贴板图片\n• 正在审查发布流程\n• 没有阻碍",
    },
  },
  content: { home, docs, privacy, changelog },
};

export default zh;
