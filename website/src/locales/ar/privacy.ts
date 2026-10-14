// Privacy page in Arabic. Keep it in step with src/data/privacy.ts.
const privacy = {
  title: "الخصوصية",
  updated: "آخر تحديث: 14 أكتوبر 2026 (الصور في الملاحظات)",
  html: `<p class="lead">لا حسابات ولا تحليلات ولا قياس عن بُعد في QuickDesk. ملاحظاتك وسجل حافظتك يعيشان على حاسوبك. تسرد هذه الصفحة كل ما يغادره.</p>
<h2>ما يبقى على حاسوبك</h2>
<ul>
<li><strong>الملاحظات وسجل الحافظة (النصوص والصور وقوائم الملفات) والإعدادات والسجلات</strong> في <code>~/.local/share/click.quickdesk</code>.</li>
<li><strong>سجل الحافظة لا يغادر حاسوبك أبدًا</strong>، حتى مع تفعيل المزامنة. ما تضع عليه مديرات كلمات المرور علامة السرية لا يُحفَظ، ويمكنك إيقاف الحفظ مؤقتًا في أي وقت.</li>
<li><strong>المنافذ</strong> تُقرأ من <code>/proc</code> وDocker على جهازك ولا تُخزَّن.</li>
<li>يُحفَظ رمز المزامنة ومفتاح التشفير في سلسلة مفاتيح النظام.</li>
</ul>
<h2>ما يرسله QuickDesk، وإلى أين</h2>
<table>
<tr><th>متى</th><th>إلى أين</th><th>ماذا</th></tr>
<tr><td data-label="متى">التحقق من التحديثات (بعد 30 ثانية من التشغيل، ثم كل 6 ساعات؛ يمكن إيقافه)</td><td data-label="إلى أين"><code>dl.quickdesk.click</code> ‏(Cloudflare)</td><td data-label="ماذا">طلب لملف <code>update.json</code>، والحزمة عندما تختار التحديث. عنوان IP الخاص بك مرئي لـ Cloudflare، كما في أي تنزيل.</td></tr>
<tr><td data-label="متى">مزامنة الملاحظات، فقط إن فعّلتها</td><td data-label="إلى أين">QuickDesk Cloud، <code>sync.quickdesk.click</code> ‏(Cloudflare)</td><td data-label="ماذا">ملاحظاتك والصور الموجودة فيها، مشفّرة على حاسوبك (XChaCha20-Poly1305) بمفتاح مشتق من عبارة مرورك؛ ولا يستطيع QuickDesk Cloud قراءتها. يخزّنها تحت معرّف حساب مزامنة عشوائي، مع أحجامها وأوقات رفعها، ويرى عنوان IP الخاص بك. لا بريد إلكتروني ولا اسم ولا بيانات شخصية أخرى. يمكنك حذف كل شيء من الإعدادات → المزامنة.</td></tr>
<tr><td data-label="متى">استخدام الذكاء الاصطناعي، فقط إن كان Claude Code مسجَّل الدخول على هذا الحاسوب (على Mac، فقط بعد أن تسمح بقراءة تسجيل دخوله من Keychain)</td><td data-label="إلى أين"><code>api.anthropic.com</code></td><td data-label="ماذا">طلب استخدام مصرَّح به بتسجيل دخول Claude Code الحالي، مرة كل 5 دقائق على الأكثر. يُقرأ الرمز، ولا يُخزَّن ولا يُجدَّد ولا يُرسَل إلى أي مكان آخر أبدًا.</td></tr>
<tr><td data-label="متى">تبويب بيئات التشغيل، عندما يبحث عن إصدارات أحدث (مرة في الساعة على الأكثر)، وعندما تفتح قائمة إصدارات أو تثبّت إصدارًا</td><td data-label="إلى أين">خوادم nvm وuv وrustup وmise: <code>nodejs.org</code> و<code>github.com</code> / <code>releases.astral.sh</code> (إصدارات Python المبنية) و<code>static.rust-lang.org</code>، وبالنسبة إلى mise موقع التنزيل الخاص بكل لغة (مثل <code>go.dev</code>)</td><td data-label="ماذا">تلك الأدوات هي التي ترسل الطلبات بنفسها، كما حين تشغّلها في الطرفية. ولا يضيف QuickDesk إليها شيئًا.</td></tr>
<tr><td data-label="متى">تبويب بيئات التشغيل، فقط إن نقرت “تثبيت mise”</td><td data-label="إلى أين"><code>github.com</code> (إصدارات jdx/mise)؛ وعلى Windows أيضًا <code>api.github.com</code></td><td data-label="ماذا">تنزيل أحدث mise وملف المجاميع الاختبارية الخاص به. على Windows يسأل QuickDesk أولًا واجهة GitHub البرمجية عن أحدث إصدار (User-Agent “QuickDesk”).</td></tr>
<tr><td data-label="متى">اللصق التلقائي عبر بوابة سطح المكتب</td><td data-label="إلى أين">سطح مكتبك أنت (GNOME)، لا الشبكة</td><td data-label="ماذا">ضغطة على Shift + Insert.</td></tr>
</table>
<p>يُقرأ استخدام Codex وAntigravity من ملفات السجل على حاسوبك؛ ولا يُرسَل شيء.</p>
<h2>هذا الموقع</h2>
<p>quickdesk.click موقع ثابت على Cloudflare Pages، بلا ملفات تعريف ارتباط ولا تحليلات ولا تتبّع. يحمّل الخطوط من Google Fonts، لذلك ترى Google عنوان IP الخاص بك. يعمل العرض التجريبي كليًا في متصفحك على بيانات تجريبية. والنظام الذي تختاره (Linux أو Windows أو macOS) والسمة الفاتحة أو الداكنة، إن غيّرتها، يُحفظان في التخزين المحلي لمتصفحك ولا يُرسَلان إلى أي مكان.</p>
<h2>عدّ التنزيلات</h2>
<p>لمعرفة عدد من يثبّتون QuickDesk ويحدّثونه، يعدّ <code>dl.quickdesk.click</code> كل تنزيل وكل تحقق من التحديثات: أي ملف (إصداره ونظامه)، ونوع العميل الذي طلبه (متصفح قادم من هذا الموقع أو من مكان آخر، أو طرفية، أو PowerShell، أو أداة تحديث التطبيق)، والبلد الذي تذكره Cloudflare للطلب. لا تُحفظ إلا المجاميع اليومية (في قاعدة بيانات Cloudflare D1): لا عنوان IP ولا سلسلة المتصفح ولا ملف تعريف ارتباط ولا أي معرّف آخر ولا سجل لكل طلب، لذا لا يمكن ربط الأعداد بشخص أو بحاسوب.</p>
<h2>أسئلة</h2>
<p>QuickDesk مفتوح المصدر: كل ما ورد هنا يمكن التحقق منه في <a href="https://github.com/TankGum/quickdesk">الشيفرة المصدرية</a>. اسأل أو أبلغ عن مشكلة في <a href="https://github.com/TankGum/quickdesk/issues">GitHub issues</a>.</p>`,
};

export default privacy;
