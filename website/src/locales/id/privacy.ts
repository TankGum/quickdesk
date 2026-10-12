// Privacy page in Indonesian. Keep it in step with src/data/privacy.ts.
const privacy = {
  title: "Privasi",
  updated: "Terakhir diperbarui: 11 Oktober 2026 (penghitungan unduhan)",
  html: `<p class="lead">QuickDesk tidak punya akun, analitik, maupun telemetri. Catatan dan riwayat clipboard Anda tinggal di komputer Anda. Halaman ini mencantumkan semua yang keluar darinya.</p>
<h2>Yang tetap di komputer Anda</h2>
<ul>
<li><strong>Catatan, riwayat clipboard (teks, gambar, daftar berkas), pengaturan, dan log</strong>, di <code>~/.local/share/click.quickdesk</code>.</li>
<li><strong>Riwayat clipboard tidak pernah keluar dari komputer Anda</strong>, bahkan saat sinkronisasi aktif. Apa pun yang ditandai rahasia oleh pengelola kata sandi tidak disimpan, dan Anda bisa menjeda penyimpanan kapan saja.</li>
<li><strong>Port</strong> dibaca dari <code>/proc</code> dan Docker di mesin Anda dan tidak disimpan.</li>
<li>Kode sinkronisasi dan kunci enkripsi Anda disimpan di keyring sistem.</li>
</ul>
<h2>Apa yang dikirim QuickDesk, dan ke mana</h2>
<table>
<tr><th>Kapan</th><th>Ke mana</th><th>Apa</th></tr>
<tr><td data-label="Kapan">Saat memeriksa pembaruan (30 detik setelah mulai, lalu setiap 6 jam; bisa dimatikan)</td><td data-label="Ke mana"><code>dl.quickdesk.click</code> (Cloudflare)</td><td data-label="Apa">Satu permintaan <code>update.json</code>, dan paketnya saat Anda memilih untuk memperbarui. Alamat IP Anda terlihat oleh Cloudflare, seperti pada unduhan apa pun.</td></tr>
<tr><td data-label="Kapan">Sinkronisasi catatan, hanya jika Anda mengaktifkannya</td><td data-label="Ke mana">QuickDesk Cloud, <code>sync.quickdesk.click</code> (Cloudflare)</td><td data-label="Apa">Catatan Anda, dienkripsi di komputer Anda (XChaCha20-Poly1305) dengan kunci yang diturunkan dari frasa sandi Anda; QuickDesk Cloud tidak bisa membacanya. Catatan disimpan di bawah ID akun sinkronisasi acak, beserta ukuran dan waktu unggahnya, dan QuickDesk Cloud melihat alamat IP Anda. Tanpa email, nama, atau data pribadi lainnya. Anda bisa menghapus semuanya dari Pengaturan → Sinkronisasi.</td></tr>
<tr><td data-label="Kapan">Penggunaan AI, hanya jika Claude Code sudah login di komputer ini (di Mac, hanya setelah Anda mengizinkan login-nya dibaca dari Rantai Kunci)</td><td data-label="Ke mana"><code>api.anthropic.com</code></td><td data-label="Apa">Satu permintaan penggunaan yang diotorisasi dengan login yang sudah dimiliki Claude Code, paling sering setiap 5 menit. Token dibaca; tidak pernah disimpan, diperbarui, atau dikirim ke tempat lain.</td></tr>
<tr><td data-label="Kapan">Tab Runtime, saat memeriksa rilis baru (paling sering sekali per jam), saat Anda membuka daftar versi, atau memasang satu versi</td><td data-label="Ke mana">Server nvm, uv, rustup, dan mise: <code>nodejs.org</code>, <code>github.com</code> / <code>releases.astral.sh</code> (build Python), <code>static.rust-lang.org</code>, dan untuk mise, situs unduhan masing-masing bahasa (misalnya <code>go.dev</code>)</td><td data-label="Apa">Alat-alat itu sendiri yang membuat permintaan, sama seperti saat Anda menjalankannya di terminal. QuickDesk tidak menambahkan apa pun.</td></tr>
<tr><td data-label="Kapan">Tab Runtime, hanya jika Anda mengklik “Pasang mise”</td><td data-label="Ke mana"><code>github.com</code> (rilis jdx/mise); di Windows juga <code>api.github.com</code></td><td data-label="Apa">Unduhan mise terbaru dan berkas checksum-nya. Di Windows, QuickDesk lebih dulu menanyakan rilis terbaru ke API GitHub (User-Agent “QuickDesk”).</td></tr>
<tr><td data-label="Kapan">Tempel otomatis lewat portal desktop</td><td data-label="Ke mana">Desktop Anda sendiri (GNOME), bukan jaringan</td><td data-label="Apa">Satu penekanan Shift + Insert.</td></tr>
</table>
<p>Penggunaan Codex dan Antigravity dibaca dari berkas log di komputer Anda; tidak ada yang dikirim.</p>
<h2>Situs web ini</h2>
<p>quickdesk.click adalah situs statis di Cloudflare Pages, tanpa cookie, tanpa analitik, dan tanpa pelacakan. Situs ini memuat font dari Google Fonts, jadi Google melihat alamat IP Anda. Demonya berjalan sepenuhnya di browser Anda dengan data contoh. Sistem yang Anda pilih (Linux, Windows, atau macOS) dan tema terang atau gelap, jika Anda menggantinya, diingat di penyimpanan lokal browser Anda dan tidak dikirim ke mana pun.</p>
<h2>Penghitungan unduhan</h2>
<p>Untuk mengetahui berapa orang yang memasang dan memperbarui QuickDesk, <code>dl.quickdesk.click</code> menghitung setiap unduhan dan pemeriksaan pembaruan: berkas mana (versi dan sistemnya), jenis klien yang memintanya (browser yang datang dari situs ini atau dari tempat lain, terminal, PowerShell, atau pembaru aplikasi), dan negara yang dilaporkan Cloudflare untuk permintaan itu. Hanya total harian yang disimpan (di database Cloudflare D1): tanpa alamat IP, string browser, cookie, pengenal lain, atau catatan per permintaan, sehingga angkanya tidak bisa dikaitkan dengan seseorang atau sebuah komputer.</p>
<h2>Pertanyaan</h2>
<p>QuickDesk adalah sumber terbuka: semua yang dikatakan di sini bisa diperiksa di <a href="https://github.com/TankGum/quickdesk">kode sumbernya</a>. Ajukan pertanyaan atau laporkan masalah di <a href="https://github.com/TankGum/quickdesk/issues">GitHub issues</a>.</p>`,
};

export default privacy;
