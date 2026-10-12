// Docs in Indonesian, by section id (src/data/docs.ts).
import { macBeta } from "../../data/release";

const docs: Record<string, { title: string; html: string }> = {
  install: {
    title: "Instalasi",
    html: `<p>Cara tercepat, di distribusi mana pun:</p>
<pre><code>curl -fsSL https://quickdesk.click/install.sh | bash</code></pre>
<p>Perintah ini memilih .deb, .rpm, atau AppImage untuk sistem Anda, memeriksa SHA-256-nya, lalu memasangnya (<a href="/install.sh">baca skripnya</a>). Atau ambil paket di <a href="/download">halaman unduhan</a> dan pasang berkas yang diunduh dengan:</p>
<ul>
<li><strong>Ubuntu, Debian, Mint, Pop!_OS:</strong> <code>sudo apt install ~/Downloads/QuickDesk_&lt;version&gt;_amd64.deb</code></li>
<li><strong>Fedora, openSUSE:</strong> <code>sudo dnf install ~/Downloads/QuickDesk-&lt;version&gt;-1.x86_64.rpm</code></li>
<li><strong>Distribusi lain:</strong> jadikan AppImage dapat dieksekusi lalu buka. AppImage membutuhkan FUSE 2 (<code>libfuse2</code>; di Ubuntu 24.04, <code>libfuse2t64</code>).</li>
</ul>
<p>QuickDesk membutuhkan sistem x86_64 dari beberapa tahun terakhir (glibc 2.34 atau lebih baru). Paling cocok di GNOME, baik di Wayland maupun X11.</p>
<p><strong>Windows 10 dan 11 (64-bit):</strong> di PowerShell, jalankan <code>irm https://quickdesk.click/install.ps1 | iex</code>, atau unduh penginstal dari <a href="/download">halaman unduhan</a>. QuickDesk dipasang hanya untuk pengguna Anda, tanpa hak administrator. Penginstal belum ditandatangani, jadi Windows mungkin mengatakan telah melindungi PC Anda: pilih <em>Info selengkapnya → Tetap jalankan</em>. Di Windows, pintasannya <kbd>Win</kbd> <kbd>Alt</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd>, QuickDesk berada di system tray, dan tempel otomatis tidak perlu diatur. Di sana tab Runtimes bekerja dengan mise, rustup, dan uv (lihat <em>Versi bahasa pemrograman</em>).</p>
<p>Saat pertama kali, layar sambutan menampilkan pintasan dan menawarkan dua pilihan: <strong>mulai saat login</strong> (diperlukan agar riwayat clipboard dan cincin AI selalu berjalan) dan <strong>tempel instan</strong> (lihat <a href="#auto-paste">Tempel otomatis</a>). Setelah itu, QuickDesk berada di bar atas.</p>`,
  },
  macos: {
    title: "macOS",
    html: `${macBeta ? '<p class="note"><strong>Masih dalam pengembangan.</strong> Versi Mac sudah berjalan, tetapi masih baru: belum semuanya diuji di setiap Mac, dan tempel otomatis serta pintasan masih terus disempurnakan. Mohon <a href="https://github.com/TankGum/quickdesk/issues">beri tahu kami</a> apa yang tidak berfungsi.</p>\n' : ""}<p>QuickDesk berjalan di macOS 11 atau lebih baru di Apple Silicon (M1 atau lebih baru). Di Terminal, jalankan <code>curl -fsSL https://quickdesk.click/install.sh | bash</code>: perintah ini menaruh QuickDesk.app di Aplikasi lalu membukanya. Atau unduh <code>.dmg</code> dari <a href="/download#macos">halaman unduhan</a> lalu seret QuickDesk ke Aplikasi.</p>
<p><strong>Membukanya pertama kali.</strong> QuickDesk belum dinotarisasi oleh Apple, jadi jika <code>.dmg</code> diunduh lewat browser, macOS menghentikan pembukaan pertama dengan peringatan bahwa “Apple tidak dapat memverifikasi bahwa aplikasi ini bebas dari malware”. Tidak ada yang salah dengan aplikasinya; lakukan salah satu dari dua cara ini sekali (jika Anda memasang dengan perintah, tidak ada peringatan yang muncul):</p>
<ol>
<li>Buka QuickDesk dari Aplikasi dan tutup peringatan macOS. Lalu buka Pengaturan Sistem → Privasi &amp; Keamanan, gulir ke Keamanan, klik <em>Tetap Buka</em> di samping ““QuickDesk” diblokir untuk melindungi Mac Anda”, konfirmasi dengan kata sandi atau Touch ID, lalu klik <em>Buka</em>. Di macOS 14 atau versi lebih lama, Control-klik QuickDesk → <em>Buka</em> → <em>Buka</em> juga bisa.</li>
<li>Atau jalankan <code>xattr -dr com.apple.quarantine /Applications/QuickDesk.app</code> di Terminal lalu buka QuickDesk seperti biasa.</li>
</ol>
<p><a href="/download#macos-open">Halaman unduhan</a> menampilkan langkah yang sama.</p>
<ul>
<li>QuickDesk berada di bar menu, tanpa ikon di Dock. Jendela pop-up-nya terbuka di Space tempat Anda berada, juga di atas aplikasi layar penuh.</li>
<li>Pintasannya <kbd>⌃</kbd> <kbd>⌥</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd> (Control + Option): ⌘⌥V dan ⌘⌥N milik Finder.</li>
<li><strong>Tempel otomatis</strong> menekan ⌘V di aplikasi yang sedang Anda pakai, dan macOS hanya mengizinkannya setelah Anda menambahkan QuickDesk di Pengaturan Sistem → Privasi &amp; Keamanan → <strong>Aksesibilitas</strong>. macOS menanyakannya saat pertama kali Anda memilih entri; sampai saat itu entri disalin dan Anda menekan ⌘V sendiri. Settings → Auto-paste menunjukkan apakah izinnya sudah diberikan.</li>
<li>Versi macOS terbaru mungkin bertanya apakah QuickDesk boleh membaca apa yang disalin aplikasi lain. Pilih Izinkan, atau riwayat clipboard akan tetap kosong.</li>
<li><strong>Penggunaan Claude:</strong> di Mac, Claude Code menyimpan login-nya di Rantai Kunci. Tab AI menawarkan untuk membacanya dari sana; macOS lalu bertanya sekali (pilih <em>Selalu Izinkan</em>). Seperti di sistem lain, login hanya dibaca dan hanya dikirim ke <code>api.anthropic.com</code>.</li>
<li>Karena aplikasinya ditandatangani secara ad hoc, macOS mungkin meminta izin Aksesibilitas dan Rantai Kunci lagi setelah pembaruan. Jika tempel otomatis berhenti berfungsi, hapus QuickDesk dari daftar Aksesibilitas lalu tambahkan lagi.</li>
<li>Tab Runtimes bekerja seperti di Linux, dengan shell login Anda (zsh: <code>~/.zshrc</code>). Salinan yang dipasang dengan Homebrew ditampilkan tetapi tidak diubah.</li>
</ul>
<p><strong>Mencopot:</strong> keluar dari QuickDesk lewat bar menu lalu pindahkan dari Aplikasi ke Tempat Sampah. Data Anda tetap di <code>~/Library/Application Support/click.quickdesk</code> sampai Anda menghapus folder itu.</p>`,
  },
  shortcuts: {
    title: "Pintasan",
    html: `<table>
<tr><th>Pintasan</th><th>Membuka</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>Catatan</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>Jendela riwayat clipboard</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>Port</td></tr>
<tr><td>belum diatur</td><td>Jendela catatan cepat (atur di Settings)</td></tr>
</table>
<p>Menekan pintasan lagi saat jendelanya terbuka akan menutupnya. Ubah pintasan mana pun di <strong>Settings → Hotkeys</strong>; QuickDesk memberi tahu jika kombinasi sudah dipakai.</p>
<p>Di GNOME, pintasan ditambahkan ke <em>Pengaturan → Papan Ketik → Pintasan Khusus</em> dan dihapus saat QuickDesk ditutup, jadi juga berfungsi di Wayland. Di desktop lain, kaitkan <code>quickdesk toggle notes</code>, <code>quickdesk toggle clipboard</code>, <code>quickdesk toggle ports</code>, atau <code>quickdesk toggle quick-note</code> ke tombol di pengaturan keyboard desktop Anda.</p>
<p><strong>Kenapa Super + Alt?</strong> Metode input bahasa Vietnam seperti Unikey dan Bamboo menelan Super + Shift + huruf saat Anda mengetik di kolom teks; kombinasi dengan Alt tetap berfungsi.</p>`,
  },
  notes: {
    title: "Catatan",
    html: `<p>Catatan adalah Markdown dengan editor kaya: judul, tebal dan miring, daftar, daftar tugas, kutipan, dan blok kode. <kbd>Ctrl</kbd> <kbd>N</kbd> membuat catatan baru, <kbd>Ctrl</kbd> <kbd>S</kbd> menyimpan, <kbd>Ctrl</kbd> <kbd>F</kbd> mencari. Pencarian mengabaikan tanda diakritik, jadi <code>tieng viet</code> menemukan “tiếng Việt”.</p>
<p>Sematkan catatan dengan bintang agar tetap di atas. Menghapus meminta konfirmasi dan bisa dibatalkan selama beberapa detik.</p>`,
  },
  sync: {
    title: "Menyinkronkan catatan antarkomputer",
    html: `<p>Catatan disinkronkan lewat <strong>QuickDesk Cloud</strong>. Tanpa akun dan tanpa email: satu komputer mendapat <strong>kode sinkronisasi</strong> dan komputer lain bergabung dengannya. Hanya catatan yang disinkronkan; riwayat clipboard dan pengaturan tetap di masing-masing komputer.</p>
<h3>Mengaturnya</h3>
<ol>
<li>Di komputer pertama: <strong>Settings → Sync → Turn on sync</strong>, lalu pilih frasa sandi (minimal 8 karakter).</li>
<li>QuickDesk menampilkan <strong>kode sinkronisasi</strong> Anda (<code>QD1-…</code>) dan <strong>kunci pemulihan</strong>. Simpan kunci pemulihan di tempat aman; kunci itu hanya ditampilkan sekali.</li>
<li>Di setiap komputer lain: <strong>Settings → Sync → I have a sync code</strong>, lalu masukkan kode dan frasa sandi yang sama (atau kunci pemulihan).</li>
</ol>
<p>Anda bisa melihat kode itu lagi kapan saja dengan <strong>Show sync code</strong> di Settings → Sync.</p>
<h3>Kapan sinkronisasi berjalan</h3>
<ul>
<li>Beberapa detik setelah Anda mengedit catatan, dan saat Anda membuka jendela QuickDesk.</li>
<li>Selain itu, QuickDesk memeriksa perubahan setiap menit saat ada jendela terbuka, dan setiap 5 menit saat berada di tray. Setiap pemeriksaan hanya satu permintaan kecil; catatan baru dikirim jika ada yang berubah.</li>
<li><strong>Sync now</strong> memeriksa saat itu juga. Perubahan yang dibuat saat offline dikirim begitu Anda online lagi.</li>
<li>Jika satu catatan berubah di dua komputer sekaligus, kedua versi disimpan: yang lebih baru tetap, yang lain menjadi salinan di sebelahnya. Suntingan menang atas penghapusan.</li>
</ul>
<h3>Privasi dan kunci</h3>
<ul>
<li>Catatan dienkripsi di komputer Anda sebelum diunggah (XChaCha20-Poly1305, kunci diturunkan dari frasa sandi Anda dengan Argon2id). QuickDesk Cloud hanya menyimpan teks terenkripsi di bawah ID akun acak dan tidak bisa membaca catatan Anda.</li>
<li>Diperlukan <strong>keduanya</strong>: kode sinkronisasi (untuk menjangkau data) dan frasa sandi (untuk mendekripsinya). Perlakukan kode itu seperti kata sandi: dengan kode itu saja, seseorang bisa menghapus catatan Anda yang tersinkron.</li>
<li>Lupa frasa sandi? Masukkan kunci pemulihan. Tanpa keduanya, salinan di QuickDesk Cloud tidak bisa dibuka, tetapi catatan di setiap komputer tidak tersentuh.</li>
<li>Jika QuickDesk meminta <strong>Unlock sync</strong> (misalnya setelah keyring sistem diatur ulang), masukkan lagi frasa sandi Anda.</li>
</ul>
<h3>Mematikannya</h3>
<p><strong>Turn off sync…</strong> menghentikan sinkronisasi di satu komputer dan tetap menyimpan catatannya. Centang <em>Also delete my notes from QuickDesk Cloud</em> untuk menghapus salinan tersinkron bagi semua komputer.</p>
<p>Batas per kode sinkronisasi: 256 MB dan 20.000 objek tersimpan, jauh lebih banyak daripada yang dibutuhkan catatan.</p>`,
  },
  clipboard: {
    title: "Riwayat clipboard",
    html: `<p>Semua yang Anda salin disimpan: teks, gambar (tangkapan layar), dan berkas yang disalin di pengelola berkas. Buka jendelanya dengan <kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd>, ketik untuk mencari, saring berdasarkan teks, gambar, atau berkas, lalu tekan <kbd>Enter</kbd> untuk menempel ke aplikasi yang sedang Anda pakai. <kbd>Ctrl</kbd> <kbd>Enter</kbd> hanya menyalin, <kbd>Ctrl</kbd> <kbd>P</kbd> menyematkan, <kbd>Delete</kbd> menghapus.</p>
<ul>
<li>Riwayat tetap di komputer ini dan tidak pernah disinkronkan.</li>
<li>Apa pun yang ditandai rahasia oleh pengelola kata sandi dilewati. Sebelum menyalin sesuatu yang sensitif, <strong>jeda penyimpanan</strong> selama 15 menit, satu jam, atau sampai Anda mengaktifkannya lagi (di tab Clipboard atau menu tray).</li>
<li>QuickDesk menyimpan 1.000 entri terbaru; yang disematkan, selamanya. Teks hingga 1 MB, gambar hingga 10 MB per gambar dan 200 MB secara total.</li>
<li><strong>Clear…</strong> di tab Clipboard menghapus semuanya kecuali entri yang disematkan.</li>
</ul>`,
  },
  "auto-paste": {
    title: "Tempel otomatis",
    html: `<p>Saat Anda memilih entri, QuickDesk kembali ke aplikasi yang sedang Anda pakai dan menekan <kbd>Shift</kbd> <kbd>Insert</kbd>, yang juga berfungsi di terminal. Pilih caranya di <strong>Settings → Auto-paste</strong>:</p>
<ul>
<li><strong>Desktop portal</strong> (default): GNOME meminta izin sekali dan menampilkan indikator kendali jarak jauh saat menempel.</li>
<li><strong>Instant (virtual keyboard)</strong>: tanpa permintaan izin dan tanpa indikator. Saat diaktifkan, meminta kata sandi admin sekali untuk mengizinkan program yang Anda jalankan membuat keyboard virtual (izin yang sama yang dipakai Steam untuk controller).</li>
</ul>
<p>Matikan tempel otomatis di tab Clipboard untuk hanya menyalin, lalu tekan <kbd>Ctrl</kbd> <kbd>V</kbd> sendiri.</p>`,
  },
  ports: {
    title: "Port",
    html: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> menampilkan setiap port TCP yang mendengarkan (IPv4 dan IPv6) beserta prosesnya, baris perintah lengkapnya, dan penggunanya, diperbarui setiap 2 detik. Saring berdasarkan port, PID, proses, atau nama kontainer.</p>
<ul>
<li><strong>Open</strong> membuka <code>http://localhost:&lt;port&gt;</code> di browser Anda; <strong>Copy PID</strong> dan <strong>Copy cmd</strong> menyalinnya.</li>
<li><strong>Kill</strong> bertanya dulu, lalu menghentikan proses dengan sopan dan baru menawarkan paksa jika proses tidak berhenti.</li>
<li>Port yang dipublikasikan Docker menampilkan kontainer dan image-nya, dan <strong>Stop</strong> menghentikan kontainer alih-alih mematikan <code>docker-proxy</code>.</li>
<li>Proses milik pengguna lain (layanan sistem) ditampilkan tersembunyi; memeriksanya membutuhkan root.</li>
</ul>`,
  },
  versions: {
    title: "Versi bahasa pemrograman (Runtimes)",
    html: `<p>Tab <strong>Runtimes</strong> mengelola versi bahasa pemrograman di komputer ini (Node.js, Python, Rust, Go, Java, Ruby, Deno, Bun, PHP). Tab ini menampilkan versi yang dijalankan <em>terminal baru</em> dan dari mana asalnya. QuickDesk bertanya ke shell login Anda, jadi inilah yang Anda dapatkan setelah <code>~/.bashrc</code> atau <code>~/.zshrc</code>, bukan yang dilihat desktop.</p>
<ul>
<li>Bahasa yang dipasang dengan <strong>nvm</strong> (Node.js), <strong>rustup</strong> (Rust), <strong>uv</strong> (Python), atau <strong>mise</strong> (Go, Java, Ruby, Deno, Bun, PHP, dan lainnya) bisa diubah: <strong>Install version…</strong> memilih dari rilis yang ditawarkan alat itu (rustup menerima kanal atau versi yang Anda ketik), <strong>Make default</strong> menetapkan yang dipakai terminal baru, <strong>Uninstall</strong> menghapus satu versi dan membebaskan ruangnya. Versi default tidak bisa dihapus; pilih default lain dulu.</li>
<li>Pemasangan berjalan di latar belakang dengan bilah kemajuan dan bisa dibatalkan. Hanya satu perubahan yang berjalan dalam satu waktu. Daftar versi secara default menampilkan rilis terbaru dari setiap seri; ganti ke LTS atau semua, atau ketik untuk menyaring.</li>
<li>Saat ada rilis lebih baru dari seri yang terpasang (Node 22.24.1 untuk 22.23.3, Python 3.13.9 untuk 3.13.7), versi itu menampilkan <strong>↑ 22.24.1</strong>: sekali klik memasangnya dan, jika versi lama adalah default, menjadikan yang baru default. Versi lama tetap ada sampai Anda mencopotnya. Kanal Rust diperbarui di tempat: <strong>stable</strong> menampilkan <strong>↑ 1.93.0</strong> saat rustup punya stable yang lebih baru, dan klik menjalankan <code>rustup update stable</code>.</li>
<li><strong>Apply right away in open terminals</strong> (di bagian atas tab) membuat perubahan sampai ke terminal yang sudah terbuka, mulai dari prompt berikutnya: QuickDesk menambahkan satu baris ke berkas shell Anda yang memuat hook prompt kecil (<code>~/.config/quickdesk/shell/hook.sh</code>). Hook ini membuat shell mencari ulang perintah setelah QuickDesk mengubah versi, dan membuat Node mengikuti <code>.nvmrc</code> terdekat saat Anda <code>cd</code> ke sebuah proyek, atau default yang Anda pilih jika tidak ada (<code>nvm use</code> manual tetap berlaku sampai Anda pindah folder atau mengganti default). Tanpa hook ini, perubahan Rust tetap langsung berlaku, perubahan Python dan mise sampai ke terminal baru (atau terminal terbuka setelah <code>hash -r</code>), dan perubahan nvm hanya ke terminal baru.</li>
<li><strong>Set for a project…</strong> menulis berkas milik alat itu di folder yang Anda pilih: <code>.nvmrc</code>, <code>rust-toolchain.toml</code>, <code>.python-version</code>, atau <code>mise.toml</code> (mise mempertahankan alat lain yang sudah tercantum di sana).</li>
<li>Bahasa tanpa pengelola versi (misalnya Go yang dipasang dengan apt) menawarkan <strong>Install mise…</strong>: QuickDesk mengunduh mise terbaru dari github.com/jdx/mise, memeriksanya dengan checksum SHA-256 rilis itu, lalu menaruhnya di <code>~/.local/bin</code>. Setelah Anda menjadikan sebuah versi default, <strong>Fix PATH…</strong> menawarkan baris yang mengaktifkan mise di shell Anda.</li>
<li>Jika terminal baru akan menjalankan salinan yang berbeda dari default (misalnya <code>python3</code> sistem lebih dulu di PATH), kartunya menjelaskan alasannya dan <strong>Fix PATH…</strong> menampilkan baris persis yang akan ditambahkan. Hanya setelah Anda setuju, QuickDesk menambahkannya sebagai blok bertanda di akhir berkas shell Anda, dengan cadangan di sebelahnya; blok itu bisa dihapus dari tempat yang sama.</li>
<li>Perbaikan PATH dan hook prompt ditulis untuk bash (<code>~/.bashrc</code>) dan zsh (<code>~/.zshrc</code>); fish belum didukung.</li>
<li>Bahasa yang belum ada di komputer muncul di <strong>Add a language</strong>: pilih satu untuk memasang mise jika perlu, lalu pilih versinya.</li>
<li><strong>Windows:</strong> QuickDesk bertanya ke PowerShell apa yang dijalankan terminal baru dan bekerja dengan <strong>mise</strong> (dipasang di <code>%LOCALAPPDATA%\\mise\\bin</code>), <strong>rustup</strong>, dan <strong>uv</strong>; nvm-windows dan fnm ditampilkan tetapi tidak diubah, dan <code>python.exe</code> penampung dari Microsoft Store diabaikan. <strong>Fix PATH…</strong> menaruh folder alat itu (untuk mise, shim-nya) di depan PATH akun Anda, yang ada di Pengaturan → Variabel lingkungan, tanpa hak administrator; nilai sebelumnya disimpan di <code>~\\.config\\quickdesk\\path</code> dan panel yang sama bisa menghapus folder itu lagi. Terminal baru akan memakainya; setelah itu, perubahan default langsung berlaku. Di Windows, “Apply right away” tidak diperlukan dan tidak ditampilkan.</li>
<li>Salinan yang dipasang sistem (apt, dnf) tidak pernah diubah atau dihapus; QuickDesk tidak pernah menjalankan <code>sudo</code>. Bahasa yang dikelola alat yang tidak didukung (fnm, volta, pyenv, SDKMAN…) ditampilkan hanya-baca.</li>
</ul>`,
  },
  ai: {
    title: "Penggunaan AI",
    html: `<p>Sebuah cincin di bar atas menunjukkan seberapa banyak batas AI yang sudah Anda pakai, dalam warna hijau, kuning, atau merah. Menunya menampilkan alat yang Anda pantau, dengan satu bar per batas dan kapan batas itu diatur ulang; tab AI menampilkan semua alat.</p>
<ul>
<li><strong>Claude Code:</strong> QuickDesk membaca login yang sudah disimpan Claude Code di komputer ini dan menanyakan penggunaan Anda ke Anthropic, paling sering setiap 5 menit. Token hanya dibaca: tidak pernah disimpan, diperbarui, atau dikirim ke mana pun selain Anthropic.</li>
<li><strong>Codex:</strong> dibaca dari log sesi yang disimpan Codex di <code>~/.codex</code>.</li>
<li><strong>Gemini (Antigravity):</strong> tidak melaporkan persentase, jadi QuickDesk menampilkan, dari log-nya, kapan terakhir kali Anda kehabisan kuota.</li>
</ul>
<p>Pilih alat dan batas mana yang dipantau di <em>Track</em>, atau sembunyikan cincinnya di tab AI.</p>`,
  },
  updates: {
    title: "Pembaruan",
    html: `<p>QuickDesk memeriksa rilis baru 30 detik setelah mulai, lalu setiap 6 jam. Saat ada rilis baru, banner di jendela utama dan entri di bagian atas menu tray memberi tahu, dengan <strong>What’s New</strong>. Tidak ada yang dipasang sampai Anda mengklik <strong>Update Now</strong>.</p>
<p>Setiap pembaruan diverifikasi dengan kunci penandatanganan QuickDesk sebelum dipasang. Paket .deb atau .rpm meminta kata sandi admin; AppImage mengganti dirinya sendiri. Setelah itu QuickDesk memulai ulang dan menampilkan apa yang berubah. Matikan pemeriksaan otomatis, atau periksa secara manual, di <strong>Settings → Updates</strong>.</p>`,
  },
  uninstall: {
    title: "Mencopot",
    html: `<ol>
<li>Matikan <em>Start QuickDesk when I log in</em> di Settings lalu keluar dari tray.</li>
<li>Hapus paketnya: <code>sudo apt remove quick-desk</code> (Debian/Ubuntu) atau <code>sudo dnf remove QuickDesk</code> (Fedora), atau hapus AppImage-nya.</li>
<li>Data Anda tetap di <code>~/.local/share/click.quickdesk</code> sampai Anda menghapus folder itu.</li>
</ol>
<p><strong>Windows:</strong> Pengaturan → Aplikasi → Aplikasi yang diinstal → QuickDesk → Copot pemasangan. Data Anda tetap di <code>%APPDATA%\\click.quickdesk</code> sampai Anda menghapus folder itu.</p>
<p>Menghapus paket juga mencabut izin tempel instan, jika Anda mengaktifkannya.</p>`,
  },
  troubleshooting: {
    title: "Pemecahan masalah",
    html: `<ul>
<li><strong>Pintasan tidak berfungsi:</strong> periksa konflik di <em>Pengaturan → Papan Ketik → Pintasan Khusus</em>, atau pilih kombinasi lain di Settings → Hotkeys.</li>
<li><strong>Tempel otomatis tidak mengetik apa pun:</strong> pastikan aplikasi tempat Anda tadi masih punya fokus; ganti caranya di Settings → Auto-paste. Entri selalu disalin, jadi <kbd>Ctrl</kbd> <kbd>V</kbd> tetap berfungsi.</li>
<li><strong>Log:</strong> <code>~/.local/share/click.quickdesk/logs/</code>. Lampirkan saat Anda <a href="https://github.com/TankGum/quickdesk/issues">melaporkan masalah</a>.</li>
</ul>`,
  },
};

export default docs;
