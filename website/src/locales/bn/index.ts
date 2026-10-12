// বাংলা (Bengali). See ../en.ts for what each part is. The app itself is in
// English and Vietnamese, so its menu and button names stay in English, and
// so do the system's own labels (System Settings, More info …).
import type { Locale } from "..";
import changelog from "./CHANGELOG.md?raw";
import docs from "./docs";
import home from "./home";
import privacy from "./privacy";

const bn: Locale = {
  strings: {
    ui: {
      home: "হোম",
      download: "ডাউনলোড",
      docs: "ডকুমেন্টেশন",
      changelog: "পরিবর্তন",
      privacy: "গোপনীয়তা",
      github: "GitHub",
      language: "ভাষা",
      footer: "QuickDesk · MIT লাইসেন্স।",
      tagline: "নোট, ক্লিপবোর্ড ইতিহাস, পোর্ট আর AI সীমা। মাত্র একটি শর্টকাট দূরে।",
      systems: "সিস্টেম",
      soon: "শীঘ্রই",
      product: "প্রোডাক্ট",
      project: "প্রজেক্ট",
      issues: "সমস্যা জানান",
      sitemap: "সাইট ম্যাপ",
      skip: "মূল অংশে যান",
      menu: "মেনু",
      close: "বন্ধ করুন",
      faq: "প্রশ্নোত্তর",
      theme: "লাইট ও ডার্কের মধ্যে বদলান",
      themeShort: "লাইট / ডার্ক",
    },
    pages: {
      home: {
        title: "QuickDesk — Linux, Windows ও macOS-এর জন্য নোট, ক্লিপবোর্ড ও পোর্ট",
        description:
          "QuickDesk হলো Linux, Windows ও macOS-এর জন্য একটি বিনামূল্যের, ওপেন-সোর্স ডেস্কটপ সহকারী: এন্ড-টু-এন্ড এনক্রিপ্টেড নোট, ক্লিপবোর্ড ইতিহাস, লাইভ পোর্ট ম্যানেজার আর আপনার AI ব্যবহারের সীমা, সবই একটি শর্টকাটে।",
      },
      download: {
        title: "QuickDesk ডাউনলোড করুন — Linux, Windows ও macOS-এর জন্য (বিনামূল্যে)",
        description:
          "Windows 10 ও 11, Apple Silicon-এর macOS, Ubuntu, Debian, Fedora ও অন্যান্য Linux ডিস্ট্রিবিউশনের জন্য QuickDesk ডাউনলোড করুন: Windows ইনস্টলার, .dmg, .deb, .rpm ও AppImage, চেকসামসহ।",
      },
      docs: {
        title: "QuickDesk ডকুমেন্টেশন: ইনস্টল, শর্টকাট, সিঙ্ক, ক্লিপবোর্ড, পোর্ট",
        description:
          "Linux, Windows ও macOS-এ QuickDesk কীভাবে ইনস্টল করবেন, এর শর্টকাট, নোট সিঙ্ক, ক্লিপবোর্ড ইতিহাস, পোর্ট, ভাষার ভার্সন, AI ব্যবহার ও আপডেট কীভাবে ব্যবহার করবেন।",
      },
      changelog: {
        title: "QuickDesk পরিবর্তন: প্রতিটি রিলিজে নতুন কী",
        description: "QuickDesk-এর প্রতিটি রিলিজে নতুন কী এল, কী উন্নত হলো আর কী ঠিক করা হলো।",
      },
      privacy: {
        title: "গোপনীয়তা · QuickDesk",
        description: "QuickDesk-এ কোনো অ্যাকাউন্ট, অ্যানালিটিক্স বা টেলিমেট্রি নেই। কী আপনার কম্পিউটারে থাকে আর কী কোথায় পাঠানো হয়।",
      },
      sitemap: {
        title: "সাইট ম্যাপ · QuickDesk",
        description:
          "quickdesk.click-এর সব পেজ: Linux, Windows ও macOS-এর ডাউনলোড, অংশ ধরে ধরে ডকুমেন্টেশন, পরিবর্তনের তালিকার প্রতিটি রিলিজ আর গোপনীয়তা।",
      },
    },
    seo: {
      app: "একটি বিনামূল্যের, ওপেন-সোর্স ডেস্কটপ সহকারী: এন্ড-টু-এন্ড এনক্রিপ্টেড সিঙ্ক হওয়া নোট, ক্লিপবোর্ড ইতিহাস, পোর্ট ম্যানেজার, প্রোগ্রামিং ভাষার ভার্সন আর AI ব্যবহারের সীমা, প্রতিটিই একটি শর্টকাট দূরে।",
      ogAlt: "QuickDesk: আপনার পুরো ডেস্ক, মাত্র একটি শর্টকাট দূরে।",
    },
    hero: {
      pick: "আপনার সিস্টেম",
      linux: "Linux-এর জন্য ডাউনলোড",
      windows: "Windows-এর জন্য ডাউনলোড",
      macos: "macOS-এর জন্য ডাউনলোড{beta}",
      macSoon: "macOS: শীঘ্রই আসছে",
    },
    download: {
      title: "QuickDesk {v} ডাউনলোড করুন",
      leadMac: "Linux ও Windows (x86_64) এবং macOS (Apple Silicon{beta})-এর জন্য · {date} তারিখে প্রকাশিত। বিনামূল্যে ও ওপেন সোর্স (MIT)।",
      leadSoon: "x86_64-এ Linux ও Windows-এর জন্য, macOS শীঘ্রই · {date} তারিখে প্রকাশিত। বিনামূল্যে ও ওপেন সোর্স (MIT)।",
      pick: "আপনার সিস্টেম",
      devTitle: "Mac ভার্সন এখনও তৈরি হচ্ছে",
      devText:
        "macOS-এর জন্য QuickDesk কাজ করে, তবে এটি নতুন: এখনও প্রতিটি Mac-এ সবকিছু চেষ্টা করে দেখা হয়নি, আর অটো-পেস্ট ও শর্টকাট এখনও উন্নত করা হচ্ছে। কিছু কাজ না করলে আমাদের জানান।",
      devLink: "GitHub-এ সমস্যা জানান",
      deb: ["Ubuntu, Debian, Linux Mint, Pop!_OS", "Ubuntu 22.04 ও নতুন, Debian 12 ও নতুন।"],
      rpm: ["Fedora ও অন্যান্য RPM-ভিত্তিক ডিস্ট্রো", "Fedora 36 ও নতুন, openSUSE Tumbleweed।"],
      appimage: ["অন্য যেকোনো ডিস্ট্রিবিউশন", "পোর্টেবল: ইনস্টলের দরকার নেই। FUSE 2 (libfuse2) লাগে।"],
      windows: ["Windows 10 ও 11", "64-বিট। শুধু আপনার ইউজারের জন্য ইনস্টল হয়: অ্যাডমিন অধিকার লাগে না।"],
      macos: ["macOS 11 বা নতুন", "Apple Silicon (M1 ও নতুন)। ডিস্ক ইমেজ খুলে QuickDesk-কে Applications-এ টেনে আনুন।"],
      oneMac:
        "Terminal-এ চালান। এটি QuickDesk.app ডাউনলোড করে, এর SHA-256 মিলিয়ে দেখে, Applications-এ রাখে এবং খুলে দেয়। এভাবে ইনস্টল করলে macOS কোনো সতর্কবার্তা দেখায় না।",
      afterMac:
        "QuickDesk মেনু বারে থাকে এবং এখন থেকে নিজে নিজেই আপডেট হয়। অটো-পেস্টের জন্য, macOS জিজ্ঞেস করলে Privacy & Security → Accessibility-তে অনুমতি দিন।",
      gatekeeper: "ব্রাউজারে ডাউনলোড করলে macOS প্রথমবার চালু হতে বাধা দেয়: QuickDesk এখনও Apple-এর notarized নয়।",
      gatekeeperHow: "কীভাবে খুলবেন",
      openTitle: "প্রথমবার QuickDesk খোলা",
      openLead:
        "QuickDesk এখনও Apple-এর notarized নয় (এর জন্য পেইড Apple Developer ID লাগে)। .dmg ব্রাউজারে ডাউনলোড হলে, macOS প্রথমবার চালু করার সময় “Apple cannot check it for malicious software” বলে থামিয়ে দেয়। অ্যাপে কোনো সমস্যা নেই: নিচের যেকোনো একটি একবার করে নিন। উপরের কমান্ড দিয়ে ইনস্টল করে থাকলে এটি বাদ দিতে পারেন।",
      openA: "System Settings-এ",
      openASteps: [
        "QuickDesk-কে Applications-এ টেনে এনে খুলুন। macOS সতর্ক করলে Done-এ ক্লিক করুন।",
        "System Settings → Privacy & Security খুলে নিচে Security পর্যন্ত স্ক্রোল করুন।",
        "“QuickDesk was blocked to protect your Mac”-এর পাশে Open Anyway-তে ক্লিক করুন, পাসওয়ার্ড বা Touch ID দিয়ে নিশ্চিত করুন, তারপর Open-এ ক্লিক করুন।",
      ],
      openOld: "macOS 14 বা পুরোনোতে, Applications-এ QuickDesk-এ Control-ক্লিক করে Open বেছে নিন, তারপর আবার Open — এটাও কাজ করে।",
      openB: "অথবা Terminal-এ",
      openBLead: "QuickDesk-কে Applications-এ টেনে আনার পর এটি চালান, তারপর QuickDesk স্বাভাবিকভাবে খুলুন:",
      get: "ডাউনলোড",
      sha: "SHA-256",
      all: "সব চেকসাম (SHA256SUMS)",
      notes: "{v}-এ নতুন কী",
      allChanges: "সব পরিবর্তন",
      afterLinux: "তারপর অ্যাপ মেনু থেকে QuickDesk খুলুন। এটি টপ বারে থাকে এবং এখন থেকে নিজে নিজেই আপডেট হয়।",
      afterWindows: "তারপর Start মেনু থেকে QuickDesk খুলুন। এটি সিস্টেম ট্রেতে থাকে এবং এখন থেকে নিজে নিজেই আপডেট হয়।",
      copy: "কপি",
      oneTitle: "একটি কমান্ডে ইনস্টল করুন",
      oneLinux:
        "টার্মিনালে চালান। আপনার ডিস্ট্রিবিউশনের জন্য সঠিক প্যাকেজ (.deb, .rpm বা AppImage) বেছে নেয়, এর SHA-256 মিলিয়ে দেখে এবং ইনস্টল করে। পাসওয়ার্ড চায় শুধু ইনস্টলের ধাপে।",
      oneWindows: "PowerShell-এ চালান। ইনস্টলার ডাউনলোড করে, এর SHA-256 মিলিয়ে দেখে এবং অ্যাডমিন অধিকার ছাড়াই আপনার ইউজারের জন্য QuickDesk ইনস্টল করে।",
      oneScript: "স্ক্রিপ্ট পড়ুন",
      orPick: "অথবা একটি প্যাকেজ বেছে নিন",
      smartscreen: "Windows বলতে পারে যে সে আপনার PC সুরক্ষিত রেখেছে: ইনস্টলারটি এখনও সাইন করা নয়। More info → Run anyway বেছে নিন।",
      macTitle: "macOS-এর জন্য QuickDesk আসছে",
      macLead: "Mac-এ নোট, ক্লিপবোর্ড ইতিহাস, পোর্ট আর AI সীমা। এর ঘোষণা পরিবর্তনের পেজে আর GitHub রিলিজে দেওয়া হবে।",
      macWatch: "GitHub-এ রিলিজ দেখুন",
    },
    cta: {
      title: "আপনি যখন প্রস্তুত, এটিও প্রস্তুত।",
      lead: "একবার ইনস্টল করুন, তারপর সবকিছু মাত্র একটি শর্টকাট দূরে: আপনার নোট, যা কপি করেছেন, ব্যবহৃত পোর্ট আর আপনার AI সীমা।",
      download: "ডাউনলোড",
      changes: "নতুন কী",
      meta: "ভার্সন {v} · {date} · বিনামূল্যে ও ওপেন সোর্স (MIT)",
      downloading: "ডাউনলোড হচ্ছে",
      verified: "SHA-256 যাচাই হয়েছে",
      installed: "QuickDesk {v} ইনস্টল হয়েছে",
      running: "QuickDesk চলছে",
      open: "খুলতে চাপুন",
    },
    intro: {
      tagline: "আপনার পুরো ডেস্ক। মাত্র একটি শর্টকাট দূরে।",
      skip: "এড়িয়ে যান",
    },
    docs: {
      title: "ডকুমেন্টেশন",
      lead: "ইনস্টল, শর্টকাট, সিঙ্ক, ক্লিপবোর্ড, পোর্ট, AI ব্যবহার আর আপডেট।",
      contents: "সূচিপত্র",
      groups: ["শুরু করুন", "ফিচার", "সাহায্য"],
      sync: "সিঙ্ক",
      clipboard: "ক্লিপবোর্ড",
      versions: "রানটাইম",
      stuck: "এখনও আটকে আছেন?",
      ask: "GitHub-এ জিজ্ঞেস করুন",
    },
    sitemap: {
      title: "সাইট ম্যাপ",
      lead: "quickdesk.click-এর প্রতিটি পেজ ও অংশ।",
      home: "হোম",
      demo: "শর্টকাট চেষ্টা করুন",
      getIt: "QuickDesk নিন",
      oneLine: "ইনস্টল স্ক্রিপ্ট",
      other: "অন্যত্র",
      machine: "sitemap.xml (সার্চ ইঞ্জিনের জন্য)",
      privacyNote: "কোনো অ্যাকাউন্ট, অ্যানালিটিক্স বা টেলিমেট্রি নেই: কী আপনার কম্পিউটারে থাকে আর কী কোথায় পাঠানো হয়।",
    },
    changelog: {
      title: "পরিবর্তন",
      lead: "QuickDesk-এর প্রতিটি রিলিজে নতুন কী এল, কী উন্নত হলো আর কী ঠিক করা হলো।",
      latest: "সর্বশেষ",
      source: "উৎস: GitHub-এ পরিবর্তনের ফাইল",
      english: "এখনও অনুবাদ হয়নি; ইংরেজিতে দেখানো হচ্ছে।",
    },
    notFound: {
      title: "পেজ পাওয়া যায়নি · QuickDesk",
      description: "এই পেজটি নেই।",
      heading: "এই পেজটি নেই।",
    },
    demo: {
      refreshed: "ব্যবহার রিফ্রেশ হয়েছে",
      paused: "ক্লিপবোর্ড ইতিহাস থামানো হয়েছে",
      resumed: "ক্লিপবোর্ড ইতিহাস আবার চালু",
      quit: "বন্ধ — আসল অ্যাপে QuickDesk এখন বন্ধ হয়ে যেত",
      encrypting: "এনক্রিপ্ট হচ্ছে…",
      synced: "এনক্রিপ্ট ও সিঙ্ক হয়েছে",
      copied: "কপি হয়েছে",
      pressCopy: "Ctrl+C চাপুন",
      copy: "কপি",
      note: "স্ট্যান্ডআপ — মঙ্গলবার\n• ক্লিপবোর্ডের ছবি রিলিজ হয়েছে\n• রিলিজ প্রক্রিয়া দেখা হচ্ছে\n• কোনো বাধা নেই",
    },
  },
  content: { home, docs, privacy, changelog },
};

export default bn;
