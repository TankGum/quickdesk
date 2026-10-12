// Privacy page in Japanese. Keep it in step with src/data/privacy.ts.
const privacy = {
  title: "プライバシー",
  updated: "最終更新：2026 年 10 月 11 日（ダウンロード数の集計）",
  html: `<p class="lead">QuickDesk にはアカウントも分析もテレメトリもありません。メモとクリップボード履歴はあなたのコンピューターにあります。このページでは、コンピューターの外に出るものをすべて挙げます。</p>
<h2>コンピューターに残るもの</h2>
<ul>
<li><strong>メモ、クリップボード履歴（テキスト、画像、ファイル一覧）、設定、ログ</strong>は <code>~/.local/share/click.quickdesk</code> にあります。</li>
<li><strong>クリップボード履歴がコンピューターの外に出ることはありません</strong>。同期をオンにしていても同じです。パスワードマネージャーが秘密として印を付けたコピーは保存されず、保存はいつでも一時停止できます。</li>
<li><strong>ポート</strong>はマシン上の <code>/proc</code> と Docker から読み取り、保存はしません。</li>
<li>同期コードと暗号鍵はシステムのキーリングに保存されます。</li>
</ul>
<h2>QuickDesk が何をどこへ送るか</h2>
<table>
<tr><th>いつ</th><th>送り先</th><th>内容</th></tr>
<tr><td data-label="いつ">アップデートの確認（起動 30 秒後、その後 6 時間ごと。オフにできます）</td><td data-label="送り先"><code>dl.quickdesk.click</code>（Cloudflare）</td><td data-label="内容"><code>update.json</code> のリクエストと、アップデートを選んだときのパッケージ。他のダウンロードと同様に、IP アドレスは Cloudflare から見えます。</td></tr>
<tr><td data-label="いつ">メモの同期（オンにした場合のみ）</td><td data-label="送り先">QuickDesk Cloud、<code>sync.quickdesk.click</code>（Cloudflare）</td><td data-label="内容">あなたのメモ。パスフレーズから導出した鍵でコンピューター上で暗号化済み（XChaCha20-Poly1305）で、QuickDesk Cloud は読めません。ランダムな同期アカウント ID のもとに、サイズとアップロード時刻とともに保存され、IP アドレスも見えます。メール、名前、その他の個人データはありません。Settings → Sync からすべて削除できます。</td></tr>
<tr><td data-label="いつ">AI 利用量（このコンピューターで Claude Code にログインしている場合のみ。Mac ではキーチェーンからのログイン情報の読み取りを許可したあとのみ）</td><td data-label="送り先"><code>api.anthropic.com</code></td><td data-label="内容">Claude Code の既存のログインで認証した利用量のリクエスト。最大 5 分に 1 回。トークンは読み取るだけで、保存、更新、他所への送信はしません。</td></tr>
<tr><td data-label="いつ">Runtimes タブで新しいリリースを探すとき（最大 1 時間に 1 回）、バージョン一覧を開くとき、バージョンをインストールするとき</td><td data-label="送り先">nvm、uv、rustup、mise のサーバー：<code>nodejs.org</code>、<code>github.com</code> / <code>releases.astral.sh</code>（Python のビルド）、<code>static.rust-lang.org</code>、mise の場合は各言語のダウンロードサイト（たとえば <code>go.dev</code>）</td><td data-label="内容">ターミナルで実行するときと同じく、リクエストはツール自身が送ります。QuickDesk は何も付け加えません。</td></tr>
<tr><td data-label="いつ">Runtimes タブで「Install mise」をクリックしたときのみ</td><td data-label="送り先"><code>github.com</code>（jdx/mise のリリース）。Windows では <code>api.github.com</code> も</td><td data-label="内容">最新の mise とそのチェックサムファイルのダウンロード。Windows では、QuickDesk がまず GitHub API に最新リリースを問い合わせます（User-Agent は「QuickDesk」）。</td></tr>
<tr><td data-label="いつ">デスクトップポータル経由の自動貼り付け</td><td data-label="送り先">あなた自身のデスクトップ（GNOME）。ネットワークではありません</td><td data-label="内容">Shift + Insert のキー入力。</td></tr>
</table>
<p>Codex と Antigravity の利用量はコンピューター上のログファイルから読み取り、何も送りません。</p>
<h2>このウェブサイト</h2>
<p>quickdesk.click は Cloudflare Pages 上の静的サイトで、Cookie も分析もトラッキングもありません。フォントを Google Fonts から読み込むため、Google には IP アドレスが見えます。デモはすべてブラウザ内でサンプルデータを使って動きます。選んだ OS（Linux、Windows、macOS）と、切り替えた場合のライト / ダークのテーマは、ブラウザのローカルストレージに保存され、どこにも送られません。</p>
<h2>ダウンロード数の集計</h2>
<p>QuickDesk をインストール・アップデートしている人の数を知るため、<code>dl.quickdesk.click</code> は各ダウンロードとアップデート確認を数えます：どのファイルか（バージョンと OS）、リクエストしたクライアントの種類（このサイトや他所から来たブラウザ、ターミナル、PowerShell、アプリのアップデーター）、そして Cloudflare がそのリクエストについて報告する国。保持するのは日ごとの合計だけです（Cloudflare D1 データベース）。IP アドレス、ブラウザの識別文字列、Cookie、その他の識別子、リクエストごとの記録は保存しないため、数値を個人やコンピューターと結びつけることはできません。</p>
<h2>お問い合わせ</h2>
<p>QuickDesk はオープンソースです。ここに書かれていることはすべて<a href="https://github.com/TankGum/quickdesk">ソースコード</a>で確かめられます。質問や問題の報告は <a href="https://github.com/TankGum/quickdesk/issues">GitHub issues</a> へどうぞ。</p>`,
};

export default privacy;
