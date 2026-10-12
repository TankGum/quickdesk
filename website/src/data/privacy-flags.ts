// One sentence for the privacy page, added to "This site" only when the site
// was built with a GrowthBook client key (QD_GROWTHBOOK_CLIENT_KEY); without
// one the site makes no such request and the page does not mention it.
import type { Lang } from "../i18n";

export const GROWTHBOOK_ON = Boolean(import.meta.env.QD_GROWTHBOOK_CLIENT_KEY);

const sentence: Record<Lang, string> = {
  en: "The site’s feature flags come from GrowthBook: your browser asks <code>cdn.growthbook.io</code> which features to show, so GrowthBook sees your IP address. No cookies or identifiers are sent.",
  vi: "Các cờ tính năng của trang đến từ GrowthBook: trình duyệt của bạn hỏi <code>cdn.growthbook.io</code> nên hiện tính năng nào, nên GrowthBook thấy địa chỉ IP của bạn. Không gửi cookie hay định danh nào.",
  zh: "网站的功能开关来自 GrowthBook：你的浏览器会向 <code>cdn.growthbook.io</code> 询问要显示哪些功能，因此 GrowthBook 能看到你的 IP 地址。不会发送任何 Cookie 或标识符。",
  ja: "サイトの機能フラグは GrowthBook から取得します。ブラウザーが <code>cdn.growthbook.io</code> に表示する機能を問い合わせるため、GrowthBook に IP アドレスが見えます。Cookie や識別子は送信されません。",
  hi: "साइट के फ़ीचर फ़्लैग GrowthBook से आते हैं: आपका ब्राउज़र <code>cdn.growthbook.io</code> से पूछता है कि कौन-सी सुविधाएँ दिखानी हैं, इसलिए GrowthBook आपका IP पता देख सकता है। कोई कुकी या पहचानकर्ता नहीं भेजा जाता।",
  es: "Los indicadores de funciones del sitio vienen de GrowthBook: tu navegador pregunta a <code>cdn.growthbook.io</code> qué funciones mostrar, así que GrowthBook ve tu dirección IP. No se envían cookies ni identificadores.",
  fr: "Les indicateurs de fonctionnalités du site viennent de GrowthBook : votre navigateur demande à <code>cdn.growthbook.io</code> quelles fonctionnalités afficher, donc GrowthBook voit votre adresse IP. Aucun cookie ni identifiant n’est envoyé.",
  ar: "تأتي أعلام الميزات في الموقع من GrowthBook: يسأل متصفحك <code>cdn.growthbook.io</code> عن الميزات التي يجب عرضها، لذا يرى GrowthBook عنوان IP الخاص بك. لا يتم إرسال أي ملفات تعريف ارتباط أو معرّفات.",
  bn: "সাইটের ফিচার ফ্ল্যাগ আসে GrowthBook থেকে: আপনার ব্রাউজার <code>cdn.growthbook.io</code>-কে জিজ্ঞেস করে কোন ফিচার দেখাতে হবে, তাই GrowthBook আপনার IP ঠিকানা দেখতে পায়। কোনো কুকি বা শনাক্তকারী পাঠানো হয় না।",
  ru: "Флаги функций сайта берутся из GrowthBook: ваш браузер спрашивает у <code>cdn.growthbook.io</code>, какие функции показывать, поэтому GrowthBook видит ваш IP-адрес. Cookie и идентификаторы не отправляются.",
  pt: "Os sinalizadores de recursos do site vêm do GrowthBook: seu navegador pergunta a <code>cdn.growthbook.io</code> quais recursos mostrar, então o GrowthBook vê seu endereço IP. Nenhum cookie ou identificador é enviado.",
  id: "Penanda fitur situs berasal dari GrowthBook: browser Anda bertanya ke <code>cdn.growthbook.io</code> fitur mana yang ditampilkan, jadi GrowthBook melihat alamat IP Anda. Tidak ada cookie atau pengenal yang dikirim.",
};

/**
 * Add the sentence at the end of the "This site" section: the fourth <h2>
 * block of the page (stays, sends, this site, download counts, questions).
 */
export function withGrowthbook(html: string, lang: Lang): string {
  if (!GROWTHBOOK_ON) return html;
  const parts = html.split("<h2>");
  if (parts.length !== 6) return html;
  parts[3] = parts[3].replace(/<\/p>\s*$/, ` ${sentence[lang]}</p>\n`);
  return parts.join("<h2>");
}
