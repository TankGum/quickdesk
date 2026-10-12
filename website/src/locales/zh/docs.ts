// Docs in Simplified Chinese, by section id (src/data/docs.ts). Links like
// /download are pointed at the Chinese pages when shown.
import { macBeta } from "../../data/release";

const docs: Record<string, { title: string; html: string }> = {
  install: {
    title: "安装",
    html: `<p>最快的方式，适用于任何发行版：</p>
<pre><code>curl -fsSL https://quickdesk.click/install.sh | bash</code></pre>
<p>它会为你的系统选择 .deb、.rpm 或 AppImage，校验 SHA-256 后安装（<a href="/install.sh">查看脚本</a>）。也可以从<a href="/download">下载页面</a>获取软件包，然后这样安装下载的文件：</p>
<ul>
<li><strong>Ubuntu、Debian、Mint、Pop!_OS：</strong> <code>sudo apt install ~/Downloads/QuickDesk_&lt;version&gt;_amd64.deb</code></li>
<li><strong>Fedora、openSUSE：</strong> <code>sudo dnf install ~/Downloads/QuickDesk-&lt;version&gt;-1.x86_64.rpm</code></li>
<li><strong>其他发行版：</strong> 给 AppImage 加上可执行权限后运行。它需要 FUSE 2（<code>libfuse2</code>，在 Ubuntu 24.04 上是 <code>libfuse2t64</code>）。</li>
</ul>
<p>QuickDesk 需要近几年的 x86_64 系统（glibc 2.34 或更新）。它在 GNOME 上效果最好，Wayland 和 X11 均可。</p>
<p><strong>Windows 10 和 11（64 位）：</strong> 在 PowerShell 中运行 <code>irm https://quickdesk.click/install.ps1 | iex</code>，或从<a href="/download">下载页面</a>下载安装程序。它只为你的用户安装，不需要管理员权限。安装程序尚未签名，因此 Windows 可能会提示“已保护你的电脑”：选择<em>更多信息 → 仍要运行</em>。在 Windows 上，快捷键是 <kbd>Win</kbd> <kbd>Alt</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd>，QuickDesk 位于系统托盘，自动粘贴无需设置。那里的 Runtimes 标签通过 mise、rustup 和 uv 工作（见<em>语言版本</em>）。</p>
<p>首次启动时，欢迎界面会列出快捷键并提供两个选项：<strong>登录时启动</strong>（剪贴板历史和 AI 圆环要一直工作就需要它）和<strong>即时粘贴</strong>（见<a href="#auto-paste">自动粘贴</a>）。之后 QuickDesk 就待在顶部栏中。</p>`,
  },
  macos: {
    title: "macOS",
    html: `${macBeta ? '<p class="note"><strong>仍在开发中。</strong>Mac 版已经可以使用，但它还很新：并非所有功能都在每台 Mac 上测试过，自动粘贴和快捷键仍在完善中。如果有什么不能用，请<a href="https://github.com/TankGum/quickdesk/issues">告诉我们</a>。</p>\n' : ""}<p>QuickDesk 可在 Apple 芯片（M1 及更新）的 macOS 11 或更高版本上运行。在终端中运行 <code>curl -fsSL https://quickdesk.click/install.sh | bash</code>：它会把 QuickDesk.app 放进“应用程序”并打开。也可以从<a href="/download#macos">下载页面</a>下载 <code>.dmg</code>，把 QuickDesk 拖到“应用程序”。</p>
<p><strong>首次打开。</strong>QuickDesk 尚未经过 Apple 公证，因此如果 <code>.dmg</code> 是用浏览器下载的，macOS 会在首次启动时提示“Apple 无法检查其是否包含恶意软件”。应用本身没有问题；只需执行以下任一方法一次（用命令安装则不会有警告）：</p>
<ol>
<li>从“应用程序”打开 QuickDesk，在 macOS 警告时点击<em>完成</em>。然后打开“系统设置 → 隐私与安全性”，向下滚动到“安全性”，在“已阻止 QuickDesk 以保护你的 Mac”旁边点击<em>仍要打开</em>，用密码或触控 ID 确认，再点击<em>打开</em>。在 macOS 14 或更早版本上，按住 Control 点按 QuickDesk → <em>打开</em> → <em>打开</em> 也可以。</li>
<li>或者在终端中运行 <code>xattr -dr com.apple.quarantine /Applications/QuickDesk.app</code>，然后照常打开 QuickDesk。</li>
</ol>
<p><a href="/download#macos-open">下载页面</a>上也有同样的步骤。</p>
<ul>
<li>QuickDesk 位于菜单栏，没有程序坞图标。它的弹出窗口会在你当前所在的空间中打开，即使有应用处于全屏状态。</li>
<li>快捷键是 <kbd>⌃</kbd> <kbd>⌥</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd>（Control + Option）：⌘⌥V 和 ⌘⌥N 属于访达。</li>
<li><strong>自动粘贴</strong>会在你刚才使用的应用中按下 ⌘V，macOS 只有在你于“系统设置 → 隐私与安全性 → <strong>辅助功能</strong>”中添加 QuickDesk 之后才允许这样做。macOS 会在你第一次选择记录时询问；在此之前，记录会被复制，由你自己按 ⌘V。Settings → Auto-paste 会显示是否已允许。</li>
<li>较新的 macOS 版本可能会询问 QuickDesk 能否读取其他应用复制的内容。请选择“允许”，否则剪贴板历史会一直是空的。</li>
<li><strong>Claude 用量：</strong>在 Mac 上，Claude Code 把登录信息保存在钥匙串中。AI 标签会提出从那里读取；随后 macOS 会询问一次（选择<em>始终允许</em>）。与其他系统一样，登录信息只会被读取，并且只发送到 <code>api.anthropic.com</code>。</li>
<li>由于应用采用 ad hoc 签名，更新后 macOS 可能会再次请求“辅助功能”和钥匙串权限。如果自动粘贴失效，请把 QuickDesk 从“辅助功能”列表中移除后再重新添加。</li>
<li>Runtimes 标签的工作方式与 Linux 上相同，使用你的登录 shell（zsh：<code>~/.zshrc</code>）。用 Homebrew 安装的副本会显示出来，但不会被修改。</li>
</ul>
<p><strong>卸载：</strong>从菜单栏退出 QuickDesk，然后把它从“应用程序”移到废纸篓。你的数据保留在 <code>~/Library/Application Support/click.quickdesk</code>，直到你删除该文件夹。</p>`,
  },
  shortcuts: {
    title: "快捷键",
    html: `<table>
<tr><th>快捷键</th><th>打开</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>笔记</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>剪贴板历史弹窗</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>端口</td></tr>
<tr><td>未设置</td><td>快速笔记弹窗（可在 Settings 中设置）</td></tr>
</table>
<p>窗口打开时再次按下快捷键会将其关闭。可以在 <strong>Settings → Hotkeys</strong> 中更改任何快捷键；如果组合已被占用，QuickDesk 会发出提醒。</p>
<p>在 GNOME 上，快捷键会被添加到<em>设置 → 键盘 → 自定义快捷键</em>，并在 QuickDesk 退出时删除，因此在 Wayland 上也能使用。在其他桌面上，请在桌面的键盘设置中把 <code>quickdesk toggle notes</code>、<code>quickdesk toggle clipboard</code>、<code>quickdesk toggle ports</code> 或 <code>quickdesk toggle quick-note</code> 绑定到按键。</p>
<p><strong>为什么是 Super + Alt？</strong>在文本框中输入时，Unikey 和 Bamboo 等越南语输入法会吞掉 Super + Shift + 字母；带 Alt 的组合则不受影响。</p>`,
  },
  notes: {
    title: "笔记",
    html: `<p>笔记使用 Markdown 和富文本编辑器：标题、粗体和斜体、列表、清单、引用和代码块。<kbd>Ctrl</kbd> <kbd>N</kbd> 新建笔记，<kbd>Ctrl</kbd> <kbd>S</kbd> 保存，<kbd>Ctrl</kbd> <kbd>F</kbd> 搜索。搜索会忽略重音符号，因此 <code>tieng viet</code> 能找到 “tiếng Việt”。</p>
<p>点星标可以把笔记固定在顶部。删除前会先询问，并且几秒内可以撤销。</p>`,
  },
  sync: {
    title: "在多台电脑之间同步笔记",
    html: `<p>笔记通过 <strong>QuickDesk Cloud</strong> 同步。不需要账号或邮箱：一台电脑获得一个<strong>同步码</strong>，你的其他电脑用它加入。只同步笔记；剪贴板历史和设置留在各自的电脑上。</p>
<h3>设置</h3>
<ol>
<li>在第一台电脑上：<strong>Settings → Sync → Turn on sync</strong>，并选择一个密码短语（至少 8 个字符）。</li>
<li>QuickDesk 会显示你的<strong>同步码</strong>（<code>QD1-…</code>）和<strong>恢复密钥</strong>。请把恢复密钥保存在安全的地方；它只显示一次。</li>
<li>在其他每台电脑上：<strong>Settings → Sync → I have a sync code</strong>，然后输入同步码和相同的密码短语（或恢复密钥）。</li>
</ol>
<p>随时可以在 Settings → Sync 中用 <strong>Show sync code</strong> 再次显示同步码。</p>
<h3>何时同步</h3>
<ul>
<li>编辑笔记几秒钟之后，以及打开 QuickDesk 窗口时。</li>
<li>其他时候，窗口打开时 QuickDesk 每分钟检查一次更改，待在托盘中时每 5 分钟检查一次。每次检查只是一个很小的请求；只有内容有变化时才会传输笔记。</li>
<li><strong>Sync now</strong> 会立即检查。离线时的修改会在你重新联网后发送。</li>
<li>如果同一条笔记在两台电脑上同时被修改，两个版本都会保留：较新的那个保留原位，另一个成为旁边的副本。编辑优先于删除。</li>
</ul>
<h3>隐私与密钥</h3>
<ul>
<li>笔记在上传之前就在你的电脑上加密（XChaCha20-Poly1305，密钥通过 Argon2id 从你的密码短语派生）。QuickDesk Cloud 只以随机账户 ID 保存密文，无法读取你的笔记。</li>
<li>需要<strong>同时</strong>拥有同步码（用于访问数据）和密码短语（用于解密）。请像对待密码一样对待同步码：仅凭它，别人仍然可以删除你已同步的笔记。</li>
<li>忘记了密码短语？改为输入恢复密钥。两者都没有的话，QuickDesk Cloud 中的副本无法打开，但每台电脑上的笔记不受影响。</li>
<li>如果 QuickDesk 要求 <strong>Unlock sync</strong>（例如系统钥匙串被重置之后），请再次输入密码短语。</li>
</ul>
<h3>关闭同步</h3>
<p><strong>Turn off sync…</strong> 会停止在某台电脑上同步，并保留它的笔记。勾选 <em>Also delete my notes from QuickDesk Cloud</em> 可为所有电脑删除已同步的副本。</p>
<p>每个同步码的上限：256 MB 和 20,000 个存储对象，远超笔记所需。</p>`,
  },
  clipboard: {
    title: "剪贴板历史",
    html: `<p>你复制的一切都会被保存：文字、图片（截图）以及在文件管理器中复制的文件。用 <kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd> 打开弹窗，输入即可搜索，可按文字、图片或文件筛选，按 <kbd>Enter</kbd> 粘贴到你刚才使用的应用中。<kbd>Ctrl</kbd> <kbd>Enter</kbd> 只复制，<kbd>Ctrl</kbd> <kbd>P</kbd> 固定，<kbd>Del</kbd> 删除。</p>
<ul>
<li>历史记录留在这台电脑上，从不同步。</li>
<li>被密码管理器标记为机密的内容会被跳过。在复制敏感内容之前，可以<strong>暂停保存</strong> 15 分钟、1 小时，或直到你重新开启（在 Clipboard 标签或托盘菜单中）。</li>
<li>QuickDesk 保留最近 1,000 条记录；已固定的永久保留。文字最大 1 MB，每张图片最大 10 MB，总计 200 MB。</li>
<li>Clipboard 标签中的 <strong>Clear…</strong> 会删除除已固定记录以外的所有内容。</li>
</ul>`,
  },
  "auto-paste": {
    title: "自动粘贴",
    html: `<p>当你选中一条记录时，QuickDesk 会回到你刚才使用的应用并按下 <kbd>Shift</kbd> <kbd>Insert</kbd>，这在终端中也有效。在 <strong>Settings → Auto-paste</strong> 中选择方式：</p>
<ul>
<li><strong>Desktop portal</strong>（默认）：GNOME 只请求一次权限，粘贴时显示远程控制指示器。</li>
<li><strong>Instant (virtual keyboard)</strong>：没有提示也没有指示器。启用时需要输入一次管理员密码，以允许以你的身份运行的程序创建虚拟键盘（Steam 为手柄使用的就是同样的权限）。</li>
</ul>
<p>在 Clipboard 标签中关闭自动粘贴后，就只复制，由你自己按 <kbd>Ctrl</kbd> <kbd>V</kbd>。</p>`,
  },
  ports: {
    title: "端口",
    html: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> 列出所有正在监听的 TCP 端口（IPv4 和 IPv6），以及进程、完整命令行和用户，每 2 秒刷新一次。可以按端口、PID、进程或容器名称筛选。</p>
<ul>
<li><strong>Open</strong> 在浏览器中打开 <code>http://localhost:&lt;port&gt;</code>；<strong>Copy PID</strong> 和 <strong>Copy cmd</strong> 复制相应内容。</li>
<li><strong>Kill</strong> 会先询问，然后温和地停止进程，只有在它不退出时才提供强制结束。</li>
<li>由 Docker 发布的端口会显示容器和镜像，<strong>Stop</strong> 会停止容器，而不是结束 <code>docker-proxy</code>。</li>
<li>属于其他用户的进程（系统服务）显示为隐藏；查看它们需要 root 权限。</li>
</ul>`,
  },
  versions: {
    title: "语言版本（Runtimes）",
    html: `<p><strong>Runtimes</strong> 标签管理这台电脑上各编程语言的版本（Node.js、Python、Rust、Go、Java、Ruby、Deno、Bun、PHP）。它显示<em>新终端</em>运行的版本及其来源。QuickDesk 会询问你的登录 shell，因此这正是执行 <code>~/.bashrc</code> 或 <code>~/.zshrc</code> 之后得到的结果，而不是桌面看到的。</p>
<ul>
<li>用 <strong>nvm</strong>（Node.js）、<strong>rustup</strong>（Rust）、<strong>uv</strong>（Python）或 <strong>mise</strong>（Go、Java、Ruby、Deno、Bun、PHP 等）安装的语言可以修改：<strong>Install version…</strong> 从工具提供的版本中选择（rustup 接受你输入的渠道或版本），<strong>Make default</strong> 设置新终端使用的版本，<strong>Uninstall</strong> 删除某个版本并释放空间。默认版本不能删除；请先选择另一个默认版本。</li>
<li>安装在后台运行，有进度条，可以取消。一次只运行一项更改。版本列表默认显示每个版本线的最新版本；可切换为 LTS 或全部，也可以输入进行筛选。</li>
<li>当已安装版本线有更新的版本发布时（例如 22.23.3 有了 Node 22.24.1，3.13.7 有了 Python 3.13.9），该版本会显示 <strong>↑ 22.24.1</strong>：一键安装新版本，如果旧版本是默认版本，则把新版本设为默认。旧版本会保留，直到你卸载它。Rust 渠道则是原地更新：当 rustup 有更新的 stable 时，<strong>stable</strong> 会显示 <strong>↑ 1.93.0</strong>，点击会运行 <code>rustup update stable</code>。</li>
<li><strong>Apply right away in open terminals</strong>（标签顶部）让更改从下一个提示符起就作用于已经打开的终端：QuickDesk 会在你的 shell 文件中加一行，加载一个小的提示符钩子（<code>~/.config/quickdesk/shell/hook.sh</code>）。这个钩子会让 shell 在 QuickDesk 更改版本后重新查找命令，并让 Node 在你 <code>cd</code> 进入项目时跟随最近的 <code>.nvmrc</code>，否则使用你选择的默认版本（手动 <code>nvm use</code> 会一直有效，直到你更换文件夹或默认版本）。没有它的话，Rust 的更改仍会立即生效，Python 和 mise 的更改会作用于新终端（或在已打开的终端中执行 <code>hash -r</code> 之后），nvm 的更改只作用于新终端。</li>
<li><strong>Set for a project…</strong> 会在你选择的文件夹中写入该工具自己的文件：<code>.nvmrc</code>、<code>rust-toolchain.toml</code>、<code>.python-version</code> 或 <code>mise.toml</code>（mise 会保留其中已列出的其他工具）。</li>
<li>没有版本管理器的语言（例如通过 apt 安装的 Go）会提供 <strong>Install mise…</strong>：QuickDesk 从 github.com/jdx/mise 下载最新的 mise，用该版本的 SHA-256 校验和进行核对，并放到 <code>~/.local/bin</code>。在你把某个版本设为默认之后，<strong>Fix PATH…</strong> 会提供在你的 shell 中启用 mise 的那一行。</li>
<li>如果新终端运行的不是默认版本（例如系统自带的 <code>python3</code> 在 PATH 中排在前面），卡片会说明原因，<strong>Fix PATH…</strong> 会显示它将要添加的具体内容。只有在你同意之后，QuickDesk 才会把它们作为一个带标记的块添加到 shell 文件末尾，并在旁边留一份备份；之后也可以在同一处删除这个块。</li>
<li>PATH 修复和提示符钩子是为 bash（<code>~/.bashrc</code>）和 zsh（<code>~/.zshrc</code>）编写的；暂不支持 fish。</li>
<li>电脑上还没有的语言列在 <strong>Add a language</strong> 下：选择一个，必要时先安装 mise，然后选择版本。</li>
<li><strong>Windows：</strong>QuickDesk 询问 PowerShell 新终端运行的是什么，并通过 <strong>mise</strong>（安装到 <code>%LOCALAPPDATA%\\mise\\bin</code>）、<strong>rustup</strong> 和 <strong>uv</strong> 工作；nvm-windows 和 fnm 会显示但不会被修改，Microsoft Store 的 <code>python.exe</code> 占位程序会被忽略。<strong>Fix PATH…</strong> 会把工具的文件夹（对 mise 来说是它的 shims）放到你账户 PATH（即“设置 → 环境变量”中的那个）的最前面，不需要管理员权限；旧值保存在 <code>~\\.config\\quickdesk\\path</code>，同一个面板也可以再次删除这些文件夹。新终端会采用它；之后更改默认版本会立即生效。Windows 上不需要也不会显示 “Apply right away”。</li>
<li>系统安装的副本（apt、dnf）永远不会被修改或删除；QuickDesk 从不运行 <code>sudo</code>。由它不支持的工具（fnm、volta、pyenv、SDKMAN…）管理的语言以只读方式显示。</li>
</ul>`,
  },
  ai: {
    title: "AI 用量",
    html: `<p>顶部栏中的一个圆环显示你已经用了多少 AI 限额，用绿色、琥珀色或红色表示。它的菜单显示你跟踪的工具，每项限额一个进度条以及重置时间；AI 标签显示所有工具。</p>
<ul>
<li><strong>Claude Code：</strong>QuickDesk 读取 Claude Code 已保存在这台电脑上的登录信息，并向 Anthropic 查询你的用量，最多每 5 分钟一次。令牌只被读取：从不存储、刷新，也不会发送到 Anthropic 以外的任何地方。</li>
<li><strong>Codex：</strong>读取 Codex 保存在 <code>~/.codex</code> 中的会话日志。</li>
<li><strong>Gemini（Antigravity）：</strong>它不提供百分比，因此 QuickDesk 根据它的日志显示你上次用完配额的时间。</li>
</ul>
<p>在 <em>Track</em> 下选择要跟踪的工具和限额，或在 AI 标签中隐藏圆环。</p>`,
  },
  updates: {
    title: "更新",
    html: `<p>QuickDesk 在启动 30 秒后检查新版本，之后每 6 小时检查一次。有新版本时，主窗口中的横幅和托盘菜单顶部的一项会提示你，并附有 <strong>What’s New</strong>。在你点击 <strong>Update Now</strong> 之前，不会安装任何东西。</p>
<p>每次更新在安装前都会用 QuickDesk 的签名密钥进行验证。.deb 或 .rpm 会要求输入管理员密码；AppImage 会替换自身。随后 QuickDesk 会重启并显示更新内容。可以在 <strong>Settings → Updates</strong> 中关闭自动检查或手动检查。</p>`,
  },
  uninstall: {
    title: "卸载",
    html: `<ol>
<li>在 Settings 中关闭 <em>Start QuickDesk when I log in</em>，然后从托盘退出。</li>
<li>删除软件包：<code>sudo apt remove quick-desk</code>（Debian/Ubuntu）或 <code>sudo dnf remove QuickDesk</code>（Fedora），或者删除 AppImage。</li>
<li>你的数据保留在 <code>~/.local/share/click.quickdesk</code>，直到你删除该文件夹。</li>
</ol>
<p><strong>Windows：</strong>设置 → 应用 → 已安装的应用 → QuickDesk → 卸载。你的数据保留在 <code>%APPDATA%\\click.quickdesk</code>，直到你删除该文件夹。</p>
<p>删除软件包也会移除即时粘贴权限（如果你启用过）。</p>`,
  },
  troubleshooting: {
    title: "故障排除",
    html: `<ul>
<li><strong>快捷键没反应：</strong>在<em>设置 → 键盘 → 自定义快捷键</em>中检查是否有冲突，或在 Settings → Hotkeys 中换一个组合。</li>
<li><strong>自动粘贴没有输入任何内容：</strong>确认你刚才使用的应用仍然处于焦点状态；在 Settings → Auto-paste 中更换方式。记录总是会被复制，所以 <kbd>Ctrl</kbd> <kbd>V</kbd> 可以用。</li>
<li><strong>日志：</strong><code>~/.local/share/click.quickdesk/logs/</code>。<a href="https://github.com/TankGum/quickdesk/issues">报告问题</a>时请附上它们。</li>
</ul>`,
  },
};

export default docs;
