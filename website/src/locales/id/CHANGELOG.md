# Yang baru

Apa yang berubah di setiap versi QuickDesk (terjemahan bahasa Indonesia). Versi asli berbahasa Inggris ada di CHANGELOG.md di akar repositori; versi yang belum ada di sini ditampilkan dalam bahasa Inggris.

## 0.6.1 — 2026-10-12

### Diperbaiki
- macOS: memilih entri di jendela clipboard kembali menempelkannya ke kolom teks tempat Anda mengetik. Kini QuickDesk menyingkir agar macOS mengembalikan fokus ke aplikasi dan kolom itu, alih-alih meminta aplikasi itu diaktifkan, sesuatu yang di versi macOS terbaru hanya terpenuhi setengah jalan.

## 0.6.0 — 2026-10-11

### Baru
- QuickDesk untuk macOS (Apple Silicon, macOS 11 atau lebih baru): catatan dengan sinkronisasi, riwayat clipboard dengan teks, gambar, dan berkas, tempel otomatis dengan ⌘V ke aplikasi yang sedang Anda pakai, port, versi bahasa pemrograman, dan penggunaan AI, dari bar menu. Pintasannya ⌃⌥N / V / P. Pasang dari Terminal dengan `curl -fsSL https://quickdesk.click/install.sh | bash` atau sebagai .dmg dari halaman unduhan; aplikasinya memperbarui diri sendiri seperti di sistem lain. Aplikasi ini belum dinotarisasi oleh Apple: jika .dmg diunduh lewat browser, macOS memblokir pembukaan pertama, dan halaman unduhan menjelaskan cara membukanya (Pengaturan Sistem → Privasi & Keamanan → Tetap Buka).
- Di Mac, tab AI juga bisa menampilkan batas Claude Anda: di sana Claude Code menyimpan login-nya di Rantai Kunci, dan QuickDesk hanya membacanya setelah Anda mengizinkannya.

### Diperbaiki
- Skrip instalasi menyebutkan paket mana yang belum ada di sebuah rilis, alih-alih berhenti tanpa pesan.

## 0.5.0 — 2026-10-11

### Baru
- QuickDesk untuk Windows 10 dan 11 (64-bit): catatan dengan sinkronisasi, riwayat clipboard dengan teks, gambar, dan berkas, tempel otomatis ke aplikasi yang sedang Anda pakai, port, dan penggunaan AI, dengan pintasan yang sama Win+Alt+N / V / P. Pasang dari PowerShell dengan `irm https://quickdesk.click/install.ps1 | iex` atau dari halaman unduhan; aplikasinya dipasang untuk pengguna Anda tanpa hak administrator dan memperbarui diri sendiri. Tab Runtimes bekerja dengan mise, rustup, dan uv: pasang mise dengan sekali klik, tambahkan Node, Python, Go, dan lainnya, dan “Fix PATH…” memasukkan foldernya ke PATH akun Anda tanpa hak administrator.
- Runtimes: bahasa yang belum ada di komputer muncul di “Add a language”, siap dipasang dengan mise.

### Lebih baik
- Runtimes: kanal Rust juga punya tombol pembaruan. Saat rustup punya stable (atau beta, nightly) yang lebih baru, kanal itu menampilkan ↑ dan sekali klik menjalankan `rustup update` untuknya.

## 0.4.0 — 2026-10-10

### Baru
- Tab Runtimes untuk versi bahasa pemrograman Anda: lihat Node.js, Python, Rust, Go… mana yang benar-benar dijalankan terminal baru dan dari mana asalnya, pasang dan copot versi, pilih versi default, dan kunci satu versi per proyek. QuickDesk bekerja dengan nvm, uv, rustup, dan mise, dan tidak pernah menyentuh paket sistem.
- Saat ada rilis lebih baru dari seri yang terpasang (misalnya Node 22.24.1 untuk 22.23.3), versi itu menampilkan tombol pembaruan; versi lama tetap ada sampai Anda menghapusnya.
- Bahasa tanpa pengelola versi (misalnya Go yang dipasang dengan apt) bisa mendapatkan mise dengan sekali klik, diunduh dari rilisnya di GitHub dan diperiksa dengan checksum SHA-256-nya.
- “Apply right away in open terminals”: perubahan sampai ke terminal yang sudah terbuka pada prompt berikutnya, dan Node mengikuti `.nvmrc` terdekat saat Anda `cd`. Saat ada sesuatu di PATH yang menghalangi, tab ini menjelaskan alasannya dan menawarkan baris persis yang perlu ditambahkan ke berkas shell Anda, beserta cadangannya.

### Lebih baik
- Halaman unduhan menampilkan perintah lengkap untuk mengunduh dan memasang dari terminal.
- Daftar versi bisa menampilkan versi terbaru dari setiap seri, hanya LTS, atau semuanya.

## 0.3.1 — 2026-10-09

### Baru
- Sinkronisasi kini sudah bawaan: aktifkan di Settings → Sync dan hubungkan komputer lain dengan kode sinkronisasi. Tanpa akun, email, atau pengaturan penyimpanan. Catatan tetap dienkripsi di komputer Anda sebelum keluar darinya; QuickDesk Cloud hanya menyimpan teks terenkripsi.

### Diperbaiki
- Catatan, riwayat clipboard, dan pengaturan dari versi sebelumnya kembali. Pembaruan ke 0.2.0 dan 0.3.0 dimulai dengan folder kosong alih-alih memindahkan data Anda; kini QuickDesk membawanya sendiri dan menyimpan folder lama sebagai cadangan.
- Membuka Settings tidak lagi membuat titik berwarna di tab lain menjadi abu-abu.

### Lebih baik
- Sinkronisasi lewat bucket S3 atau R2 milik sendiri tidak lagi ditawarkan. Jika Anda memakainya, sinkronisasi dimatikan sekali, dengan catatan di Settings → Sync; catatan Anda tetap di komputer Anda.
- Menu penggunaan AI di bar atas hanya menampilkan alat yang Anda pantau, dengan bar hijau, kuning, atau merah untuk setiap batas dan waktu pengaturan ulangnya di bawah. Pilih alat lain di Track.

## 0.3.0 — 2026-10-09

### Baru
- QuickDesk punya alamatnya sendiri: [quickdesk.click](https://quickdesk.click). Unduhan dan pembaruan datang dari sana.
- Situs web sungguhan: halaman unduhan yang menyarankan paket yang tepat, dokumentasi, daftar perubahan, dan halaman privasi, dalam bahasa Inggris dan Vietnam.

### Lebih baik
- Data Anda kini tinggal di `~/.local/share/click.quickdesk`. QuickDesk memindahkannya sendiri saat pertama kali dijalankan; catatan, riwayat clipboard, dan pengaturan tetap seperti semula.
- Memulai ulang setelah pembaruan kini lebih andal.

## 0.2.2 — 2026-10-09

### Baru
- QuickDesk kini memperbarui dirinya sendiri. QuickDesk memeriksa rilis baru di latar belakang, memberi tahu apa yang berubah, dan hanya memasangnya saat Anda memilih. Setiap pembaruan diverifikasi dengan kunci penandatanganan QuickDesk sebelum dipasang.
- Settings → Updates: lihat versi Anda, periksa sekarang, atau matikan pemeriksaan otomatis.
- Setelah pembaruan, QuickDesk menampilkan yang baru satu kali.
- Halaman unduhan tersedia dalam bahasa Vietnam dan menautkan ke kode sumber di GitHub.

### Lebih baik
- Satu ikon di mana-mana: dock, menu aplikasi, jendela, dan tray kini memakai ikon QuickDesk yang sama.

## 0.2.1 — 2026-10-08

### Baru
- Tampilan baru, dipakai bersama oleh aplikasi dan halaman unduhannya: satu warna per bagian (catatan, clipboard, port, AI), petunjuk keyboard di jendela pop-up, dan bar tab bergaya macOS.
- Ikon garis menggantikan emoji di seluruh aplikasi.
- Menu tray dengan gaya macOS yang lebih rapi: label pendek, tanda centang untuk “Save Clipboard History”, dan batas AI dikelompokkan per alat.

### Lebih baik
- Demo di halaman unduhan adalah aplikasi sungguhan yang berjalan dengan data contoh.

## 0.2.0 — 2026-10-08

### Baru
- Paket untuk Ubuntu/Debian (.deb), Fedora (.rpm), dan distribusi lain (AppImage), diterbitkan di situs unduhan QuickDesk.

### Lebih baik
- Data Anda dipindahkan otomatis ke folder data QuickDesk yang baru (`~/.local/share/io.github.tankgum.quickdesk`).
