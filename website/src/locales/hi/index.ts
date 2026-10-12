// हिन्दी (Hindi). See ../en.ts for what each part is. The app itself is in
// English and Vietnamese, so its menu and button names stay in English, and
// so do the system's own labels (System Settings, More info …).
import type { Locale } from "..";
import changelog from "./CHANGELOG.md?raw";
import docs from "./docs";
import home from "./home";
import privacy from "./privacy";

const hi: Locale = {
  strings: {
    ui: {
      home: "होम",
      download: "डाउनलोड",
      docs: "दस्तावेज़",
      changelog: "बदलाव",
      privacy: "गोपनीयता",
      github: "GitHub",
      language: "भाषा",
      footer: "QuickDesk · MIT लाइसेंस।",
      tagline: "नोट्स, क्लिपबोर्ड इतिहास, पोर्ट और AI सीमाएँ। बस एक शॉर्टकट दूर।",
      systems: "सिस्टम",
      soon: "जल्द",
      product: "प्रोडक्ट",
      project: "प्रोजेक्ट",
      issues: "समस्या बताएँ",
      sitemap: "साइट मैप",
      skip: "सामग्री पर जाएँ",
      menu: "मेन्यू",
      close: "बंद करें",
      faq: "सवाल-जवाब",
      theme: "लाइट और डार्क के बीच बदलें",
      themeShort: "लाइट / डार्क",
    },
    pages: {
      home: {
        title: "QuickDesk — Linux, Windows और macOS के लिए नोट्स, क्लिपबोर्ड और पोर्ट",
        description:
          "QuickDesk, Linux, Windows और macOS के लिए एक मुफ़्त, ओपन-सोर्स डेस्कटॉप सहायक है: एंड-टू-एंड एन्क्रिप्टेड नोट्स, क्लिपबोर्ड इतिहास, लाइव पोर्ट मैनेजर और आपकी AI उपयोग सीमाएँ, सब एक शॉर्टकट से।",
      },
      download: {
        title: "QuickDesk डाउनलोड करें — Linux, Windows और macOS के लिए (मुफ़्त)",
        description:
          "Windows 10 और 11, Apple Silicon वाले macOS, Ubuntu, Debian, Fedora और दूसरे Linux डिस्ट्रीब्यूशन के लिए QuickDesk डाउनलोड करें: Windows इंस्टॉलर, .dmg, .deb, .rpm और AppImage, चेकसम के साथ।",
      },
      docs: {
        title: "QuickDesk दस्तावेज़: इंस्टॉल, शॉर्टकट, सिंक, क्लिपबोर्ड, पोर्ट",
        description:
          "Linux, Windows और macOS पर QuickDesk कैसे इंस्टॉल करें, उसके शॉर्टकट, नोट्स सिंक, क्लिपबोर्ड इतिहास, पोर्ट, भाषा वर्ज़न, AI उपयोग और अपडेट कैसे इस्तेमाल करें।",
      },
      changelog: {
        title: "QuickDesk बदलाव: हर रिलीज़ में क्या नया है",
        description: "QuickDesk की हर रिलीज़ में क्या नया है, क्या बेहतर हुआ और क्या ठीक हुआ।",
      },
      privacy: {
        title: "गोपनीयता · QuickDesk",
        description: "QuickDesk में न खाते हैं, न एनालिटिक्स, न टेलीमेट्री। क्या आपके कंप्यूटर पर रहता है और क्या कहाँ भेजा जाता है।",
      },
      sitemap: {
        title: "साइट मैप · QuickDesk",
        description:
          "quickdesk.click के सभी पेज: Linux, Windows और macOS के लिए डाउनलोड, खंड-दर-खंड दस्तावेज़, बदलावों में हर रिलीज़, और गोपनीयता।",
      },
    },
    seo: {
      app: "एक मुफ़्त, ओपन-सोर्स डेस्कटॉप सहायक: एंड-टू-एंड एन्क्रिप्टेड सिंक होने वाले नोट्स, क्लिपबोर्ड इतिहास, पोर्ट मैनेजर, प्रोग्रामिंग भाषा वर्ज़न और AI उपयोग सीमाएँ, हर एक बस एक शॉर्टकट दूर।",
      ogAlt: "QuickDesk: आपकी पूरी डेस्क, बस एक शॉर्टकट दूर।",
    },
    hero: {
      pick: "आपका सिस्टम",
      linux: "Linux के लिए डाउनलोड करें",
      windows: "Windows के लिए डाउनलोड करें",
      macos: "macOS के लिए डाउनलोड करें{beta}",
      macSoon: "macOS: जल्द आ रहा है",
    },
    download: {
      title: "QuickDesk {v} डाउनलोड करें",
      leadMac: "Linux और Windows (x86_64) तथा macOS (Apple Silicon{beta}) के लिए · {date} को रिलीज़। मुफ़्त और ओपन सोर्स (MIT)।",
      leadSoon: "x86_64 पर Linux और Windows के लिए, macOS जल्द · {date} को रिलीज़। मुफ़्त और ओपन सोर्स (MIT)।",
      pick: "आपका सिस्टम",
      devTitle: "Mac वर्ज़न अभी विकास में है",
      devText:
        "macOS के लिए QuickDesk काम करता है, पर यह नया है: अभी हर Mac पर हर चीज़ आज़माई नहीं गई है, और ऑटो-पेस्ट तथा शॉर्टकट अभी बेहतर किए जा रहे हैं। अगर कुछ काम न करे, तो हमें ज़रूर बताएँ।",
      devLink: "GitHub पर समस्या बताएँ",
      deb: ["Ubuntu, Debian, Linux Mint, Pop!_OS", "Ubuntu 22.04 और नए, Debian 12 और नए।"],
      rpm: ["Fedora और दूसरे RPM-आधारित डिस्ट्रो", "Fedora 36 और नए, openSUSE Tumbleweed।"],
      appimage: ["कोई भी दूसरा डिस्ट्रीब्यूशन", "पोर्टेबल: इंस्टॉल की ज़रूरत नहीं। FUSE 2 (libfuse2) चाहिए।"],
      windows: ["Windows 10 और 11", "64-बिट। सिर्फ़ आपके यूज़र के लिए इंस्टॉल होता है: एडमिन अधिकार नहीं चाहिए।"],
      macos: ["macOS 11 या नया", "Apple Silicon (M1 और नए)। डिस्क इमेज खोलें और QuickDesk को Applications में खींचें।"],
      oneMac:
        "Terminal में चलाएँ। यह QuickDesk.app डाउनलोड करता है, उसका SHA-256 जाँचता है, उसे Applications में रखता है और खोल देता है। इस तरह इंस्टॉल करने पर macOS चेतावनी नहीं देता।",
      afterMac:
        "QuickDesk मेन्यू बार में रहता है और अब से खुद अपडेट होता है। ऑटो-पेस्ट के लिए, macOS पूछे तो Privacy & Security → Accessibility में उसे अनुमति दें।",
      gatekeeper: "ब्राउज़र से डाउनलोड करने पर macOS पहली बार खुलने से रोकता है: QuickDesk अभी Apple से notarized नहीं है।",
      gatekeeperHow: "इसे कैसे खोलें",
      openTitle: "QuickDesk को पहली बार खोलना",
      openLead:
        "QuickDesk अभी Apple से notarized नहीं है (उसके लिए पैसे वाला Apple Developer ID चाहिए)। अगर .dmg ब्राउज़र से डाउनलोड हुई है, तो macOS पहली बार खोलते समय “Apple cannot check it for malicious software” कहकर रोक देता है। ऐप में कोई खराबी नहीं है: नीचे में से कोई एक काम एक बार कर लें। ऊपर दिए कमांड से इंस्टॉल किया है, तो यह छोड़ सकते हैं।",
      openA: "System Settings में",
      openASteps: [
        "QuickDesk को Applications में खींचें और खोलें। macOS चेतावनी दे, तो Done पर क्लिक करें।",
        "System Settings → Privacy & Security खोलें और नीचे Security तक स्क्रॉल करें।",
        "“QuickDesk was blocked to protect your Mac” के पास Open Anyway पर क्लिक करें, पासवर्ड या Touch ID से पुष्टि करें, फिर Open पर क्लिक करें।",
      ],
      openOld: "macOS 14 या पुराने पर, Applications में QuickDesk पर Control-क्लिक करके Open चुनें, फिर दोबारा Open करें — यह भी काम करता है।",
      openB: "या Terminal में",
      openBLead: "QuickDesk को Applications में खींचने के बाद यह चलाएँ, फिर QuickDesk को हमेशा की तरह खोलें:",
      get: "डाउनलोड करें",
      sha: "SHA-256",
      all: "सभी चेकसम (SHA256SUMS)",
      notes: "{v} में नया क्या है",
      allChanges: "सभी बदलाव",
      afterLinux: "फिर QuickDesk को ऐप मेन्यू से खोलें। यह टॉप बार में रहता है और अब से खुद अपडेट होता है।",
      afterWindows: "फिर QuickDesk को Start मेन्यू से खोलें। यह सिस्टम ट्रे में रहता है और अब से खुद अपडेट होता है।",
      copy: "कॉपी करें",
      oneTitle: "एक कमांड से इंस्टॉल करें",
      oneLinux:
        "टर्मिनल में चलाएँ। यह आपके डिस्ट्रीब्यूशन के लिए सही पैकेज (.deb, .rpm या AppImage) चुनता है, उसका SHA-256 जाँचता है और इंस्टॉल करता है। पासवर्ड सिर्फ़ इंस्टॉल वाले चरण में माँगा जाता है।",
      oneWindows: "PowerShell में चलाएँ। यह इंस्टॉलर डाउनलोड करता है, उसका SHA-256 जाँचता है और बिना एडमिन अधिकार के आपके यूज़र के लिए QuickDesk इंस्टॉल करता है।",
      oneScript: "स्क्रिप्ट पढ़ें",
      orPick: "या कोई पैकेज चुनें",
      smartscreen: "Windows कह सकता है कि उसने आपके PC की सुरक्षा की: इंस्टॉलर अभी साइन नहीं है। More info → Run anyway चुनें।",
      macTitle: "macOS के लिए QuickDesk आ रहा है",
      macLead: "Mac पर नोट्स, क्लिपबोर्ड इतिहास, पोर्ट और AI सीमाएँ। इसकी घोषणा बदलावों के पेज और GitHub रिलीज़ में होगी।",
      macWatch: "GitHub पर रिलीज़ देखें",
    },
    cta: {
      title: "जब आप तैयार हों, यह तैयार है।",
      lead: "एक बार इंस्टॉल करें और सब कुछ बस एक शॉर्टकट दूर: आपके नोट्स, जो आपने कॉपी किया, इस्तेमाल हो रहे पोर्ट और आपकी AI सीमाएँ।",
      download: "डाउनलोड करें",
      changes: "नया क्या है",
      meta: "वर्ज़न {v} · {date} · मुफ़्त और ओपन सोर्स (MIT)",
      downloading: "डाउनलोड हो रहा है",
      verified: "SHA-256 जाँचा गया",
      installed: "QuickDesk {v} इंस्टॉल हो गया",
      running: "QuickDesk चल रहा है",
      open: "खोलने के लिए",
    },
    intro: {
      tagline: "आपकी पूरी डेस्क। बस एक शॉर्टकट दूर।",
      skip: "छोड़ें",
    },
    docs: {
      title: "दस्तावेज़",
      lead: "इंस्टॉल, शॉर्टकट, सिंक, क्लिपबोर्ड, पोर्ट, AI उपयोग और अपडेट।",
      contents: "विषय-सूची",
      groups: ["शुरुआत", "फ़ीचर", "मदद"],
      sync: "सिंक",
      clipboard: "क्लिपबोर्ड",
      versions: "रनटाइम",
      stuck: "अब भी अटके हैं?",
      ask: "GitHub पर पूछें",
    },
    sitemap: {
      title: "साइट मैप",
      lead: "quickdesk.click का हर पेज और हर खंड।",
      home: "होम",
      demo: "शॉर्टकट आज़माएँ",
      getIt: "QuickDesk पाएँ",
      oneLine: "इंस्टॉल स्क्रिप्ट",
      other: "और कहीं",
      machine: "sitemap.xml (सर्च इंजन के लिए)",
      privacyNote: "न खाते, न एनालिटिक्स, न टेलीमेट्री: क्या आपके कंप्यूटर पर रहता है और क्या कहाँ भेजा जाता है।",
    },
    changelog: {
      title: "बदलाव",
      lead: "QuickDesk की हर रिलीज़ में क्या नया है, क्या बेहतर हुआ और क्या ठीक हुआ।",
      latest: "नवीनतम",
      source: "स्रोत: GitHub पर बदलावों की फ़ाइल",
      english: "अभी अनुवाद नहीं हुआ है; अंग्रेज़ी में दिखाया जा रहा है।",
    },
    notFound: {
      title: "पेज नहीं मिला · QuickDesk",
      description: "यह पेज मौजूद नहीं है।",
      heading: "यह पेज मौजूद नहीं है।",
    },
    demo: {
      refreshed: "उपयोग ताज़ा किया गया",
      paused: "क्लिपबोर्ड इतिहास रोका गया",
      resumed: "क्लिपबोर्ड इतिहास फिर शुरू",
      quit: "बंद करें — असली ऐप में QuickDesk अब बंद हो जाता",
      encrypting: "एन्क्रिप्ट हो रहा है…",
      synced: "एन्क्रिप्ट और सिंक हो गया",
      copied: "कॉपी हो गया",
      pressCopy: "Ctrl+C दबाएँ",
      copy: "कॉपी करें",
      note: "स्टैंडअप — मंगलवार\n• क्लिपबोर्ड इमेज रिलीज़ कीं\n• रिलीज़ प्रक्रिया की समीक्षा\n• कोई रुकावट नहीं",
    },
  },
  content: { home, docs, privacy, changelog },
};

export default hi;
