// /privacy. Keep this true to what the app does; update it with any new
// network connection or stored data.
import type { Lang } from "../i18n";

export const privacy: Record<Lang, { title: string; updated: string; html: string }> = {
  en: {
    title: "Privacy",
    updated: "Last updated 9 October 2026",
    html: `<p class="lead">QuickDesk has no accounts, no analytics and no telemetry. Your notes and clipboard history live on your computer. This page lists everything that leaves it.</p>
<h2>What stays on your computer</h2>
<ul>
<li><strong>Notes, clipboard history (text, images, file lists), settings and logs</strong> in <code>~/.local/share/click.quickdesk</code>.</li>
<li><strong>Clipboard history never leaves your computer</strong>, not even with sync on. Copies that password managers mark as secret are not saved, and you can pause saving at any time.</li>
<li><strong>Ports</strong> are read from <code>/proc</code> and Docker on your machine and are not stored.</li>
<li>The secret key of your sync bucket is kept in your system keyring.</li>
</ul>
<h2>What QuickDesk sends, and where</h2>
<table>
<tr><th>When</th><th>To</th><th>What</th></tr>
<tr><td data-label="When">Checking for updates (30 s after start, then every 6 hours; can be turned off)</td><td data-label="To"><code>dl.quickdesk.click</code> (Cloudflare)</td><td data-label="What">A request for <code>update.json</code>, and the package when you choose to update. Your IP address is visible to Cloudflare, as with any download.</td></tr>
<tr><td data-label="When">Note sync, only if you set it up</td><td data-label="To">The storage bucket <strong>you</strong> chose</td><td data-label="What">Your notes, encrypted on your computer (XChaCha20-Poly1305) with a key derived from your passphrase. The bucket owner, you, cannot read them without the passphrase or recovery key; neither can anyone else.</td></tr>
<tr><td data-label="When">AI usage, only if Claude Code is signed in on this computer</td><td data-label="To"><code>api.anthropic.com</code></td><td data-label="What">A usage request authorised with Claude Code's existing login, at most every 5 minutes. The token is read, never stored, refreshed or sent anywhere else.</td></tr>
<tr><td data-label="When">Auto-paste through the desktop portal</td><td data-label="To">Your own desktop (GNOME), not the network</td><td data-label="What">A Shift + Insert key press.</td></tr>
</table>
<p>Codex and Antigravity usage is read from log files on your computer; nothing is sent.</p>
<h2>This website</h2>
<p>quickdesk.click is a static site on Cloudflare Pages, with no cookies, no analytics and no tracking. It loads fonts from Google Fonts, so Google sees your IP address. The demo runs entirely in your browser on sample data.</p>
<h2>Questions</h2>
<p>QuickDesk is open source: every claim here can be checked in the <a href="https://github.com/TankGum/quickdesk">source code</a>. Ask or report a problem in <a href="https://github.com/TankGum/quickdesk/issues">GitHub issues</a>.</p>`,
  },
  vi: {
    title: "Quyền riêng tư",
    updated: "Cập nhật lần cuối ngày 9/10/2026",
    html: `<p class="lead">QuickDesk không có tài khoản, không có analytics và không gửi telemetry. Ghi chú và lịch sử clipboard nằm trên máy bạn. Trang này liệt kê mọi thứ được gửi ra ngoài.</p>
<h2>Những gì ở lại trên máy bạn</h2>
<ul>
<li><strong>Ghi chú, lịch sử clipboard (chữ, ảnh, danh sách file), cài đặt và log</strong> trong <code>~/.local/share/click.quickdesk</code>.</li>
<li><strong>Lịch sử clipboard không bao giờ rời khỏi máy</strong>, kể cả khi bật đồng bộ. Mục copy được trình quản lý mật khẩu đánh dấu bí mật sẽ không được lưu, và bạn có thể tạm dừng lưu bất cứ lúc nào.</li>
<li><strong>Cổng</strong> được đọc từ <code>/proc</code> và Docker trên máy bạn, không lưu lại.</li>
<li>Secret key của bucket đồng bộ được giữ trong keyring của hệ thống.</li>
</ul>
<h2>QuickDesk gửi gì, và gửi tới đâu</h2>
<table>
<tr><th>Khi nào</th><th>Tới đâu</th><th>Nội dung</th></tr>
<tr><td data-label="Khi nào">Kiểm tra bản mới (30 giây sau khi mở, rồi mỗi 6 giờ; có thể tắt)</td><td data-label="Tới đâu"><code>dl.quickdesk.click</code> (Cloudflare)</td><td data-label="Nội dung">Một yêu cầu tải <code>update.json</code>, và gói cài khi bạn chọn cập nhật. Cloudflare thấy địa chỉ IP của bạn, như mọi lần tải file.</td></tr>
<tr><td data-label="Khi nào">Đồng bộ ghi chú, chỉ khi bạn cài đặt</td><td data-label="Tới đâu">Bucket lưu trữ do <strong>bạn</strong> chọn</td><td data-label="Nội dung">Ghi chú đã được mã hoá trên máy (XChaCha20-Poly1305) bằng khoá sinh từ passphrase của bạn. Không ai đọc được nếu không có passphrase hoặc recovery key.</td></tr>
<tr><td data-label="Khi nào">Usage AI, chỉ khi Claude Code đã đăng nhập trên máy</td><td data-label="Tới đâu"><code>api.anthropic.com</code></td><td data-label="Nội dung">Một yêu cầu usage dùng phiên đăng nhập sẵn có của Claude Code, tối đa mỗi 5 phút. Token chỉ được đọc: không lưu, không làm mới, không gửi đi đâu khác.</td></tr>
<tr><td data-label="Khi nào">Tự động dán qua portal của desktop</td><td data-label="Tới đâu">Chính desktop của bạn (GNOME), không qua mạng</td><td data-label="Nội dung">Một lần bấm Shift + Insert.</td></tr>
</table>
<p>Usage của Codex và Antigravity được đọc từ file log trên máy; không gửi gì cả.</p>
<h2>Trang web này</h2>
<p>quickdesk.click là trang tĩnh trên Cloudflare Pages: không cookie, không analytics, không theo dõi. Trang tải font từ Google Fonts nên Google thấy địa chỉ IP của bạn. Demo chạy hoàn toàn trong trình duyệt với dữ liệu mẫu.</p>
<h2>Câu hỏi</h2>
<p>QuickDesk là mã nguồn mở: mọi điều ghi ở đây đều kiểm chứng được trong <a href="https://github.com/TankGum/quickdesk">mã nguồn</a>. Hỏi hoặc báo lỗi tại <a href="https://github.com/TankGum/quickdesk/issues">GitHub issues</a>.</p>`,
  },
};
