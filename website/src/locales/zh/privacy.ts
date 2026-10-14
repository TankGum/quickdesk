// Privacy page in Simplified Chinese. Keep it in step with src/data/privacy.ts.
const privacy = {
  title: "隐私",
  updated: "最后更新于 2026 年 10 月 14 日（笔记中的图片）",
  html: `<p class="lead">QuickDesk 没有账号、没有分析、没有遥测。你的笔记和剪贴板历史保存在你的电脑上。本页列出了所有会离开电脑的内容。</p>
<h2>留在你电脑上的内容</h2>
<ul>
<li><strong>笔记、剪贴板历史（文字、图片、文件列表）、设置和日志</strong>，位于 <code>~/.local/share/click.quickdesk</code>。</li>
<li><strong>剪贴板历史永远不会离开你的电脑</strong>，即使开启了同步也一样。被密码管理器标记为机密的内容不会被保存，你也可以随时暂停保存。</li>
<li><strong>端口</strong>信息从你机器上的 <code>/proc</code> 和 Docker 读取，不会被存储。</li>
<li>你的同步码和加密密钥保存在系统钥匙串中。</li>
</ul>
<h2>QuickDesk 发送什么，发送到哪里</h2>
<table>
<tr><th>何时</th><th>发送到</th><th>内容</th></tr>
<tr><td data-label="何时">检查更新（启动 30 秒后，之后每 6 小时一次；可以关闭）</td><td data-label="发送到"><code>dl.quickdesk.click</code>（Cloudflare）</td><td data-label="内容">一个获取 <code>update.json</code> 的请求，以及你选择更新时的软件包。与任何下载一样，Cloudflare 能看到你的 IP 地址。</td></tr>
<tr><td data-label="何时">笔记同步，仅当你开启它时</td><td data-label="发送到">QuickDesk Cloud，<code>sync.quickdesk.click</code>（Cloudflare）</td><td data-label="内容">你的笔记及其中的图片，已在你的电脑上用由密码短语派生的密钥加密（XChaCha20-Poly1305）；QuickDesk Cloud 无法读取。它以随机的同步账户 ID 保存这些数据及其大小和上传时间，并能看到你的 IP 地址。不涉及邮箱、姓名或其他个人数据。你可以在设置 → 同步中删除所有内容。</td></tr>
<tr><td data-label="何时">AI 用量，仅当这台电脑上已登录 Claude Code 时（在 Mac 上，仅在你允许从钥匙串读取其登录信息之后）</td><td data-label="发送到"><code>api.anthropic.com</code></td><td data-label="内容">一个用 Claude Code 现有登录授权的用量请求，最多每 5 分钟一次。令牌只被读取，从不存储、刷新或发送到其他任何地方。</td></tr>
<tr><td data-label="何时">运行时标签，在查找新版本时（最多每小时一次）、打开版本列表或安装版本时</td><td data-label="发送到">nvm、uv、rustup 和 mise 的服务器：<code>nodejs.org</code>、<code>github.com</code> / <code>releases.astral.sh</code>（Python 构建）、<code>static.rust-lang.org</code>，对 mise 而言则是各语言自己的下载站点（例如 <code>go.dev</code>）</td><td data-label="内容">这些请求由工具自己发出，就像你在终端中运行它们一样。QuickDesk 不会添加任何内容。</td></tr>
<tr><td data-label="何时">运行时标签，仅当你点击 “安装 mise” 时</td><td data-label="发送到"><code>github.com</code>（jdx/mise 版本发布）；在 Windows 上还有 <code>api.github.com</code></td><td data-label="内容">下载最新的 mise 及其校验和文件。在 Windows 上，QuickDesk 会先向 GitHub API 查询哪个版本是最新的（User-Agent 为 “QuickDesk”）。</td></tr>
<tr><td data-label="何时">通过桌面门户自动粘贴</td><td data-label="发送到">你自己的桌面（GNOME），不经过网络</td><td data-label="内容">一次 Shift + Insert 按键。</td></tr>
</table>
<p>Codex 和 Antigravity 的用量从你电脑上的日志文件读取；不会发送任何内容。</p>
<h2>本网站</h2>
<p>quickdesk.click 是托管在 Cloudflare Pages 上的静态网站，没有 Cookie、没有分析、没有跟踪。它从 Google Fonts 加载字体，因此 Google 能看到你的 IP 地址。演示完全在你的浏览器中用示例数据运行。你选择的系统（Linux、Windows 或 macOS）以及你切换后的浅色或深色主题会保存在浏览器的本地存储中，不会发送到任何地方。</p>
<h2>下载计数</h2>
<p>为了了解有多少人安装和更新 QuickDesk，<code>dl.quickdesk.click</code> 会统计每次下载和更新检查：哪个文件（版本和系统）、发出请求的客户端类型（来自本网站或其他地方的浏览器、终端、PowerShell 或应用的更新程序），以及 Cloudflare 为该请求报告的国家。只保留每日总数（保存在 Cloudflare D1 数据库中）：不保存 IP 地址、浏览器标识、Cookie、其他标识符或逐条请求记录，因此这些计数无法关联到某个人或某台电脑。</p>
<h2>问题</h2>
<p>QuickDesk 是开源的：这里的每一条说明都可以在<a href="https://github.com/TankGum/quickdesk">源代码</a>中核实。有问题或要报告故障，请到 <a href="https://github.com/TankGum/quickdesk/issues">GitHub issues</a>。</p>`,
};

export default privacy;
