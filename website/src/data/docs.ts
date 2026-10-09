// /docs content. Each section is HTML in both languages; keep facts in step
// with the app (defaults, limits, paths) when they change.
import type { Lang } from "../i18n";

export interface Section {
  id: string;
  title: Record<Lang, string>;
  html: Record<Lang, string>;
}

export const sections: Section[] = [
  {
    id: "install",
    title: { en: "Install", vi: "Cài đặt" },
    html: {
      en: `<p>Get the package for your distribution from the <a href="/download">download page</a>:</p>
<ul>
<li><strong>Ubuntu, Debian, Mint, Pop!_OS:</strong> <code>sudo apt install ./QuickDesk_&lt;version&gt;_amd64.deb</code></li>
<li><strong>Fedora, openSUSE:</strong> <code>sudo dnf install ./QuickDesk-&lt;version&gt;-1.x86_64.rpm</code></li>
<li><strong>Anything else:</strong> make the AppImage executable and run it. It needs FUSE 2 (<code>libfuse2</code>, on Ubuntu 24.04 <code>libfuse2t64</code>).</li>
</ul>
<p>QuickDesk needs an x86_64 system from the last few years (glibc 2.34 or newer). It works best on GNOME, on both Wayland and X11.</p>
<p>On first start a welcome screen lists the shortcuts and offers two options: <strong>start when you log in</strong> (needed for clipboard history and the AI ring to work all the time) and <strong>instant paste</strong> (see <a href="#auto-paste">Auto-paste</a>). After that QuickDesk lives in the top bar.</p>`,
      vi: `<p>Tải gói phù hợp với distro của bạn ở <a href="/vi/download">trang tải về</a>:</p>
<ul>
<li><strong>Ubuntu, Debian, Mint, Pop!_OS:</strong> <code>sudo apt install ./QuickDesk_&lt;phiên-bản&gt;_amd64.deb</code></li>
<li><strong>Fedora, openSUSE:</strong> <code>sudo dnf install ./QuickDesk-&lt;phiên-bản&gt;-1.x86_64.rpm</code></li>
<li><strong>Distro khác:</strong> cho file AppImage quyền chạy rồi mở nó. Cần FUSE 2 (<code>libfuse2</code>, trên Ubuntu 24.04 là <code>libfuse2t64</code>).</li>
</ul>
<p>QuickDesk cần máy x86_64 đời vài năm gần đây (glibc 2.34 trở lên). Chạy tốt nhất trên GNOME, cả Wayland lẫn X11.</p>
<p>Lần mở đầu tiên, màn hình chào hiện danh sách phím tắt và hai lựa chọn: <strong>tự chạy khi đăng nhập</strong> (cần để lịch sử clipboard và vòng tròn AI luôn hoạt động) và <strong>dán tức thì</strong> (xem <a href="#auto-paste">Tự động dán</a>). Sau đó QuickDesk nằm trên thanh trên cùng.</p>`,
    },
  },
  {
    id: "shortcuts",
    title: { en: "Shortcuts", vi: "Phím tắt" },
    html: {
      en: `<table>
<tr><th>Shortcut</th><th>Opens</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>Notes</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>Clipboard history popup</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>Ports</td></tr>
<tr><td>not set</td><td>Quick note popup (set one in Settings)</td></tr>
</table>
<p>Pressing a shortcut again while its window is open closes it. Change any of them in <strong>Settings → Hotkeys</strong>; QuickDesk warns when a combination is already taken.</p>
<p>On GNOME the shortcuts are added to <em>Settings → Keyboard → Custom Shortcuts</em> and removed when QuickDesk quits, so they work on Wayland too. On other desktops, bind <code>quickdesk toggle notes</code>, <code>quickdesk toggle clipboard</code>, <code>quickdesk toggle ports</code> or <code>quickdesk toggle quick-note</code> to keys in your desktop's keyboard settings.</p>
<p><strong>Why Super + Alt?</strong> Vietnamese input methods such as Unikey and Bamboo swallow Super + Shift + letter while you type in a text field; combinations with Alt keep working.</p>`,
      vi: `<table>
<tr><th>Phím tắt</th><th>Mở</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>Ghi chú</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>Popup lịch sử clipboard</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>Cổng</td></tr>
<tr><td>chưa đặt</td><td>Popup ghi chú nhanh (đặt trong Cài đặt)</td></tr>
</table>
<p>Bấm lại phím tắt khi cửa sổ đang mở là đóng nó. Đổi phím trong <strong>Cài đặt → Phím tắt</strong>; QuickDesk sẽ báo nếu tổ hợp đã bị dùng.</p>
<p>Trên GNOME, phím tắt được thêm vào <em>Settings → Keyboard → Custom Shortcuts</em> và gỡ đi khi thoát QuickDesk, nên chạy được cả trên Wayland. Trên desktop khác, hãy gán <code>quickdesk toggle notes</code>, <code>quickdesk toggle clipboard</code>, <code>quickdesk toggle ports</code> hoặc <code>quickdesk toggle quick-note</code> vào phím trong cài đặt bàn phím.</p>
<p><strong>Sao lại là Super + Alt?</strong> Bộ gõ tiếng Việt như Unikey, Bamboo nuốt mất Super + Shift + chữ khi bạn đang gõ trong ô nhập; tổ hợp có Alt thì vẫn chạy.</p>`,
    },
  },
  {
    id: "notes",
    title: { en: "Notes", vi: "Ghi chú" },
    html: {
      en: `<p>Notes are Markdown with a rich editor: headings, bold and italic, lists, checklists, quotes and code blocks. <kbd>Ctrl</kbd> <kbd>N</kbd> writes a new note, <kbd>Ctrl</kbd> <kbd>S</kbd> saves, <kbd>Ctrl</kbd> <kbd>F</kbd> searches. Search ignores accents, so <code>tieng viet</code> finds “tiếng Việt”.</p>
<p>Pin a note with the star to keep it on top. Deleting asks first and can be undone for a few seconds.</p>`,
      vi: `<p>Ghi chú dùng Markdown với trình soạn đầy đủ: tiêu đề, in đậm và nghiêng, danh sách, checklist, trích dẫn, khối code. <kbd>Ctrl</kbd> <kbd>N</kbd> để viết ghi chú mới, <kbd>Ctrl</kbd> <kbd>S</kbd> để lưu, <kbd>Ctrl</kbd> <kbd>F</kbd> để tìm. Tìm kiếm không cần dấu: gõ <code>tieng viet</code> vẫn ra “tiếng Việt”.</p>
<p>Bấm ngôi sao để ghim ghi chú lên đầu. Xoá sẽ hỏi trước và có thể hoàn tác trong vài giây.</p>`,
    },
  },
  {
    id: "sync",
    title: { en: "Sync notes between computers", vi: "Đồng bộ ghi chú giữa các máy" },
    html: {
      en: `<p>Notes sync through <strong>QuickDesk Cloud</strong>. There is no account or email: a computer gets a <strong>sync code</strong>, and your other computers join with it. Only notes sync; clipboard history and settings stay on each computer.</p>
<h3>Set it up</h3>
<ol>
<li>On the first computer: <strong>Settings → Sync → Turn on sync</strong>, and choose a passphrase (at least 8 characters).</li>
<li>QuickDesk shows your <strong>sync code</strong> (<code>QD1-…</code>) and a <strong>recovery key</strong>. Save the recovery key somewhere safe; it is shown only once.</li>
<li>On every other computer: <strong>Settings → Sync → I have a sync code</strong>, then enter the code and the same passphrase (or the recovery key).</li>
</ol>
<p>The sync code can be shown again any time with <strong>Show sync code</strong> under Settings → Sync.</p>
<h3>When it syncs</h3>
<ul>
<li>A couple of seconds after you edit a note, and when you open a QuickDesk window.</li>
<li>Otherwise QuickDesk checks for changes every minute while a window is open, and every 5 minutes while it sits in the tray. A check is a single small request; notes only travel when something changed.</li>
<li><strong>Sync now</strong> checks straight away. Edits made offline are sent when you are back online.</li>
<li>If a note changed on two computers at once, both versions are kept: the newer one stays, the other becomes a copy next to it. An edit wins over a delete.</li>
</ul>
<h3>Privacy and keys</h3>
<ul>
<li>Notes are encrypted on your computer before upload (XChaCha20-Poly1305, key from your passphrase via Argon2id). QuickDesk Cloud only holds ciphertext under a random account id and cannot read your notes.</li>
<li>It takes <strong>both</strong> the sync code (to reach the data) and the passphrase (to decrypt it). Treat the sync code like a password: with it alone, someone could still delete your synced notes.</li>
<li>Forgot the passphrase? Enter the recovery key instead. Without either, the copy in QuickDesk Cloud cannot be opened, but the notes on each computer are untouched.</li>
<li>If QuickDesk asks to <strong>Unlock sync</strong> (for example after the system keyring was reset), enter the passphrase again.</li>
</ul>
<h3>Turning it off</h3>
<p><strong>Turn off sync…</strong> stops syncing on one computer and keeps its notes. Tick <em>Also delete my notes from QuickDesk Cloud</em> to remove the synced copy for every computer.</p>
<p>Limits per sync code: 256 MB and 20,000 stored objects, far more than notes need.</p>`,
      vi: `<p>Ghi chú đồng bộ qua <strong>QuickDesk Cloud</strong>. Không cần tài khoản hay email: một máy nhận <strong>mã đồng bộ</strong>, các máy khác nhập mã đó để tham gia. Chỉ ghi chú được đồng bộ; lịch sử clipboard và cài đặt ở lại trên từng máy.</p>
<h3>Thiết lập</h3>
<ol>
<li>Trên máy đầu tiên: <strong>Cài đặt → Đồng bộ → Bật đồng bộ</strong>, rồi đặt passphrase (ít nhất 8 ký tự).</li>
<li>QuickDesk hiện <strong>mã đồng bộ</strong> (<code>QD1-…</code>) và <strong>recovery key</strong>. Hãy cất recovery key ở nơi an toàn; nó chỉ hiện một lần.</li>
<li>Trên mỗi máy khác: <strong>Cài đặt → Đồng bộ → Tôi đã có mã đồng bộ</strong>, rồi nhập mã và cùng passphrase (hoặc recovery key).</li>
</ol>
<p>Mã đồng bộ có thể xem lại bất cứ lúc nào bằng <strong>Hiện mã đồng bộ</strong> trong Cài đặt → Đồng bộ.</p>
<h3>Khi nào đồng bộ</h3>
<ul>
<li>Vài giây sau khi bạn sửa ghi chú, và khi bạn mở một cửa sổ QuickDesk.</li>
<li>Ngoài ra QuickDesk kiểm tra thay đổi mỗi phút khi có cửa sổ đang mở, và mỗi 5 phút khi chỉ chạy ở khay hệ thống. Mỗi lần kiểm tra chỉ là một request nhỏ; ghi chú chỉ được gửi hoặc tải khi có thay đổi.</li>
<li><strong>Đồng bộ ngay</strong> kiểm tra luôn. Chỉnh sửa lúc mất mạng sẽ được gửi khi có mạng lại.</li>
<li>Nếu một ghi chú bị sửa cùng lúc trên hai máy, cả hai bản đều được giữ: bản mới hơn giữ chỗ, bản còn lại thành một bản sao bên cạnh. Sửa thắng xoá.</li>
</ul>
<h3>Quyền riêng tư và khoá</h3>
<ul>
<li>Ghi chú được mã hoá trên máy trước khi tải lên (XChaCha20-Poly1305, khoá sinh từ passphrase bằng Argon2id). QuickDesk Cloud chỉ giữ dữ liệu đã mã hoá theo một mã tài khoản ngẫu nhiên và không đọc được ghi chú của bạn.</li>
<li>Cần <strong>cả hai</strong>: mã đồng bộ (để tới được dữ liệu) và passphrase (để giải mã). Hãy giữ mã đồng bộ như mật khẩu: chỉ có mã thôi, người khác vẫn có thể xoá ghi chú đã đồng bộ.</li>
<li>Quên passphrase? Nhập recovery key thay thế. Mất cả hai thì bản trên QuickDesk Cloud không mở được nữa, nhưng ghi chú trên từng máy vẫn nguyên vẹn.</li>
<li>Nếu QuickDesk yêu cầu <strong>Mở khoá đồng bộ</strong> (ví dụ sau khi keyring của hệ thống bị đặt lại), hãy nhập lại passphrase.</li>
</ul>
<h3>Tắt đồng bộ</h3>
<p><strong>Tắt đồng bộ…</strong> ngừng đồng bộ trên một máy và giữ nguyên ghi chú trên máy đó. Tích <em>Xoá luôn ghi chú trên QuickDesk Cloud</em> để xoá bản đồng bộ cho mọi máy.</p>
<p>Giới hạn cho mỗi mã đồng bộ: 256 MB và 20.000 object, dư sức cho ghi chú.</p>`,
    },
  },
  {
    id: "clipboard",
    title: { en: "Clipboard history", vi: "Lịch sử clipboard" },
    html: {
      en: `<p>Everything you copy is saved: text, images (screenshots) and files copied in your file manager. Open the popup with <kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd>, type to search, filter by Text, Images or Files, and press <kbd>Enter</kbd> to paste into the app you were using. <kbd>Ctrl</kbd> <kbd>Enter</kbd> only copies, <kbd>Ctrl</kbd> <kbd>P</kbd> pins, <kbd>Del</kbd> deletes.</p>
<ul>
<li>History stays on this computer and never syncs.</li>
<li>Copies that password managers mark as secret are skipped. Before copying something sensitive, <strong>pause saving</strong> for 15 minutes, an hour or until you turn it back on (in the Clipboard tab or the tray menu).</li>
<li>QuickDesk keeps the latest 1,000 entries; pinned ones are kept forever. Text up to 1 MB, images up to 10 MB each and 200 MB in total.</li>
<li><strong>Clear…</strong> in the Clipboard tab removes everything except pinned entries.</li>
</ul>`,
      vi: `<p>Mọi thứ bạn copy đều được lưu: chữ, ảnh (ảnh chụp màn hình) và file copy từ trình quản lý file. Mở popup bằng <kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd>, gõ để tìm, lọc theo Chữ, Ảnh hoặc File, rồi bấm <kbd>Enter</kbd> để dán vào app bạn đang dùng. <kbd>Ctrl</kbd> <kbd>Enter</kbd> chỉ copy, <kbd>Ctrl</kbd> <kbd>P</kbd> để ghim, <kbd>Del</kbd> để xoá.</p>
<ul>
<li>Lịch sử chỉ nằm trên máy này, không bao giờ đồng bộ.</li>
<li>Mục copy được trình quản lý mật khẩu đánh dấu bí mật sẽ bị bỏ qua. Trước khi copy thứ nhạy cảm, hãy <strong>tạm dừng lưu</strong> 15 phút, 1 giờ hoặc tới khi bật lại (trong tab Clipboard hoặc menu trên khay hệ thống).</li>
<li>QuickDesk giữ 1.000 mục gần nhất; mục đã ghim được giữ mãi. Chữ tối đa 1 MB, ảnh tối đa 10 MB mỗi ảnh và 200 MB tổng cộng.</li>
<li><strong>Xoá lịch sử…</strong> trong tab Clipboard xoá hết, trừ các mục đã ghim.</li>
</ul>`,
    },
  },
  {
    id: "auto-paste",
    title: { en: "Auto-paste", vi: "Tự động dán" },
    html: {
      en: `<p>When you pick an entry, QuickDesk returns to the app you were using and presses <kbd>Shift</kbd> <kbd>Insert</kbd>, which also works in terminals. Choose how in <strong>Settings → Auto-paste</strong>:</p>
<ul>
<li><strong>Desktop portal</strong> (default): GNOME asks for permission once and shows a remote-control indicator while pasting.</li>
<li><strong>Instant (virtual keyboard)</strong>: no prompts or indicator. Enabling it asks for your administrator password once to allow programs running as you to create a virtual keyboard (the same permission Steam uses for controllers).</li>
</ul>
<p>Turn auto-paste off in the Clipboard tab to only copy, then press <kbd>Ctrl</kbd> <kbd>V</kbd> yourself.</p>`,
      vi: `<p>Khi bạn chọn một mục, QuickDesk quay lại app bạn đang dùng và bấm <kbd>Shift</kbd> <kbd>Insert</kbd>, chạy được cả trong terminal. Chọn cách dán trong <strong>Cài đặt → Tự động dán</strong>:</p>
<ul>
<li><strong>Portal của desktop</strong> (mặc định): GNOME hỏi quyền một lần và hiện biểu tượng điều khiển từ xa trong lúc dán.</li>
<li><strong>Tức thì (bàn phím ảo)</strong>: không hỏi, không hiện biểu tượng. Lúc bật sẽ hỏi mật khẩu quản trị một lần để cho phép chương trình của bạn tạo bàn phím ảo (cùng quyền Steam dùng cho tay cầm).</li>
</ul>
<p>Tắt tự động dán trong tab Clipboard nếu chỉ muốn copy, rồi tự bấm <kbd>Ctrl</kbd> <kbd>V</kbd>.</p>`,
    },
  },
  {
    id: "ports",
    title: { en: "Ports", vi: "Cổng" },
    html: {
      en: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> lists every listening TCP port (IPv4 and IPv6) with the process, its full command line and user, refreshed every 2 seconds. Filter by port, PID, process or container name.</p>
<ul>
<li><strong>Open</strong> opens <code>http://localhost:&lt;port&gt;</code> in your browser; <strong>Copy PID</strong> and <strong>Copy cmd</strong> copy them.</li>
<li><strong>Kill</strong> asks first, then stops the process politely and offers a force kill only if it does not quit.</li>
<li>Ports published by Docker show the container and image, and <strong>Stop</strong> stops the container instead of killing <code>docker-proxy</code>.</li>
<li>Processes owned by other users (system services) show as hidden; inspecting them needs root.</li>
</ul>`,
      vi: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> liệt kê mọi cổng TCP đang lắng nghe (IPv4 và IPv6) cùng tiến trình, dòng lệnh đầy đủ và user, làm mới mỗi 2 giây. Lọc theo cổng, PID, tiến trình hoặc tên container.</p>
<ul>
<li><strong>Open</strong> mở <code>http://localhost:&lt;cổng&gt;</code> trên trình duyệt; <strong>Copy PID</strong> và <strong>Copy cmd</strong> để copy.</li>
<li><strong>Kill</strong> hỏi trước, dừng tiến trình nhẹ nhàng và chỉ đề nghị force kill khi nó không chịu thoát.</li>
<li>Cổng do Docker mở hiện tên container và image, và <strong>Stop</strong> dừng container thay vì kill <code>docker-proxy</code>.</li>
<li>Tiến trình của user khác (dịch vụ hệ thống) hiện dạng ẩn; xem chi tiết cần quyền root.</li>
</ul>`,
    },
  },
  {
    id: "ai",
    title: { en: "AI usage", vi: "Usage AI" },
    html: {
      en: `<p>A ring in the top bar shows how much of your AI limits you have used, coloured green, amber or red. Its menu shows the tool you track, with a bar for each limit and when it resets; the AI tab shows every tool.</p>
<ul>
<li><strong>Claude Code:</strong> QuickDesk reads the login Claude Code already saved on this computer and asks Anthropic for your usage, at most every 5 minutes. The token is only read: never stored, refreshed or sent anywhere except Anthropic.</li>
<li><strong>Codex:</strong> read from the session logs Codex keeps in <code>~/.codex</code>.</li>
<li><strong>Gemini (Antigravity):</strong> it reports no percentage, so QuickDesk shows when you last ran out of quota, from its logs.</li>
</ul>
<p>Choose which tool and limit to follow under <em>Track</em>, or hide the ring in the AI tab.</p>`,
      vi: `<p>Một vòng tròn trên thanh trên cùng cho biết bạn đã dùng bao nhiêu hạn mức AI, đổi màu xanh, cam hoặc đỏ. Menu của nó hiện công cụ bạn đang theo dõi, mỗi hạn mức có một thanh tiến trình và lúc đặt lại; tab AI hiện mọi công cụ.</p>
<ul>
<li><strong>Claude Code:</strong> QuickDesk đọc phiên đăng nhập Claude Code đã lưu trên máy và hỏi Anthropic số liệu usage, tối đa mỗi 5 phút một lần. Token chỉ được đọc: không lưu lại, không làm mới và không gửi đi đâu ngoài Anthropic.</li>
<li><strong>Codex:</strong> đọc từ log phiên làm việc Codex lưu trong <code>~/.codex</code>.</li>
<li><strong>Gemini (Antigravity):</strong> không báo phần trăm, nên QuickDesk hiện lần cuối bạn hết quota, lấy từ log của nó.</li>
</ul>
<p>Chọn công cụ và hạn mức cần theo dõi trong <em>Theo dõi</em>, hoặc ẩn vòng tròn trong tab AI.</p>`,
    },
  },
  {
    id: "updates",
    title: { en: "Updates", vi: "Cập nhật" },
    html: {
      en: `<p>QuickDesk checks for a new version 30 seconds after it starts and then every 6 hours. When one is out, a banner in the main window and an entry at the top of the tray menu say so, with <strong>What’s New</strong>. Nothing is installed until you click <strong>Update Now</strong>.</p>
<p>Every update is verified against QuickDesk's signing key before it is installed. A .deb or .rpm asks for your administrator password; an AppImage replaces itself. QuickDesk then restarts and shows what changed. Turn automatic checks off, or check by hand, in <strong>Settings → Updates</strong>.</p>`,
      vi: `<p>QuickDesk kiểm tra bản mới 30 giây sau khi khởi động, rồi cứ mỗi 6 giờ. Khi có bản mới, banner trong cửa sổ chính và một mục ở đầu menu khay hệ thống sẽ báo, kèm <strong>Có gì mới</strong>. Không có gì được cài cho tới khi bạn bấm <strong>Cập nhật ngay</strong>.</p>
<p>Mỗi bản cập nhật đều được kiểm tra chữ ký số của QuickDesk trước khi cài. Gói .deb hoặc .rpm sẽ hỏi mật khẩu quản trị; AppImage tự thay chính nó. Sau đó QuickDesk khởi động lại và cho bạn xem có gì thay đổi. Tắt tự động kiểm tra, hoặc tự kiểm tra, trong <strong>Cài đặt → Cập nhật</strong>.</p>`,
    },
  },
  {
    id: "uninstall",
    title: { en: "Uninstall", vi: "Gỡ cài đặt" },
    html: {
      en: `<ol>
<li>Turn off <em>Start QuickDesk when I log in</em> in Settings, and quit from the tray.</li>
<li>Remove the package: <code>sudo apt remove quick-desk</code> (Debian/Ubuntu) or <code>sudo dnf remove QuickDesk</code> (Fedora), or delete the AppImage.</li>
<li>Your data stays in <code>~/.local/share/click.quickdesk</code> until you delete that folder.</li>
</ol>
<p>Removing the package also removes the instant-paste permission if you enabled it.</p>`,
      vi: `<ol>
<li>Tắt <em>Tự chạy QuickDesk khi đăng nhập</em> trong Cài đặt, rồi thoát app từ khay hệ thống.</li>
<li>Gỡ gói: <code>sudo apt remove quick-desk</code> (Debian/Ubuntu) hoặc <code>sudo dnf remove QuickDesk</code> (Fedora), hoặc xoá file AppImage.</li>
<li>Dữ liệu vẫn nằm trong <code>~/.local/share/click.quickdesk</code> cho tới khi bạn xoá thư mục đó.</li>
</ol>
<p>Gỡ gói cũng gỡ luôn quyền dán tức thì nếu bạn đã bật.</p>`,
    },
  },
  {
    id: "troubleshooting",
    title: { en: "Troubleshooting", vi: "Khắc phục sự cố" },
    html: {
      en: `<ul>
<li><strong>A shortcut does nothing:</strong> check <em>Settings → Keyboard → Custom Shortcuts</em> for a clash, or pick another combination in Settings → Hotkeys.</li>
<li><strong>Auto-paste types nothing:</strong> make sure the app you were in still has focus; switch the method in Settings → Auto-paste. The entry is always copied, so <kbd>Ctrl</kbd> <kbd>V</kbd> works.</li>
<li><strong>Logs:</strong> <code>~/.local/share/click.quickdesk/logs/</code>. Attach them when you <a href="https://github.com/TankGum/quickdesk/issues">report an issue</a>.</li>
</ul>`,
      vi: `<ul>
<li><strong>Phím tắt không có tác dụng:</strong> xem <em>Settings → Keyboard → Custom Shortcuts</em> có bị trùng không, hoặc chọn tổ hợp khác trong Cài đặt → Phím tắt.</li>
<li><strong>Tự động dán không gõ gì:</strong> kiểm tra app bạn đang dùng vẫn còn được focus; đổi cách dán trong Cài đặt → Tự động dán. Mục đã chọn luôn được copy, nên <kbd>Ctrl</kbd> <kbd>V</kbd> vẫn dùng được.</li>
<li><strong>Log:</strong> <code>~/.local/share/click.quickdesk/logs/</code>. Đính kèm khi bạn <a href="https://github.com/TankGum/quickdesk/issues">báo lỗi</a>.</li>
</ul>`,
    },
  },
];
