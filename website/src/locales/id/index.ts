// Bahasa Indonesia (Indonesian). See ../en.ts for what each part is. The app's
// menu and button names are written as the app itself shows them
// (src/shared/locales/id.ts). The system's own labels are named as the system
// shows them.
import type { Locale } from "..";
import changelog from "./CHANGELOG.md?raw";
import docs from "./docs";
import home from "./home";
import privacy from "./privacy";

const id: Locale = {
  strings: {
    ui: {
      home: "Beranda",
      download: "Unduh",
      docs: "Dokumentasi",
      changelog: "Yang baru",
      privacy: "Privasi",
      github: "GitHub",
      language: "Bahasa",
      footer: "QuickDesk · Lisensi MIT.",
      tagline: "Catatan, riwayat clipboard, port, dan batas AI. Cukup satu pintasan.",
      systems: "Sistem",
      soon: "segera",
      product: "Produk",
      project: "Proyek",
      issues: "Laporkan masalah",
      sitemap: "Peta situs",
      skip: "Langsung ke konten",
      menu: "Menu",
      close: "Tutup",
      faq: "Pertanyaan",
      theme: "Ganti tema terang dan gelap",
      themeShort: "Terang / gelap",
    },
    pages: {
      home: {
        title: "QuickDesk — Catatan, clipboard, dan port untuk Linux, Windows, dan macOS",
        description:
          "QuickDesk adalah asisten desktop gratis dan sumber terbuka untuk Linux, Windows, dan macOS: catatan terenkripsi end-to-end, riwayat clipboard, pengelola port langsung, dan batas penggunaan AI Anda, cukup dengan satu pintasan.",
      },
      download: {
        title: "Unduh QuickDesk untuk Linux, Windows, dan macOS (gratis)",
        description:
          "Unduh QuickDesk untuk Windows 10 dan 11, macOS di Apple Silicon, Ubuntu, Debian, Fedora, dan distribusi Linux lainnya: penginstal Windows, .dmg, .deb, .rpm, dan AppImage, lengkap dengan checksum.",
      },
      docs: {
        title: "Dokumentasi QuickDesk: instalasi, pintasan, sinkronisasi, clipboard, port",
        description:
          "Cara memasang QuickDesk di Linux, Windows, dan macOS serta memakai pintasan, sinkronisasi catatan, riwayat clipboard, port, versi bahasa pemrograman, penggunaan AI, dan pembaruan.",
      },
      changelog: {
        title: "Yang baru di QuickDesk: perubahan di setiap versi",
        description: "Apa yang baru, apa yang lebih baik, dan apa yang diperbaiki di setiap versi QuickDesk.",
      },
      privacy: {
        title: "Privasi · QuickDesk",
        description: "QuickDesk tidak punya akun, analitik, maupun telemetri. Apa yang tetap di komputer Anda, dan apa yang dikirim ke mana.",
      },
      sitemap: {
        title: "Peta situs · QuickDesk",
        description:
          "Semua halaman quickdesk.click: unduhan untuk Linux, Windows, dan macOS, dokumentasi per bagian, setiap rilis di daftar perubahan, dan privasi.",
      },
    },
    seo: {
      app: "Asisten desktop gratis dan sumber terbuka: catatan tersinkron dan terenkripsi end-to-end, riwayat clipboard, pengelola port, versi bahasa pemrograman, dan batas penggunaan AI, masing-masing cukup dengan satu pintasan.",
      ogAlt: "QuickDesk: seluruh desktop Anda, cukup satu pintasan.",
    },
    hero: {
      pick: "Sistem Anda",
      linux: "Unduh untuk Linux",
      windows: "Unduh untuk Windows",
      macos: "Unduh untuk macOS{beta}",
      macSoon: "macOS: segera hadir",
    },
    download: {
      title: "Unduh QuickDesk {v}",
      leadMac: "Untuk Linux dan Windows (x86_64) serta macOS (Apple Silicon{beta}) · dirilis {date}. Gratis dan sumber terbuka (MIT).",
      leadSoon: "Untuk Linux dan Windows di x86_64, macOS segera hadir · dirilis {date}. Gratis dan sumber terbuka (MIT).",
      pick: "Sistem Anda",
      devTitle: "Versi Mac masih dalam pengembangan",
      devText:
        "QuickDesk untuk macOS sudah berjalan, tetapi masih baru: belum semuanya diuji di setiap Mac, dan tempel otomatis serta pintasan masih terus disempurnakan. Jika ada yang tidak berfungsi, beri tahu kami.",
      devLink: "Laporkan masalah di GitHub",
      deb: ["Ubuntu, Debian, Linux Mint, Pop!_OS", "Ubuntu 22.04 atau lebih baru, Debian 12 atau lebih baru."],
      rpm: ["Fedora dan distribusi berbasis RPM lainnya", "Fedora 36 atau lebih baru, openSUSE Tumbleweed."],
      appimage: ["Distribusi lain mana pun", "Portabel: tidak perlu dipasang. Membutuhkan FUSE 2 (libfuse2)."],
      windows: ["Windows 10 dan 11", "64-bit. Dipasang hanya untuk pengguna Anda: tanpa hak administrator."],
      macos: ["macOS 11 atau lebih baru", "Apple Silicon (M1 atau lebih baru). Buka image disk lalu seret QuickDesk ke Aplikasi."],
      oneMac:
        "Di Terminal. Mengunduh QuickDesk.app, memeriksa SHA-256-nya, menaruhnya di Aplikasi, lalu membukanya. Jika dipasang dengan cara ini, macOS tidak menampilkan peringatan apa pun.",
      afterMac:
        "QuickDesk berada di bar menu dan mulai sekarang memperbarui dirinya sendiri. Untuk tempel otomatis, izinkan di Privasi & Keamanan → Aksesibilitas saat macOS memintanya.",
      gatekeeper: "Jika diunduh lewat browser, macOS memblokir saat pertama dibuka: QuickDesk belum dinotarisasi oleh Apple.",
      gatekeeperHow: "Cara membukanya",
      openTitle: "Membuka QuickDesk untuk pertama kali",
      openLead:
        "QuickDesk belum dinotarisasi oleh Apple (itu memerlukan Apple Developer ID berbayar). Jika .dmg diunduh lewat browser, macOS menghentikan pembukaan pertama dengan peringatan bahwa “Apple tidak dapat memverifikasi bahwa aplikasi ini bebas dari malware”. Tidak ada yang salah dengan aplikasinya: lakukan salah satu dari dua cara ini, cukup sekali. Jika Anda memasangnya dengan perintah di atas, Anda bisa melewati bagian ini.",
      openA: "Di Pengaturan Sistem",
      openASteps: [
        "Seret QuickDesk ke Aplikasi lalu buka. Saat macOS menampilkan peringatan, tutup peringatannya.",
        "Buka Pengaturan Sistem → Privasi & Keamanan dan gulir ke bagian Keamanan.",
        "Di samping ““QuickDesk” diblokir untuk melindungi Mac Anda”, klik Tetap Buka, konfirmasi dengan kata sandi atau Touch ID, lalu klik Buka.",
      ],
      openOld: "Di macOS 14 atau versi lebih lama, Anda juga bisa Control-klik QuickDesk di Aplikasi, pilih Buka, lalu klik Buka sekali lagi.",
      openB: "Atau di Terminal",
      openBLead: "Setelah menyeret QuickDesk ke Aplikasi, jalankan perintah ini lalu buka QuickDesk seperti biasa:",
      get: "Unduh",
      sha: "SHA-256",
      all: "Semua checksum (SHA256SUMS)",
      notes: "Yang baru di versi {v}",
      allChanges: "Semua perubahan",
      afterLinux: "Lalu buka QuickDesk dari menu aplikasi. QuickDesk berada di bar atas dan mulai sekarang memperbarui dirinya sendiri.",
      afterWindows: "Lalu buka QuickDesk dari menu Start. QuickDesk berada di system tray dan mulai sekarang memperbarui dirinya sendiri.",
      copy: "Salin",
      oneTitle: "Pasang dengan satu perintah",
      oneLinux:
        "Di terminal. Memilih paket yang tepat untuk distribusi Anda (.deb, .rpm, atau AppImage), memeriksa SHA-256-nya, lalu memasangnya. Kata sandi hanya diminta pada langkah pemasangan.",
      oneWindows: "Di PowerShell. Mengunduh penginstal, memeriksa SHA-256-nya, lalu memasang QuickDesk untuk pengguna Anda, tanpa hak administrator.",
      oneScript: "Baca skripnya",
      orPick: "Atau pilih paket",
      smartscreen: "Windows mungkin mengatakan telah melindungi PC Anda: penginstal belum ditandatangani. Pilih Info selengkapnya → Tetap jalankan.",
      macTitle: "QuickDesk untuk macOS segera hadir",
      macLead: "Catatan, riwayat clipboard, port, dan batas AI di Mac. Akan diumumkan di daftar perubahan dan rilis GitHub.",
      macWatch: "Ikuti rilis di GitHub",
    },
    cta: {
      title: "Siap kapan pun Anda siap.",
      lead: "Pasang sekali dan semuanya cukup satu pintasan: catatan Anda, apa yang Anda salin, port yang sedang dipakai, dan batas AI Anda.",
      download: "Unduh",
      changes: "Yang baru",
      meta: "Versi {v} · {date} · Gratis dan sumber terbuka (MIT)",
      downloading: "Mengunduh",
      verified: "SHA-256 terverifikasi",
      installed: "QuickDesk {v} terpasang",
      running: "QuickDesk sedang berjalan",
      open: "Buka dengan",
    },
    intro: {
      tagline: "Seluruh desktop Anda. Cukup satu pintasan.",
      skip: "Lewati",
    },
    docs: {
      title: "Dokumentasi",
      lead: "Instalasi, pintasan, sinkronisasi, clipboard, port, penggunaan AI, dan pembaruan.",
      contents: "Isi",
      groups: ["Memulai", "Fitur", "Bantuan"],
      sync: "Sinkronisasi",
      clipboard: "Clipboard",
      versions: "Runtime",
      stuck: "Masih kesulitan?",
      ask: "Tanya di GitHub",
    },
    sitemap: {
      title: "Peta situs",
      lead: "Semua halaman dan bagian di quickdesk.click.",
      home: "Beranda",
      demo: "Coba pintasannya",
      getIt: "Dapatkan QuickDesk",
      oneLine: "Skrip instalasi",
      other: "Di tempat lain",
      machine: "sitemap.xml (untuk mesin pencari)",
      privacyNote: "Tanpa akun, analitik, maupun telemetri: apa yang tetap di komputer Anda, dan apa yang dikirim ke mana.",
    },
    changelog: {
      title: "Yang baru",
      lead: "Apa yang baru, apa yang lebih baik, dan apa yang diperbaiki di setiap versi QuickDesk.",
      latest: "Terbaru",
      source: "Sumber: changelog di GitHub",
      english: "Belum diterjemahkan; ditampilkan dalam bahasa Inggris.",
    },
    notFound: {
      title: "Halaman tidak ditemukan · QuickDesk",
      description: "Halaman ini tidak ada.",
      heading: "Halaman ini tidak ada.",
    },
    demo: {
      refreshed: "Penggunaan diperbarui",
      paused: "Riwayat clipboard dijeda",
      resumed: "Riwayat clipboard dilanjutkan",
      quit: "Keluar — di aplikasi sungguhan, QuickDesk akan ditutup sekarang",
      encrypting: "Mengenkripsi…",
      synced: "Terenkripsi dan tersinkron",
      copied: "Disalin",
      pressCopy: "Tekan Ctrl+C",
      copy: "Salin",
      note: "Standup — Selasa\n• Gambar clipboard sudah rilis\n• Meninjau alur rilis\n• Tidak ada hambatan",
    },
  },
  content: { home, docs, privacy, changelog },
};

export default id;
