// Docs in Arabic, by section id (src/data/docs.ts). Shown right to left;
// code, keys and commands stay left to right (site.css).
import { macBeta } from "../../data/release";

const docs: Record<string, { title: string; html: string }> = {
  install: {
    title: "التثبيت",
    html: `<p>أسرع طريقة، على أي توزيعة:</p>
<pre><code>curl -fsSL https://quickdesk.click/install.sh | bash</code></pre>
<p>يختار ملف .deb أو .rpm أو AppImage المناسب لنظامك، ويتحقق من SHA-256 الخاص به ويثبّته (<a href="/install.sh">اقرأ السكربت</a>). أو نزّل حزمة من <a href="/download">صفحة التنزيل</a> وثبّت الملف المنزَّل بالأمر:</p>
<ul>
<li><strong>Ubuntu وDebian وMint وPop!_OS:</strong> <code>sudo apt install ~/Downloads/QuickDesk_&lt;version&gt;_amd64.deb</code></li>
<li><strong>Fedora وopenSUSE:</strong> <code>sudo dnf install ~/Downloads/QuickDesk-&lt;version&gt;-1.x86_64.rpm</code></li>
<li><strong>أي توزيعة أخرى:</strong> اجعل ملف AppImage قابلًا للتنفيذ وشغّله. يحتاج إلى FUSE 2 ‏(<code>libfuse2</code>، وعلى Ubuntu 24.04 ‏<code>libfuse2t64</code>).</li>
</ul>
<p>يحتاج QuickDesk إلى نظام x86_64 من السنوات الأخيرة (glibc 2.34 أو أحدث). ويعمل على أفضل وجه في GNOME، على Wayland وX11 كليهما.</p>
<p><strong>Windows 10 و11 (64 بت):</strong> في PowerShell شغّل <code>irm https://quickdesk.click/install.ps1 | iex</code>، أو نزّل المثبّت من <a href="/download">صفحة التنزيل</a>. يُثبَّت لمستخدمك فقط، دون صلاحيات المسؤول. المثبّت غير موقَّع بعد، لذلك قد يقول Windows إنه حمى جهازك: اختر <em>More info → Run anyway</em>. على Windows تكون الاختصارات <kbd>Win</kbd> <kbd>Alt</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd>، ويعيش QuickDesk في علبة النظام، ولا يحتاج اللصق التلقائي إلى أي إعداد. ويعمل تبويب بيئات التشغيل هناك عبر mise وrustup وuv (انظر <em>إصدارات اللغات</em>).</p>
<p>عند التشغيل الأول تعرض شاشة ترحيب الاختصارات وتقدّم خيارين: <strong>التشغيل عند تسجيل الدخول</strong> (ضروري ليعمل سجل الحافظة وحلقة الذكاء الاصطناعي طوال الوقت) و<strong>اللصق الفوري</strong> (انظر <a href="#auto-paste">اللصق التلقائي</a>). بعد ذلك يعيش QuickDesk في الشريط العلوي.</p>`,
  },
  macos: {
    title: "macOS",
    html: `${macBeta ? '<p class="note"><strong>ما زال قيد التطوير.</strong> يعمل إصدار Mac، لكنه جديد: لم تُجرَّب كل الميزات على كل أجهزة Mac بعد، وما زال اللصق التلقائي والاختصارات قيد التحسين. من فضلك <a href="https://github.com/TankGum/quickdesk/issues">أبلغنا</a> بما لا يعمل.</p>\n' : ""}<p>يعمل QuickDesk على macOS 11 أو أحدث على Apple Silicon ‏(M1 وأحدث). في Terminal شغّل <code>curl -fsSL https://quickdesk.click/install.sh | bash</code>: يضع QuickDesk.app في Applications ويفتحه. أو نزّل ملف <code>.dmg</code> من <a href="/download#macos">صفحة التنزيل</a> واسحب QuickDesk إلى Applications.</p>
<p><strong>فتحه لأول مرة.</strong> لم يُوثَّق QuickDesk لدى Apple بعد، لذلك إذا نُزِّل ملف <code>.dmg</code> من المتصفح يوقف macOS التشغيل الأول برسالة “Apple cannot check it for malicious software”. لا مشكلة في التطبيق؛ نفّذ إحدى الطريقتين مرة واحدة (عند التثبيت بالأمر لا يظهر أي تحذير):</p>
<ol>
<li>افتح QuickDesk من Applications وانقر <em>Done</em> عندما يحذّرك macOS. ثم افتح System Settings → Privacy &amp; Security، ومرّر حتى Security، وانقر <em>Open Anyway</em> بجوار “QuickDesk was blocked to protect your Mac”، وأكّد بكلمة المرور أو Touch ID ثم انقر <em>فتح</em>. على macOS 14 أو أقدم ينجح أيضًا النقر مع Control على QuickDesk ← <em>فتح</em> ← <em>فتح</em>.</li>
<li>أو شغّل <code>xattr -dr com.apple.quarantine /Applications/QuickDesk.app</code> في Terminal، ثم افتح QuickDesk كالمعتاد.</li>
</ol>
<p>تعرض <a href="/download#macos-open">صفحة التنزيل</a> الخطوات نفسها.</p>
<ul>
<li>يعيش QuickDesk في شريط القوائم، دون أيقونة في Dock. تُفتح نوافذه المنبثقة في مساحة العمل (Space) التي أنت فيها، حتى فوق تطبيق في وضع ملء الشاشة.</li>
<li>الاختصارات هي <kbd>⌃</kbd> <kbd>⌥</kbd> <kbd>N</kbd> / <kbd>V</kbd> / <kbd>P</kbd> ‏(Control + Option): أما ⌘⌥V و⌘⌥N فتخص Finder.</li>
<li><strong>اللصق التلقائي</strong> يضغط ⌘V في التطبيق الذي كنت تستخدمه، ولا يسمح macOS بذلك إلا بعد أن تضيف QuickDesk في System Settings → Privacy & Security → <strong>Accessibility</strong>. يسألك macOS في أول مرة تختار فيها عنصرًا؛ وحتى ذلك الحين يُنسَخ العنصر وتضغط ⌘V بنفسك. يُظهر الإعدادات → اللصق التلقائي ما إذا كان مسموحًا.</li>
<li>قد تسأل إصدارات macOS الأحدث عمّا إذا كان يحق لـ QuickDesk قراءة ما تنسخه التطبيقات الأخرى. اختر Allow، وإلا سيبقى سجل الحافظة فارغًا.</li>
<li><strong>استخدام Claude:</strong> على Mac يحفظ Claude Code تسجيل دخوله في Keychain. يعرض تبويب AI قراءته من هناك؛ ثم يسألك macOS مرة واحدة (اختر <em>Always Allow</em>). وكما في الأنظمة الأخرى، يُقرأ تسجيل الدخول فقط ولا يُرسَل إلا إلى <code>api.anthropic.com</code>.</li>
<li>لأن التطبيق موقَّع توقيعًا مخصّصًا (ad hoc)، قد يطلب macOS أذونات Accessibility وKeychain مجددًا بعد التحديث. إن توقف اللصق التلقائي عن العمل، فأزِل QuickDesk من قائمة Accessibility ثم أضفه من جديد.</li>
<li>يعمل تبويب بيئات التشغيل كما على Linux، مع صدفة تسجيل الدخول الخاصة بك (zsh: <code>~/.zshrc</code>). تظهر النسخ المثبَّتة عبر Homebrew لكنها لا تُعدَّل.</li>
</ul>
<p><strong>إزالة التثبيت:</strong> أنهِ QuickDesk من شريط القوائم وانقله من Applications إلى سلة المهملات. تبقى بياناتك في <code>~/Library/Application Support/click.quickdesk</code> حتى تحذف ذلك المجلد.</p>`,
  },
  shortcuts: {
    title: "الاختصارات",
    html: `<table>
<tr><th>الاختصار</th><th>يفتح</th></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>N</kbd></td><td>الملاحظات</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd></td><td>نافذة سجل الحافظة المنبثقة</td></tr>
<tr><td><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd></td><td>المنافذ</td></tr>
<tr><td>غير معيَّن</td><td>نافذة الملاحظة السريعة (عيّن اختصارًا في الإعدادات)</td></tr>
</table>
<p>الضغط على الاختصار مرة أخرى أثناء فتح نافذته يغلقها. غيّر أيًّا منها في <strong>الإعدادات → الاختصارات</strong>؛ وينبّهك QuickDesk عندما تكون التركيبة مستخدمة بالفعل.</p>
<p>في GNOME تُضاف الاختصارات إلى <em>Settings → Keyboard → Custom Shortcuts</em> وتُزال عند إنهاء QuickDesk، لذلك تعمل على Wayland أيضًا. وفي بيئات سطح المكتب الأخرى، اربط <code>quickdesk toggle notes</code> أو <code>quickdesk toggle clipboard</code> أو <code>quickdesk toggle ports</code> أو <code>quickdesk toggle quick-note</code> بمفاتيح في إعدادات لوحة المفاتيح.</p>
<p><strong>لماذا Super + Alt؟</strong> تبتلع أساليب الإدخال الفيتنامية مثل Unikey وBamboo تركيبة Super + Shift + حرف أثناء الكتابة في حقل نصي؛ أما التركيبات التي فيها Alt فتظل تعمل.</p>`,
  },
  notes: {
    title: "الملاحظات",
    html: `<p>الملاحظات بصيغة Markdown مع محرر غني: عناوين، وخط عريض ومائل، وقوائم، وقوائم مهام، واقتباسات، وكتل شيفرة. <kbd>Ctrl</kbd> <kbd>N</kbd> ينشئ ملاحظة جديدة، و<kbd>Ctrl</kbd> <kbd>S</kbd> يحفظ، و<kbd>Ctrl</kbd> <kbd>F</kbd> يبحث. يتجاهل البحث علامات التشكيل، لذا يجد <code>tieng viet</code> عبارة “tiếng Việt”.</p>
<p>ثبّت ملاحظة بالنجمة لتبقيها في الأعلى. يسألك الحذف أولًا ويمكن التراجع عنه لبضع ثوانٍ.</p>`,
  },
  sync: {
    title: "مزامنة الملاحظات بين الحواسيب",
    html: `<p>تتزامن الملاحظات عبر <strong>QuickDesk Cloud</strong>. لا حساب ولا بريد إلكتروني: يحصل حاسوب على <strong>رمز مزامنة</strong>، وتنضم حواسيبك الأخرى به. تتزامن الملاحظات فقط؛ ويبقى سجل الحافظة والإعدادات على كل حاسوب.</p>
<h3>الإعداد</h3>
<ol>
<li>على الحاسوب الأول: <strong>الإعدادات → المزامنة → تشغيل المزامنة</strong>، واختر عبارة مرور (8 أحرف على الأقل).</li>
<li>يعرض QuickDesk <strong>رمز المزامنة</strong> (<code>QD1-…</code>) و<strong>مفتاح الاسترداد</strong>. احفظ مفتاح الاسترداد في مكان آمن؛ فهو يُعرض مرة واحدة فقط.</li>
<li>على كل حاسوب آخر: <strong>الإعدادات → المزامنة → لديّ رمز مزامنة</strong>، ثم أدخل الرمز وعبارة المرور نفسها (أو مفتاح الاسترداد).</li>
</ol>
<p>يمكن عرض رمز المزامنة مجددًا في أي وقت عبر <strong>عرض رمز المزامنة</strong> في الإعدادات → المزامنة.</p>
<h3>متى تتم المزامنة</h3>
<ul>
<li>بعد ثانيتين تقريبًا من تعديل ملاحظة، وعند فتح نافذة من نوافذ QuickDesk.</li>
<li>وفيما عدا ذلك، يتحقق QuickDesk من التغييرات كل دقيقة ما دامت نافذة مفتوحة، وكل 5 دقائق وهو في العلبة. التحقق طلب صغير واحد؛ ولا تنتقل الملاحظات إلا إذا تغيّر شيء.</li>
<li><strong>زامن الآن</strong> يتحقق فورًا. وتُرسَل التعديلات التي أُجريت دون اتصال عند عودتك إلى الشبكة.</li>
<li>إذا تغيّرت ملاحظة على حاسوبين في الوقت نفسه، تُحفظ النسختان: تبقى الأحدث، وتصبح الأخرى نسخة بجوارها. التعديل يغلب الحذف.</li>
</ul>
<h3>الخصوصية والمفاتيح</h3>
<ul>
<li>تُشفَّر الملاحظات على حاسوبك قبل رفعها (XChaCha20-Poly1305، بمفتاح من عبارة مرورك عبر Argon2id). لا يحتفظ QuickDesk Cloud إلا بنص مشفّر تحت معرّف حساب عشوائي، ولا يستطيع قراءة ملاحظاتك.</li>
<li>يلزم <strong>كلاهما</strong>: رمز المزامنة (للوصول إلى البيانات) وعبارة المرور (لفك تشفيرها). تعامل مع رمز المزامنة ككلمة مرور: فبه وحده قد يحذف أحدهم ملاحظاتك المتزامنة.</li>
<li>نسيت عبارة المرور؟ أدخل مفتاح الاسترداد بدلًا منها. ودون أيٍّ منهما لا يمكن فتح النسخة في QuickDesk Cloud، لكن الملاحظات على كل حاسوب تبقى كما هي.</li>
<li>إذا طلب QuickDesk <strong>فتح قفل المزامنة</strong> (مثلًا بعد إعادة ضبط سلسلة مفاتيح النظام)، فأدخل عبارة المرور مرة أخرى.</li>
</ul>
<h3>إيقافها</h3>
<p><strong>إيقاف المزامنة…</strong> يوقف المزامنة على حاسوب واحد ويحتفظ بملاحظاته. فعّل <em>احذف أيضًا ملاحظاتي من QuickDesk Cloud</em> لحذف النسخة المتزامنة لكل الحواسيب.</p>
<p>الحدود لكل رمز مزامنة: 256 ميغابايت و20,000 كائن مخزَّن، وهذا أكثر بكثير مما تحتاجه الملاحظات.</p>`,
  },
  clipboard: {
    title: "سجل الحافظة",
    html: `<p>كل ما تنسخه يُحفَظ: النصوص والصور (لقطات الشاشة) والملفات المنسوخة في مدير الملفات. افتح النافذة المنبثقة بـ <kbd>Super</kbd> <kbd>Alt</kbd> <kbd>V</kbd>، واكتب للبحث، وصفِّ حسب النص أو الصور أو الملفات، واضغط <kbd>Enter</kbd> للصق في التطبيق الذي كنت تستخدمه. <kbd>Ctrl</kbd> <kbd>Enter</kbd> ينسخ فقط، و<kbd>Ctrl</kbd> <kbd>P</kbd> يثبّت، و<kbd>Del</kbd> يحذف.</p>
<ul>
<li>يبقى السجل على هذا الحاسوب ولا يتزامن أبدًا.</li>
<li>ما تضع عليه مديرات كلمات المرور علامة السرية يُتجاهَل. وقبل نسخ شيء حساس، <strong>أوقف الحفظ مؤقتًا</strong> لمدة 15 دقيقة أو ساعة أو حتى تعيد تشغيله (من تبويب الحافظة أو قائمة العلبة).</li>
<li>يحتفظ QuickDesk بآخر 1,000 عنصر؛ والمثبَّتة للأبد. النص حتى 1 ميغابايت، والصور حتى 10 ميغابايت لكل منها و200 ميغابايت إجمالًا.</li>
<li><strong>مسح…</strong> في تبويب الحافظة يحذف كل شيء عدا العناصر المثبَّتة.</li>
</ul>`,
  },
  "auto-paste": {
    title: "اللصق التلقائي",
    html: `<p>عندما تختار عنصرًا، يعود QuickDesk إلى التطبيق الذي كنت تستخدمه ويضغط <kbd>Shift</kbd> <kbd>Insert</kbd>، وهو يعمل في الطرفيات أيضًا. اختر الطريقة في <strong>الإعدادات → اللصق التلقائي</strong>:</p>
<ul>
<li><strong>بوابة سطح المكتب</strong> (الافتراضي): يطلب GNOME الإذن مرة واحدة ويعرض مؤشر تحكم عن بُعد أثناء اللصق.</li>
<li><strong>فوري (لوحة مفاتيح افتراضية)</strong>: بلا أسئلة ولا مؤشر. يطلب تفعيله كلمة مرور المسؤول مرة واحدة ليسمح للبرامج التي تعمل باسمك بإنشاء لوحة مفاتيح افتراضية (الإذن نفسه الذي يستخدمه Steam لأذرع التحكم).</li>
</ul>
<p>أوقف اللصق التلقائي من تبويب الحافظة لتكتفي بالنسخ، ثم اضغط <kbd>Ctrl</kbd> <kbd>V</kbd> بنفسك.</p>`,
  },
  ports: {
    title: "المنافذ",
    html: `<p><kbd>Super</kbd> <kbd>Alt</kbd> <kbd>P</kbd> يعرض كل منفذ TCP يستمع (IPv4 وIPv6) مع العملية وسطر أوامرها الكامل والمستخدم، ويُحدَّث كل ثانيتين. صفِّ حسب المنفذ أو PID أو اسم العملية أو الحاوية.</p>
<ul>
<li><strong>فتح</strong> يفتح <code>http://localhost:&lt;port&gt;</code> في متصفحك؛ و<strong>نسخ PID</strong> و<strong>نسخ الأمر</strong> ينسخانهما.</li>
<li><strong>إنهاء</strong> يسأل أولًا، ثم يوقف العملية بلطف، ولا يعرض الإنهاء القسري إلا إذا لم تتوقف.</li>
<li>المنافذ التي ينشرها Docker تعرض الحاوية والصورة، و<strong>إيقاف</strong> يوقف الحاوية بدلًا من قتل <code>docker-proxy</code>.</li>
<li>عمليات المستخدمين الآخرين (خدمات النظام) تظهر مخفية؛ وفحصها يتطلب صلاحيات root.</li>
</ul>`,
  },
  versions: {
    title: "إصدارات اللغات (بيئات التشغيل)",
    html: `<p>يدير تبويب <strong>بيئات التشغيل</strong> إصدارات لغات البرمجة على هذا الحاسوب (Node.js وPython وRust وGo وJava وRuby وDeno وBun وPHP). يعرض الإصدار الذي تشغّله <em>طرفية جديدة</em> ومن أين يأتي. يسأل QuickDesk صدفة تسجيل الدخول الخاصة بك، لذا فهذا ما تحصل عليه بعد <code>~/.bashrc</code> أو <code>~/.zshrc</code>، لا ما يراه سطح المكتب.</p>
<ul>
<li>اللغات المثبَّتة عبر <strong>nvm</strong> ‏(Node.js) أو <strong>rustup</strong> ‏(Rust) أو <strong>uv</strong> ‏(Python) أو <strong>mise</strong> ‏(Go وJava وRuby وDeno وBun وPHP وغيرها) يمكن تغييرها: <strong>تثبيت إصدار…</strong> يختار من الإصدارات التي تعرضها الأداة (يقبل rustup قناة أو إصدارًا تكتبه)، و<strong>اجعله الافتراضي</strong> يحدد ما تستخدمه الطرفيات الجديدة، و<strong>إلغاء التثبيت</strong> يزيل إصدارًا ويحرّر مساحته. لا يمكن إزالة الإصدار الافتراضي؛ اختر افتراضيًا آخر أولًا.</li>
<li>تعمل عمليات التثبيت في الخلفية مع شريط تقدّم ويمكن إلغاؤها. لا يعمل إلا تغيير واحد في كل مرة. تعرض قائمة الإصدارات افتراضيًا أحدث إصدار من كل سلسلة؛ بدّل إلى LTS أو الكل، أو اكتب للتصفية.</li>
<li>عندما يصدر إصدار أحدث لسلسلة مثبَّتة (Node 22.24.1 بدل 22.23.3، أو Python 3.13.9 بدل 3.13.7)، يُظهر الإصدار <strong>↑ 22.24.1</strong>: نقرة واحدة تثبّته، وإن كان القديم هو الافتراضي تجعله الافتراضي. يبقى الإصدار القديم حتى تزيل تثبيته. أما قنوات Rust فتُحدَّث في مكانها: يُظهر <strong>stable</strong> ‏<strong>↑ 1.93.0</strong> عندما يكون لدى rustup إصدار stable أحدث، والنقرة تشغّل <code>rustup update stable</code>.</li>
<li><strong>التطبيق فورًا في الطرفيات المفتوحة</strong> (أعلى التبويب) يوصل التغييرات إلى الطرفيات المفتوحة بالفعل، بدءًا من موجّهها التالي: يضيف QuickDesk سطرًا إلى ملف صدفتك يحمّل خطّاف موجّه صغيرًا (<code>~/.config/quickdesk/shell/hook.sh</code>). يجعل الخطّاف الصدفة تبحث عن الأوامر من جديد بعد أن يغيّر QuickDesk إصدارًا، ويجعل Node يتبع أقرب <code>.nvmrc</code> عندما تدخل مشروعًا بـ <code>cd</code>، وإلا فالافتراضي الذي تختاره (يبقى <code>nvm use</code> اليدوي حتى تغيّر المجلد أو الافتراضي). ودونه تُطبَّق تغييرات Rust فورًا مع ذلك، وتصل تغييرات Python وmise إلى الطرفيات الجديدة (أو إلى طرفية مفتوحة بعد <code>hash -r</code>)، وتغييرات nvm إلى الطرفيات الجديدة فقط.</li>
<li><strong>تعيين لمشروع…</strong> يكتب ملف الأداة نفسها في مجلد تختاره: <code>.nvmrc</code> أو <code>rust-toolchain.toml</code> أو <code>.python-version</code> أو <code>mise.toml</code> (ويحتفظ mise بالأدوات الأخرى المدرجة فيه).</li>
<li>اللغة التي لا مدير إصدارات لها (Go المثبَّت عبر apt مثلًا) تعرض <strong>تثبيت mise…</strong>: ينزّل QuickDesk أحدث mise من github.com/jdx/mise، ويتحقق منه بمجاميع SHA-256 للإصدار، ويضعه في <code>~/.local/bin</code>. وبعد أن تجعل إصدارًا افتراضيًا، يعرض <strong>إصلاح PATH…</strong> السطر الذي يفعّل mise في صدفتك.</li>
<li>إذا كانت الطرفيات الجديدة ستشغّل نسخة غير الافتراضية (مثلًا يأتي <code>python3</code> الخاص بالنظام أولًا في PATH)، تشرح البطاقة السبب، ويعرض <strong>إصلاح PATH…</strong> الأسطر التي سيضيفها بالضبط. ولا يضيفها QuickDesk إلا بعد موافقتك، ككتلة معلَّمة في نهاية ملف صدفتك مع نسخة احتياطية بجوارها؛ ويمكن إزالة الكتلة من المكان نفسه.</li>
<li>تُكتب إصلاحات PATH وخطّاف الموجّه لـ bash ‏(<code>~/.bashrc</code>) وzsh ‏(<code>~/.zshrc</code>)؛ وfish غير مدعوم بعد.</li>
<li>اللغات غير الموجودة على الحاسوب بعد مدرجة تحت <strong>إضافة لغة</strong>: اختر واحدة لتثبيت mise إن لزم، ثم إصدارًا.</li>
<li><strong>Windows:</strong> يسأل QuickDesk ‏PowerShell عمّا تشغّله الطرفية الجديدة ويعمل عبر <strong>mise</strong> (المثبَّت في <code>%LOCALAPPDATA%\\mise\\bin</code>) و<strong>rustup</strong> و<strong>uv</strong>؛ ويُعرض nvm-windows وfnm دون تعديلهما، ويُتجاهَل الملف البديل <code>python.exe</code> الخاص بالمتجر. يضع <strong>إصلاح PATH…</strong> مجلدات الأداة (وبالنسبة إلى mise ‏shims الخاصة به) في مقدمة PATH الخاص بحسابك — ذاك الموجود في Settings → Environment Variables — دون صلاحيات المسؤول؛ وتُحفظ القيمة القديمة في <code>~\\.config\\quickdesk\\path</code>، واللوحة نفسها تزيل المجلدات مجددًا. تلتقطه الطرفيات الجديدة؛ وبعد ذلك تُطبَّق تغييرات الافتراضي فورًا. لا حاجة إلى “التطبيق فورًا في الطرفيات المفتوحة” على Windows ولا يُعرض هناك.</li>
<li>النسخ التي ثبّتها النظام (apt وdnf) لا تُغيَّر ولا تُزال أبدًا؛ ولا يشغّل QuickDesk ‏<code>sudo</code> أبدًا. أما اللغات التي تديرها أدوات لا يدعمها (fnm وvolta وpyenv وSDKMAN…) فتُعرض للقراءة فقط.</li>
</ul>`,
  },
  ai: {
    title: "استخدام الذكاء الاصطناعي",
    html: `<p>حلقة في الشريط العلوي تُظهر كم استهلكت من حدود الذكاء الاصطناعي، بالأخضر أو الكهرماني أو الأحمر. تعرض قائمتها الأداة التي تتتبعها، مع شريط لكل حد وموعد إعادة ضبطه؛ ويعرض تبويب AI كل الأدوات.</p>
<ul>
<li><strong>Claude Code:</strong> يقرأ QuickDesk تسجيل الدخول الذي حفظه Claude Code على هذا الحاسوب ويسأل Anthropic عن استخدامك، مرة كل 5 دقائق على الأكثر. يُقرأ الرمز فقط: لا يُخزَّن ولا يُجدَّد ولا يُرسَل إلى أي مكان سوى Anthropic.</li>
<li><strong>Codex:</strong> يُقرأ من سجلات الجلسات التي يحفظها Codex في <code>~/.codex</code>.</li>
<li><strong>Gemini ‏(Antigravity):</strong> لا يعطي نسبة مئوية، لذلك يعرض QuickDesk من سجلاته آخر مرة نفدت فيها حصتك.</li>
</ul>
<p>اختر الأداة والحد اللذين تريد متابعتهما من <em>تتبُّع</em>، أو أخفِ الحلقة من تبويب AI.</p>`,
  },
  updates: {
    title: "التحديثات",
    html: `<p>يتحقق QuickDesk من وجود إصدار جديد بعد 30 ثانية من تشغيله ثم كل 6 ساعات. وعندما يصدر إصدار، يخبرك به شريط في النافذة الرئيسية وعنصر في أعلى قائمة العلبة، مع <strong>ما الجديد</strong>. لا يُثبَّت شيء حتى تنقر <strong>حدّث الآن</strong>.</p>
<p>يُتحقَّق من كل تحديث بمفتاح توقيع QuickDesk قبل تثبيته. يطلب ملف .deb أو .rpm كلمة مرور المسؤول؛ ويستبدل AppImage نفسه. ثم يعيد QuickDesk تشغيل نفسه ويعرض ما تغيّر. أوقف التحقق التلقائي، أو تحقّق يدويًا، من <strong>الإعدادات → التحديثات</strong>.</p>`,
  },
  uninstall: {
    title: "إزالة التثبيت",
    html: `<ol>
<li>أوقف <em>تشغيل QuickDesk عند تسجيل الدخول</em> في الإعدادات، وأنهِه من العلبة.</li>
<li>أزِل الحزمة: <code>sudo apt remove quick-desk</code> ‏(Debian/Ubuntu) أو <code>sudo dnf remove QuickDesk</code> ‏(Fedora)، أو احذف ملف AppImage.</li>
<li>تبقى بياناتك في <code>~/.local/share/click.quickdesk</code> حتى تحذف ذلك المجلد.</li>
</ol>
<p><strong>Windows:</strong> Settings → Apps → Installed apps → QuickDesk → Uninstall. تبقى بياناتك في <code>%APPDATA%\\click.quickdesk</code> حتى تحذف ذلك المجلد.</p>
<p>إزالة الحزمة تزيل أيضًا إذن اللصق الفوري إن كنت قد فعّلته.</p>`,
  },
  troubleshooting: {
    title: "حل المشكلات",
    html: `<ul>
<li><strong>اختصار لا يفعل شيئًا:</strong> تحقّق من وجود تعارض في <em>Settings → Keyboard → Custom Shortcuts</em>، أو اختر تركيبة أخرى في الإعدادات → الاختصارات.</li>
<li><strong>اللصق التلقائي لا يكتب شيئًا:</strong> تأكّد أن التطبيق الذي كنت فيه ما زال في المقدّمة؛ وغيّر الطريقة في الإعدادات → اللصق التلقائي. العنصر يُنسَخ دائمًا، لذا يعمل <kbd>Ctrl</kbd> <kbd>V</kbd>.</li>
<li><strong>السجلات:</strong> <code>~/.local/share/click.quickdesk/logs/</code>. أرفقها عندما <a href="https://github.com/TankGum/quickdesk/issues">تبلّغ عن مشكلة</a>.</li>
</ul>`,
  },
};

export default docs;
