// Docs in Japanese, by section id (src/data/docs.ts).
import { macBeta } from "../../data/release";

const docs: Record<string, { title: string; html: string }> = {
  install: {
    title: "インストール",
    html: `<p>いちばん手早い方法（どのディストリビューションでも）：</p>
<pre><code>curl -fsSL https://quickdesk.click/install.sh | bash</code></pre>
<p>お使いのシステムに合った .deb、.rpm、AppImage を選び、SHA-256 を確認してインストールします（<a href="/install.sh">スクリプトを読む</a>）。または<a href="/download">ダウンロードページ</a>からパッケージを取得し、ダウンロードしたファイルを次のようにインストールします：</p>
<ul>
<li><strong>Ubuntu、Debian、Mint、Pop!_OS：</strong> <code>sudo apt install ~/Downloads/QuickDesk_&lt;version&gt;_amd64.deb</code></li>
<li><strong>Fedora、openSUSE：</strong> <code>sudo dnf install ~/Downloads/QuickDesk-&lt;version&gt;-1.x86_64.rpm</code></li>
<li><strong>その他：</strong> AppImage に実行権限を付けて起動します。FUSE 2（<code>libfuse2</code>、Ubuntu 24.04 では <code>libfuse2t64</code>）が必要です。</li>
</ul>
<p>QuickDesk にはここ数年の x86_64 システム（glibc 2.34 以降）が必要です。GNOME で最もよく動き、Wayland と X11 のどちらにも対応します。</p>
<p><strong>Windows 10 と 11（64 ビット）：</strong> PowerShell で <code>irm https://quickdesk.click/install.ps1 | iex</code> を実行するか、<a href="/download">ダウンロードページ</a>からインストーラーを入手します。管理者権限なしで、ご自分のユーザーにのみインストールされます。インストーラーにはまだ署名がないため、Windows が「PC が保護されました」と表示することがあります。<em>詳細情報 → 実行</em>を選んでください。Windows でのショートカットは <kbd>Win</kbd> <kbd>Alt</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd> で、QuickDesk はタスクトレイに常駐し、自動貼り付けは設定不要です。Runtimes タブは mise、rustup、uv を通じて動きます（<em>言語のバージョン</em>を参照）。</p>
<p>初回起動時にはようこそ画面がショートカットを案内し、2 つのオプションを提示します：<strong>ログイン時に起動</strong>（クリップボード履歴と AI リングを常に動かすのに必要）と<strong>即時貼り付け</strong>（<a href="#auto-paste">自動貼り付け</a>を参照）。その後、QuickDesk はトップバーに常駐します。</p>`,
  },
  macos: {
    title: "macOS",
    html: `${macBeta ? '<p class="note"><strong>まだ開発中です。</strong>Mac 版は使えますが、まだ新しいため、すべての Mac ですべてを試せてはいません。自動貼り付けとショートカットは改良を続けています。うまく動かないことがあれば<a href="https://github.com/TankGum/quickdesk/issues">お知らせください</a>。</p>\n' : ""}<p>QuickDesk は Apple シリコン（M1 以降）の macOS 11 以降で動きます。ターミナルで <code>curl -fsSL https://quickdesk.click/install.sh | bash</code> を実行すると、QuickDesk.app を「アプリケーション」に置いて起動します。または<a href="/download#macos">ダウンロードページ</a>から <code>.dmg</code> を入手し、QuickDesk を「アプリケーション」へドラッグします。</p>
<p><strong>初めて開くとき。</strong>QuickDesk はまだ Apple の公証を受けていないため、<code>.dmg</code> をブラウザでダウンロードした場合、macOS は初回起動時に「マルウェアが含まれていないことを Apple は検証できません」と表示して止めます。アプリには何の問題もありません。次のどちらかを一度行ってください（コマンドでインストールした場合は警告は出ません）：</p>
<ol>
<li>「アプリケーション」から QuickDesk を開き、macOS が警告したら<em>完了</em>をクリックします。次に「システム設定 → プライバシーとセキュリティ」を開いて「セキュリティ」までスクロールし、「Mac を保護するために “QuickDesk” がブロックされました」の横の<em>このまま開く</em>をクリックし、パスワードまたは Touch ID で確認して<em>開く</em>をクリックします。macOS 14 以前では、QuickDesk を Control キーを押しながらクリック → <em>開く</em> → <em>開く</em> でも開けます。</li>
<li>またはターミナルで <code>xattr -dr com.apple.quarantine /Applications/QuickDesk.app</code> を実行してから、いつもどおり QuickDesk を開きます。</li>
</ol>
<p><a href="/download#macos-open">ダウンロードページ</a>にも同じ手順があります。</p>
<ul>
<li>QuickDesk はメニューバーに常駐し、Dock にアイコンは出ません。ポップアップは今いる操作スペースに開き、フルスクリーンのアプリの上にも表示されます。</li>
<li>ショートカットは <kbd>⌃</kbd> <kbd>⌥</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd>（Control + Option）です。⌘⌥V と ⌘⌥N は Finder が使っています。</li>
<li><strong>自動貼り付け</strong>は使っていたアプリで ⌘V を押します。macOS がこれを許すのは、「システム設定 → プライバシーとセキュリティ → <strong>アクセシビリティ</strong>」に QuickDesk を追加したあとだけです。初めて項目を選ぶときに macOS が尋ねます。それまでは項目がコピーされるので、ご自分で ⌘V を押してください。許可の状態は Settings → Auto-paste で確認できます。</li>
<li>新しい macOS では、他のアプリがコピーした内容を QuickDesk が読み取ってよいか尋ねられることがあります。「許可」を選ばないと、クリップボード履歴は空のままです。</li>
<li><strong>Claude の利用量：</strong>Mac では Claude Code がログイン情報をキーチェーンに保存します。AI タブがそこから読み取ることを提案し、macOS が一度だけ尋ねます（<em>常に許可</em>を選びます）。他の OS と同じく、ログイン情報は読み取るだけで、<code>api.anthropic.com</code> にのみ送られます。</li>
<li>アプリはアドホック署名のため、アップデート後に macOS がアクセシビリティとキーチェーンの許可を再び求めることがあります。自動貼り付けが動かなくなったら、アクセシビリティの一覧から QuickDesk を削除して追加し直してください。</li>
<li>Runtimes タブは Linux と同じように、ログインシェル（zsh：<code>~/.zshrc</code>）で動きます。Homebrew でインストールしたものは表示されますが、変更はしません。</li>
</ul>
<p><strong>アンインストール：</strong>メニューバーから QuickDesk を終了し、「アプリケーション」からゴミ箱へ移動します。データは <code>~/Library/Application Support/click.quickdesk</code> に、そのフォルダーを削除するまで残ります。</p>`,
  },
  shortcuts: {
    title: "ショートカット",
    html: `<table>
<tr><th>ショートカット</th><th>開くもの</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>メモ</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>クリップボード履歴のポップアップ</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>ポート</td></tr>
<tr><td>未設定</td><td>クイックメモのポップアップ（Settings で設定）</td></tr>
</table>
<p>ウィンドウが開いているときにもう一度ショートカットを押すと閉じます。どれも <strong>Settings → Hotkeys</strong> で変更でき、組み合わせがすでに使われていれば QuickDesk が警告します。</p>
<p>GNOME ではショートカットが<em>設定 → キーボード → カスタムショートカット</em>に追加され、QuickDesk の終了時に削除されるので、Wayland でも動きます。他のデスクトップでは、キーボード設定で <code>quickdesk toggle notes</code>、<code>quickdesk toggle clipboard</code>、<code>quickdesk toggle ports</code>、<code>quickdesk toggle quick-note</code> をキーに割り当ててください。</p>
<p><strong>なぜ Super + Alt？</strong>Unikey や Bamboo などのベトナム語入力メソッドは、テキスト欄で入力中に Super + Shift + 文字を横取りしてしまいますが、Alt を使った組み合わせは影響を受けません。</p>`,
  },
  notes: {
    title: "メモ",
    html: `<p>メモはリッチエディター付きの Markdown です：見出し、太字と斜体、リスト、チェックリスト、引用、コードブロック。<kbd>Ctrl</kbd> <kbd>N</kbd> で新しいメモ、<kbd>Ctrl</kbd> <kbd>S</kbd> で保存、<kbd>Ctrl</kbd> <kbd>F</kbd> で検索します。検索はアクセント記号を区別しないので、<code>tieng viet</code> で「tiếng Việt」が見つかります。</p>
<p>星を付けるとメモが上に固定されます。削除の前には確認があり、数秒間は元に戻せます。</p>`,
  },
  sync: {
    title: "コンピューター間でメモを同期",
    html: `<p>メモは <strong>QuickDesk Cloud</strong> を通じて同期されます。アカウントやメールは不要です。1 台のコンピューターが<strong>同期コード</strong>を受け取り、他のコンピューターはそれで参加します。同期されるのはメモだけで、クリップボード履歴と設定はそれぞれのコンピューターに残ります。</p>
<h3>設定する</h3>
<ol>
<li>1 台目のコンピューターで：<strong>Settings → Sync → Turn on sync</strong> を選び、パスフレーズ（8 文字以上）を決めます。</li>
<li>QuickDesk が<strong>同期コード</strong>（<code>QD1-…</code>）と<strong>復旧キー</strong>を表示します。復旧キーは安全な場所に保管してください。表示されるのは一度だけです。</li>
<li>他のすべてのコンピューターで：<strong>Settings → Sync → I have a sync code</strong> を選び、コードと同じパスフレーズ（または復旧キー）を入力します。</li>
</ol>
<p>同期コードは、Settings → Sync の <strong>Show sync code</strong> でいつでも再表示できます。</p>
<h3>同期のタイミング</h3>
<ul>
<li>メモを編集した数秒後と、QuickDesk のウィンドウを開いたとき。</li>
<li>それ以外は、ウィンドウが開いている間は 1 分ごと、トレイにあるときは 5 分ごとに変更を確認します。確認は小さなリクエスト 1 つだけで、メモが送られるのは何かが変わったときだけです。</li>
<li><strong>Sync now</strong> ですぐに確認します。オフラインでの編集は、オンラインに戻ったときに送られます。</li>
<li>同じメモを 2 台で同時に変更した場合は両方のバージョンが残ります。新しい方がそのまま残り、もう一方はその隣にコピーとして置かれます。削除より編集が優先されます。</li>
</ul>
<h3>プライバシーと鍵</h3>
<ul>
<li>メモはアップロード前にコンピューター上で暗号化されます（XChaCha20-Poly1305、鍵はパスフレーズから Argon2id で導出）。QuickDesk Cloud はランダムなアカウント ID のもとに暗号文を保持するだけで、メモを読むことはできません。</li>
<li>データにたどり着くための同期コードと、復号するためのパスフレーズの<strong>両方</strong>が必要です。同期コードはパスワードと同じように扱ってください。同期コードだけでも、同期済みのメモを削除されるおそれがあります。</li>
<li>パスフレーズを忘れたら、代わりに復旧キーを入力します。どちらもなければ QuickDesk Cloud のコピーは開けませんが、各コンピューター上のメモはそのままです。</li>
<li>QuickDesk が <strong>Unlock sync</strong> を求めたら（たとえばシステムのキーリングがリセットされたあと）、もう一度パスフレーズを入力してください。</li>
</ul>
<h3>同期をオフにする</h3>
<p><strong>Turn off sync…</strong> はそのコンピューターでの同期を止め、メモはそのまま残します。<em>Also delete my notes from QuickDesk Cloud</em> にチェックを入れると、すべてのコンピューターの同期済みコピーを削除します。</p>
<p>同期コードごとの上限：256 MB と 20,000 個の保存オブジェクト。メモには十分すぎる量です。</p>`,
  },
  clipboard: {
    title: "クリップボード履歴",
    html: `<p>コピーしたものはすべて保存されます：テキスト、画像（スクリーンショット）、ファイルマネージャーでコピーしたファイル。<kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd> でポップアップを開き、入力して検索し、テキスト・画像・ファイルで絞り込み、<kbd>Enter</kbd> で使っていたアプリに貼り付けます。<kbd>Ctrl</kbd> <kbd>Enter</kbd> はコピーのみ、<kbd>Ctrl</kbd> <kbd>P</kbd> はピン留め、<kbd>Del</kbd> は削除です。</p>
<ul>
<li>履歴はこのコンピューターに残り、同期はされません。</li>
<li>パスワードマネージャーが秘密として印を付けたコピーは保存されません。機密情報をコピーする前に、<strong>保存を一時停止</strong>できます（15 分、1 時間、またはオンに戻すまで。Clipboard タブかトレイメニューから）。</li>
<li>QuickDesk は最新の 1,000 件を保持し、ピン留めしたものは永久に保持します。テキストは 1 MB まで、画像は 1 枚 10 MB、合計 200 MB までです。</li>
<li>Clipboard タブの <strong>Clear…</strong> は、ピン留めした項目以外をすべて削除します。</li>
</ul>`,
  },
  "auto-paste": {
    title: "自動貼り付け",
    html: `<p>項目を選ぶと、QuickDesk は使っていたアプリに戻って <kbd>Shift</kbd> <kbd>Insert</kbd> を押します。これはターミナルでも動きます。方法は <strong>Settings → Auto-paste</strong> で選べます：</p>
<ul>
<li><strong>Desktop portal</strong>（デフォルト）：GNOME が一度だけ許可を求め、貼り付け中はリモート操作のインジケーターを表示します。</li>
<li><strong>Instant (virtual keyboard)</strong>：確認もインジケーターもありません。有効にするときに一度だけ管理者パスワードを入力し、あなたの権限で動くプログラムが仮想キーボードを作れるようにします（Steam がコントローラーに使うのと同じ権限です）。</li>
</ul>
<p>Clipboard タブで自動貼り付けをオフにするとコピーだけになるので、ご自分で <kbd>Ctrl</kbd> <kbd>V</kbd> を押してください。</p>`,
  },
  ports: {
    title: "ポート",
    html: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> で、待ち受け中のすべての TCP ポート（IPv4 と IPv6）を、プロセス、完全なコマンドライン、ユーザーとともに一覧表示し、2 秒ごとに更新します。ポート、PID、プロセス名、コンテナ名で絞り込めます。</p>
<ul>
<li><strong>Open</strong> はブラウザで <code>http://localhost:&lt;port&gt;</code> を開きます。<strong>Copy PID</strong> と <strong>Copy cmd</strong> はそれぞれをコピーします。</li>
<li><strong>Kill</strong> はまず確認し、プロセスを穏やかに停止します。終了しないときだけ強制終了を提案します。</li>
<li>Docker が公開しているポートにはコンテナとイメージが表示され、<strong>Stop</strong> は <code>docker-proxy</code> を強制終了する代わりにコンテナを停止します。</li>
<li>他のユーザー（システムサービス）のプロセスは非表示として示されます。詳しく見るには root 権限が必要です。</li>
</ul>`,
  },
  versions: {
    title: "言語のバージョン（Runtimes）",
    html: `<p><strong>Runtimes</strong> タブでは、このコンピューターのプログラミング言語（Node.js、Python、Rust、Go、Java、Ruby、Deno、Bun、PHP）のバージョンを管理します。<em>新しいターミナル</em>で動くバージョンと、その出どころを表示します。QuickDesk はログインシェルに問い合わせるので、表示されるのはデスクトップから見えるものではなく、<code>~/.bashrc</code> や <code>~/.zshrc</code> を読み込んだあとの結果です。</p>
<ul>
<li><strong>nvm</strong>（Node.js）、<strong>rustup</strong>（Rust）、<strong>uv</strong>（Python）、<strong>mise</strong>（Go、Java、Ruby、Deno、Bun、PHP など）でインストールした言語は変更できます。<strong>Install version…</strong> はツールが提供するバージョンから選び（rustup は入力したチャンネルかバージョン）、<strong>Make default</strong> は新しいターミナルで使うものを設定し、<strong>Uninstall</strong> はバージョンを削除して容量を空けます。デフォルトは削除できないので、先に別のものをデフォルトにしてください。</li>
<li>インストールは進捗バー付きでバックグラウンドで実行され、キャンセルもできます。一度に実行される変更は 1 つだけです。バージョン一覧はデフォルトで各系列の最新リリースを表示します。LTS やすべてに切り替えたり、入力して絞り込んだりできます。</li>
<li>インストール済みの系列に新しいリリースが出ると（22.23.3 に対する Node 22.24.1、3.13.7 に対する Python 3.13.9）、そのバージョンに <strong>↑ 22.24.1</strong> が表示されます。クリックひとつでインストールし、古い方がデフォルトだった場合は新しい方をデフォルトにします。古いバージョンはアンインストールするまで残ります。Rust のチャンネルはその場で更新されます。rustup に新しい stable があると <strong>stable</strong> に <strong>↑ 1.93.0</strong> が表示され、クリックで <code>rustup update stable</code> を実行します。</li>
<li><strong>Apply right away in open terminals</strong>（タブの上部）を使うと、変更がすでに開いているターミナルにも次のプロンプトから反映されます。QuickDesk はシェルファイルに 1 行追加し、小さなプロンプトフック（<code>~/.config/quickdesk/shell/hook.sh</code>）を読み込みます。このフックは、QuickDesk がバージョンを変えたあとにシェルがコマンドを探し直すようにし、プロジェクトに <code>cd</code> したときに Node がいちばん近い <code>.nvmrc</code> に従うようにします（なければ選んだデフォルトに。手動の <code>nvm use</code> はフォルダーかデフォルトを変えるまで有効です）。これを使わない場合でも Rust の変更はすぐに反映され、Python と mise の変更は新しいターミナル（または開いているターミナルで <code>hash -r</code> のあと）に、nvm の変更は新しいターミナルにだけ反映されます。</li>
<li><strong>Set for a project…</strong> は、選んだフォルダーにツール自身のファイルを書き込みます：<code>.nvmrc</code>、<code>rust-toolchain.toml</code>、<code>.python-version</code>、<code>mise.toml</code>（mise は記載済みの他のツールを残します）。</li>
<li>バージョンマネージャーのない言語（たとえば apt で入れた Go）には <strong>Install mise…</strong> が表示されます。QuickDesk は最新の mise を github.com/jdx/mise からダウンロードし、リリースの SHA-256 と照合して <code>~/.local/bin</code> に置きます。バージョンをデフォルトにしたあと、<strong>Fix PATH…</strong> がシェルで mise を有効にする行を提案します。</li>
<li>新しいターミナルがデフォルトとは別のものを実行してしまう場合（たとえばシステムの <code>python3</code> が PATH で先に来る場合）、カードがその理由を示し、<strong>Fix PATH…</strong> が追加する行をそのまま表示します。あなたが同意したときだけ、QuickDesk はそれを印の付いたブロックとしてシェルファイルの末尾に追加し、隣にバックアップを残します。ブロックは同じ場所から削除できます。</li>
<li>PATH の修正とプロンプトフックは bash（<code>~/.bashrc</code>）と zsh（<code>~/.zshrc</code>）向けです。fish にはまだ対応していません。</li>
<li>まだコンピューターにない言語は <strong>Add a language</strong> に一覧されます。選ぶと、必要なら mise をインストールしてからバージョンを選べます。</li>
<li><strong>Windows：</strong>QuickDesk は新しいターミナルで何が動くかを PowerShell に問い合わせ、<strong>mise</strong>（<code>%LOCALAPPDATA%\\mise\\bin</code> にインストール）、<strong>rustup</strong>、<strong>uv</strong> を通じて動きます。nvm-windows と fnm は表示されますが変更はせず、ストアの <code>python.exe</code> のプレースホルダーは無視します。<strong>Fix PATH…</strong> は、ツールのフォルダー（mise の場合は shims）を、管理者権限なしでアカウントの PATH（設定 → 環境変数 にあるもの）の先頭に追加します。以前の値は <code>~\\.config\\quickdesk\\path</code> に保存され、同じパネルからフォルダーを再び取り除けます。新しいターミナルがそれを読み込み、以後はデフォルトの変更がすぐに反映されます。Windows では「Apply right away」は不要なので表示されません。</li>
<li>システムがインストールしたもの（apt、dnf）は変更も削除もしません。QuickDesk が <code>sudo</code> を実行することはありません。対応していないツール（fnm、volta、pyenv、SDKMAN…）で管理されている言語は読み取り専用で表示します。</li>
</ul>`,
  },
  ai: {
    title: "AI 利用量",
    html: `<p>トップバーのリングが、AI の利用上限をどれだけ使ったかを緑・琥珀・赤で示します。そのメニューには追跡中のツールが表示され、上限ごとのバーとリセット時刻が並びます。AI タブにはすべてのツールが表示されます。</p>
<ul>
<li><strong>Claude Code：</strong>QuickDesk は Claude Code がこのコンピューターに保存済みのログイン情報を読み取り、最大 5 分に 1 回、Anthropic に利用量を問い合わせます。トークンは読み取るだけで、保存も更新もせず、Anthropic 以外のどこにも送りません。</li>
<li><strong>Codex：</strong>Codex が <code>~/.codex</code> に残すセッションログから読み取ります。</li>
<li><strong>Gemini（Antigravity）：</strong>割合を報告しないため、QuickDesk はそのログから最後に割り当てを使い切った時刻を表示します。</li>
</ul>
<p>追跡するツールと上限は <em>Track</em> で選べます。リングは AI タブで非表示にできます。</p>`,
  },
  updates: {
    title: "アップデート",
    html: `<p>QuickDesk は起動 30 秒後に新しいバージョンを確認し、その後は 6 時間ごとに確認します。新しいバージョンが出ると、メインウィンドウのバナーとトレイメニューの先頭の項目が <strong>What’s New</strong> とともに知らせます。<strong>Update Now</strong> をクリックするまで何もインストールされません。</p>
<p>アップデートはインストール前に必ず QuickDesk の署名鍵で検証されます。.deb や .rpm は管理者パスワードを求め、AppImage は自分自身を置き換えます。その後 QuickDesk が再起動し、変更点を表示します。自動確認をオフにしたり手動で確認したりするには <strong>Settings → Updates</strong> を使います。</p>`,
  },
  uninstall: {
    title: "アンインストール",
    html: `<ol>
<li>Settings で <em>Start QuickDesk when I log in</em> をオフにし、トレイから終了します。</li>
<li>パッケージを削除します：<code>sudo apt remove quick-desk</code>（Debian/Ubuntu）または <code>sudo dnf remove QuickDesk</code>（Fedora）、または AppImage を削除します。</li>
<li>データは <code>~/.local/share/click.quickdesk</code> に、そのフォルダーを削除するまで残ります。</li>
</ol>
<p><strong>Windows：</strong>設定 → アプリ → インストールされているアプリ → QuickDesk → アンインストール。データは <code>%APPDATA%\\click.quickdesk</code> に、そのフォルダーを削除するまで残ります。</p>
<p>パッケージを削除すると、有効にしていた即時貼り付けの権限も取り除かれます。</p>`,
  },
  troubleshooting: {
    title: "トラブルシューティング",
    html: `<ul>
<li><strong>ショートカットが反応しない：</strong><em>設定 → キーボード → カスタムショートカット</em>で衝突がないか確認するか、Settings → Hotkeys で別の組み合わせを選びます。</li>
<li><strong>自動貼り付けで何も入力されない：</strong>使っていたアプリにまだフォーカスがあるか確認し、Settings → Auto-paste で方法を切り替えます。項目は必ずコピーされるので、<kbd>Ctrl</kbd> <kbd>V</kbd> で貼り付けられます。</li>
<li><strong>ログ：</strong><code>~/.local/share/click.quickdesk/logs/</code>。<a href="https://github.com/TankGum/quickdesk/issues">問題を報告</a>するときに添付してください。</li>
</ul>`,
  },
};

export default docs;
