// العربية (Arabic), shown right to left. See ../en.ts for what each part is.
// The app's menu and button names are written as the app itself shows them
// (src/shared/locales/ar.ts). The system's own labels (System Settings, More
// info …) stay in English.
import type { Locale } from "..";
import changelog from "./CHANGELOG.md?raw";
import docs from "./docs";
import home from "./home";
import privacy from "./privacy";

const ar: Locale = {
  strings: {
    ui: {
      home: "الرئيسية",
      download: "تنزيل",
      docs: "التوثيق",
      changelog: "سجل التغييرات",
      privacy: "الخصوصية",
      github: "GitHub",
      language: "اللغة",
      footer: "QuickDesk · رخصة MIT.",
      tagline: "الملاحظات وسجل الحافظة والمنافذ وحدود الذكاء الاصطناعي. على بُعد اختصار واحد.",
      systems: "الأنظمة",
      soon: "قريبًا",
      product: "المنتج",
      project: "المشروع",
      issues: "الإبلاغ عن مشكلة",
      sitemap: "خريطة الموقع",
      skip: "انتقل إلى المحتوى",
      menu: "القائمة",
      close: "إغلاق",
      faq: "الأسئلة الشائعة",
      theme: "التبديل بين الفاتح والداكن",
      themeShort: "فاتح / داكن",
    },
    pages: {
      home: {
        title: "QuickDesk — الملاحظات والحافظة والمنافذ على Linux وWindows وmacOS",
        description:
          "QuickDesk مساعد سطح مكتب مجاني ومفتوح المصدر لأنظمة Linux وWindows وmacOS: ملاحظات مشفّرة من طرف إلى طرف، وسجل للحافظة، ومدير منافذ مباشر، وحدود استخدامك للذكاء الاصطناعي، كلها باختصار واحد.",
      },
      download: {
        title: "تنزيل QuickDesk لأنظمة Linux وWindows وmacOS (مجانًا)",
        description:
          "نزّل QuickDesk لنظامي Windows 10 و11، وmacOS على Apple Silicon، وUbuntu وDebian وFedora وتوزيعات Linux الأخرى: مثبّت Windows وملفات .dmg و.deb و.rpm وAppImage، مع المجاميع الاختبارية.",
      },
      docs: {
        title: "توثيق QuickDesk: التثبيت والاختصارات والمزامنة والحافظة والمنافذ",
        description:
          "كيف تثبّت QuickDesk على Linux وWindows وmacOS، وتستخدم اختصاراته ومزامنة الملاحظات وسجل الحافظة والمنافذ وإصدارات لغات البرمجة واستخدام الذكاء الاصطناعي والتحديثات.",
      },
      changelog: {
        title: "سجل تغييرات QuickDesk: الجديد في كل إصدار",
        description: "الجديد والمحسَّن والمُصلَح في كل إصدار من QuickDesk.",
      },
      privacy: {
        title: "الخصوصية · QuickDesk",
        description: "لا حسابات ولا تحليلات ولا قياس عن بُعد في QuickDesk. ما يبقى على حاسوبك وما يُرسَل وإلى أين.",
      },
      sitemap: {
        title: "خريطة الموقع · QuickDesk",
        description:
          "كل صفحات quickdesk.click: التنزيل لأنظمة Linux وWindows وmacOS، والتوثيق قسمًا قسمًا، وكل إصدار في سجل التغييرات، والخصوصية.",
      },
    },
    seo: {
      app: "مساعد سطح مكتب مجاني ومفتوح المصدر: ملاحظات متزامنة مشفّرة من طرف إلى طرف، وسجل للحافظة، ومدير للمنافذ، وإصدارات لغات البرمجة، وحدود استخدام الذكاء الاصطناعي، كلٌّ منها على بُعد اختصار واحد.",
      ogAlt: "QuickDesk: مكتبك كله، على بُعد اختصار واحد.",
    },
    hero: {
      pick: "نظامك",
      linux: "تنزيل لنظام Linux",
      windows: "تنزيل لنظام Windows",
      macos: "تنزيل لنظام macOS{beta}",
      macSoon: "macOS: قريبًا",
    },
    download: {
      title: "تنزيل QuickDesk {v}",
      leadMac: "لنظامي Linux وWindows ‏(x86_64) ولنظام macOS ‏(Apple Silicon{beta}) · صدر في {date}. مجاني ومفتوح المصدر (MIT).",
      leadSoon: "لنظامي Linux وWindows على x86_64، وmacOS قريبًا · صدر في {date}. مجاني ومفتوح المصدر (MIT).",
      pick: "نظامك",
      devTitle: "إصدار Mac ما زال قيد التطوير",
      devText:
        "يعمل QuickDesk على macOS، لكنه جديد: لم تُجرَّب كل الميزات على كل أجهزة Mac بعد، وما زال اللصق التلقائي والاختصارات قيد التحسين. إن لم يعمل شيء ما، فأخبرنا من فضلك.",
      devLink: "أبلغ عن مشكلة على GitHub",
      deb: ["Ubuntu وDebian وLinux Mint وPop!_OS", "Ubuntu 22.04 وأحدث، Debian 12 وأحدث."],
      rpm: ["Fedora والتوزيعات الأخرى المبنية على RPM", "Fedora 36 وأحدث، openSUSE Tumbleweed."],
      appimage: ["أي توزيعة أخرى", "نسخة محمولة: لا تحتاج إلى تثبيت. تتطلب FUSE 2 ‏(libfuse2)."],
      windows: ["Windows 10 و11", "64 بت. يُثبَّت لمستخدمك فقط: لا حاجة إلى صلاحيات المسؤول."],
      macos: ["macOS 11 أو أحدث", "Apple Silicon ‏(M1 وأحدث). افتح صورة القرص واسحب QuickDesk إلى Applications."],
      oneMac:
        "في Terminal. ينزّل QuickDesk.app ويتحقق من SHA-256 الخاص به ويضعه في Applications ويفتحه. عند التثبيت بهذه الطريقة لا يُظهر macOS أي تحذير.",
      afterMac:
        "يعيش QuickDesk في شريط القوائم ويحدّث نفسه من الآن فصاعدًا. للصق التلقائي، اسمح له في Privacy & Security → Accessibility عندما يسألك macOS.",
      gatekeeper: "عند التنزيل من المتصفح، يمنع macOS التشغيل الأول: لم يُوثَّق QuickDesk لدى Apple بعد.",
      gatekeeperHow: "كيف تفتحه",
      openTitle: "فتح QuickDesk لأول مرة",
      openLead:
        "لم يُوثَّق QuickDesk لدى Apple بعد (يتطلب ذلك Apple Developer ID مدفوعًا). إذا نُزِّل ملف .dmg من المتصفح، يوقف macOS التشغيل الأول برسالة “Apple cannot check it for malicious software”. لا مشكلة في التطبيق: نفّذ إحدى الطريقتين أدناه مرة واحدة. إن ثبّته بالأمر أعلاه، فيمكنك تخطي هذا.",
      openA: "في System Settings",
      openASteps: [
        "اسحب QuickDesk إلى Applications وافتحه. عندما يحذّرك macOS، انقر Done.",
        "افتح System Settings → Privacy & Security ومرّر إلى الأسفل حتى Security.",
        "بجوار “QuickDesk was blocked to protect your Mac”، انقر Open Anyway، وأكّد بكلمة المرور أو Touch ID، ثم انقر Open.",
      ],
      openOld: "على macOS 14 أو أقدم، يمكنك أيضًا النقر مع Control على QuickDesk في Applications واختيار Open، ثم Open مرة أخرى.",
      openB: "أو في Terminal",
      openBLead: "بعد سحب QuickDesk إلى Applications، شغّل هذا الأمر، ثم افتح QuickDesk كالمعتاد:",
      get: "تنزيل",
      sha: "SHA-256",
      all: "كل المجاميع الاختبارية (SHA256SUMS)",
      notes: "الجديد في {v}",
      allChanges: "كل التغييرات",
      afterLinux: "ثم افتح QuickDesk من قائمة التطبيقات. يعيش في الشريط العلوي ويحدّث نفسه من الآن فصاعدًا.",
      afterWindows: "ثم افتح QuickDesk من قائمة Start. يعيش في علبة النظام ويحدّث نفسه من الآن فصاعدًا.",
      copy: "نسخ",
      oneTitle: "التثبيت بأمر واحد",
      oneLinux:
        "في الطرفية. يختار الحزمة المناسبة لتوزيعتك (.deb أو .rpm أو AppImage)، ويتحقق من SHA-256 الخاص بها ويثبّتها. لا يطلب كلمة مرورك إلا في خطوة التثبيت.",
      oneWindows: "في PowerShell. ينزّل المثبّت ويتحقق من SHA-256 الخاص به ويثبّت QuickDesk لمستخدمك، دون صلاحيات المسؤول.",
      oneScript: "اقرأ السكربت",
      orPick: "أو اختر حزمة",
      smartscreen: "قد يقول Windows إنه حمى جهازك: المثبّت غير موقَّع بعد. اختر More info → Run anyway.",
      macTitle: "QuickDesk لنظام macOS في الطريق",
      macLead: "الملاحظات وسجل الحافظة والمنافذ وحدود الذكاء الاصطناعي على Mac. سيُعلَن عنه في سجل التغييرات وفي إصدارات GitHub.",
      macWatch: "تابع الإصدارات على GitHub",
    },
    cta: {
      title: "جاهز متى كنت جاهزًا.",
      lead: "ثبّته مرة واحدة وسيصبح كل شيء على بُعد اختصار: ملاحظاتك، وما نسخته، والمنافذ المستخدمة، وحدود الذكاء الاصطناعي.",
      download: "تنزيل",
      changes: "الجديد",
      meta: "الإصدار {v} · {date} · مجاني ومفتوح المصدر (MIT)",
      downloading: "جارٍ تنزيل",
      verified: "تم التحقق من SHA-256",
      installed: "تم تثبيت QuickDesk {v}",
      running: "QuickDesk يعمل الآن",
      open: "افتحه بـ",
    },
    intro: {
      tagline: "مكتبك كله. على بُعد اختصار واحد.",
      skip: "تخطَّ",
    },
    docs: {
      title: "التوثيق",
      lead: "التثبيت والاختصارات والمزامنة والحافظة والمنافذ واستخدام الذكاء الاصطناعي والتحديثات.",
      contents: "المحتويات",
      groups: ["البداية", "الميزات", "المساعدة"],
      sync: "المزامنة",
      clipboard: "الحافظة",
      versions: "بيئات التشغيل",
      stuck: "ما زلت عالقًا؟",
      ask: "اسأل على GitHub",
    },
    sitemap: {
      title: "خريطة الموقع",
      lead: "كل صفحة وقسم في quickdesk.click.",
      home: "الرئيسية",
      demo: "جرّب الاختصارات",
      getIt: "احصل على QuickDesk",
      oneLine: "سكربتات التثبيت",
      other: "في أماكن أخرى",
      machine: "sitemap.xml (لمحركات البحث)",
      privacyNote: "لا حسابات ولا تحليلات ولا قياس عن بُعد: ما يبقى على حاسوبك وما يُرسَل وإلى أين.",
    },
    changelog: {
      title: "سجل التغييرات",
      lead: "الجديد والمحسَّن والمُصلَح في كل إصدار من QuickDesk.",
      latest: "الأحدث",
      source: "المصدر: سجل التغييرات على GitHub",
      english: "لم يُترجَم بعد؛ يُعرض بالإنجليزية.",
    },
    notFound: {
      title: "الصفحة غير موجودة · QuickDesk",
      description: "هذه الصفحة غير موجودة.",
      heading: "هذه الصفحة غير موجودة.",
    },
    demo: {
      refreshed: "تم تحديث الاستخدام",
      paused: "تم إيقاف سجل الحافظة مؤقتًا",
      resumed: "تم استئناف سجل الحافظة",
      quit: "خروج — في التطبيق الحقيقي سيُغلَق QuickDesk الآن",
      encrypting: "جارٍ التشفير…",
      synced: "تم التشفير والمزامنة",
      copied: "تم النسخ",
      pressCopy: "اضغط Ctrl+C",
      copy: "نسخ",
      note: "الاجتماع اليومي — الثلاثاء\n• أُطلقت صور الحافظة\n• مراجعة خطوات الإصدار\n• لا عوائق",
    },
  },
  content: { home, docs, privacy, changelog },
};

export default ar;
