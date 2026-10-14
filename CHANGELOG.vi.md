# Nhật ký thay đổi

Mỗi bản phát hành QuickDesk có gì mới. Bản tiếng Anh là
[CHANGELOG.md](CHANGELOG.md); hai file cần có cùng các phiên bản.

Mỗi bản là một mục `## <phiên bản> — <ngày>` với tối đa ba nhóm:
`### Mới`, `### Cải thiện`, `### Sửa lỗi`.

## Unreleased

### Cải thiện
- Menu usage AI trên thanh trên cùng vẽ mỗi giới hạn bằng thanh mịn và thấp hơn: 25 ô bằng hai ký tự khối mảnh, nét dày cho phần đã dùng và nét mảnh cho phần còn lại, thay cho 10 ô vuông màu to. Thanh không còn màu (chỉ cần một emoji là dòng menu cao lên); phần trăm và vòng tròn vẫn cho biết đã dùng bao nhiêu. Dòng làm mới giờ chỉ ghi “Làm mới”.

## 0.7.1 — 2026-10-13

### Sửa lỗi
- macOS: không còn hỏi mật khẩu khi lấy usage Claude. QuickDesk giờ đọc phiên đăng nhập của Claude Code trong Keychain theo cùng cách Claude Code tự đọc, nên macOS không hỏi lại mỗi lần QuickDesk khởi động hay Claude Code làm mới phiên đăng nhập.

## 0.7.0 — 2026-10-12

### Mới
- Ứng dụng có các ngôn ngữ như website: ngoài tiếng Anh và tiếng Việt còn có tiếng Trung (giản thể), Nhật, Hindi, Tây Ban Nha, Pháp, Ả Rập (viết từ phải sang trái), Bengali, Nga, Bồ Đào Nha (Brazil) và Indonesia. Chọn trong Cài đặt → Ngôn ngữ; “Giống hệ thống” giờ cũng theo ngôn ngữ của macOS và Windows. Menu khay và phần “có gì mới” cũng theo ngôn ngữ đó.

## 0.6.3 — 2026-10-12

### Cải thiện
- macOS: QuickDesk được ký bằng chứng chỉ riêng. Sau bản cập nhật này macOS hỏi Keychain (chọn “Always Allow”) và quyền Accessibility lần cuối; các bản cập nhật sau giữ nguyên cả hai, không còn phải nhập mật khẩu sau mỗi lần cập nhật.

## 0.6.2 — 2026-10-12

### Sửa lỗi
- macOS: popup clipboard và popup ghi chú nhanh không còn làm app bạn đang gõ bị mất focus. Chúng mở dạng panel nhận bàn phím trong khi app kia vẫn active và ô nhập vẫn giữ focus, nên chọn một mục clipboard là dán đúng vào đó.
- macOS: Keychain hỏi mật khẩu ít hơn hẳn. Trước đây QuickDesk đọc khóa sync từ Keychain mỗi lần sync (mỗi phút khi đang mở cửa sổ, hai lần ngay lúc khởi động) và đọc phiên đăng nhập Claude Code mỗi 5 phút; giờ mỗi mục chỉ đọc một lần mỗi lần chạy, phiên Claude chỉ đọc lại khi nó hết hiệu lực. Khi macOS hỏi, hãy chọn “Always Allow”.

## 0.6.1 — 2026-10-12

### Sửa lỗi
- macOS: chọn một mục trong popup clipboard lại dán đúng vào ô bạn đang gõ. QuickDesk giờ tự lùi lại để macOS trả focus về đúng app và ô đó, thay vì yêu cầu kích hoạt app, điều mà các bản macOS mới có thể chỉ làm nửa vời.

## 0.6.0 — 2026-10-11

### Mới
- QuickDesk cho macOS (Apple Silicon, macOS 11 trở lên): ghi chú có đồng bộ, lịch sử clipboard gồm chữ, ảnh và file, tự dán bằng ⌘V vào app bạn đang dùng, cổng, version ngôn ngữ lập trình và usage AI, ngay trên menu bar. Phím tắt là ⌃⌥N / V / P. Cài từ Terminal bằng `curl -fsSL https://quickdesk.click/install.sh | bash` hoặc bằng file .dmg ở trang tải về; app tự cập nhật như trên các hệ điều hành khác. App chưa được Apple notarize: nếu tải file .dmg bằng trình duyệt, macOS sẽ chặn lần mở đầu tiên, và trang tải về có hướng dẫn cách mở (System Settings → Privacy & Security → Open Anyway).
- Trên Mac, tab AI cũng hiện được hạn mức Claude: ở đó Claude Code giữ phiên đăng nhập trong Keychain, và QuickDesk chỉ đọc nó sau khi bạn cho phép.

### Sửa lỗi
- Script cài đặt báo rõ bản phát hành thiếu gói nào, thay vì dừng mà không nói gì.

## 0.5.0 — 2026-10-11

### Mới
- QuickDesk cho Windows 10 và 11 (64-bit): ghi chú có đồng bộ, lịch sử clipboard gồm chữ, ảnh và file, tự dán vào ứng dụng bạn đang dùng, cổng và usage AI, với cùng phím tắt Win+Alt+N / V / P. Cài từ PowerShell bằng `irm https://quickdesk.click/install.ps1 | iex` hoặc từ trang tải về; bộ cài chỉ cài cho tài khoản của bạn, không cần quyền quản trị, và tự cập nhật. Tab Runtime làm việc qua mise, rustup và uv: cài mise bằng một nút bấm, thêm Node, Python, Go và các ngôn ngữ khác, và “Sửa PATH…” đưa thư mục của chúng vào PATH của tài khoản mà không cần quyền administrator.
- Runtime: ngôn ngữ chưa có trên máy nằm trong mục “Thêm ngôn ngữ”, cài được ngay qua mise.

### Cải thiện
- Runtime: các channel của Rust cũng có nút cập nhật. Khi rustup có bản stable (hoặc beta, nightly) mới hơn, channel đó hiện ↑ và một bấm là chạy `rustup update` cho nó.

## 0.4.0 — 2026-10-10

### Mới
- Tab Runtime cho version các ngôn ngữ lập trình: xem terminal mới thật sự chạy Node.js, Python, Rust, Go… bản nào và lấy từ đâu, cài và gỡ version, chọn bản mặc định và ghim version cho từng project. QuickDesk làm việc qua nvm, uv, rustup và mise, và không bao giờ đụng tới gói của hệ thống.
- Khi dòng version đang cài có bản mới hơn (ví dụ Node 22.24.1 so với 22.23.3), version đó hiện nút cập nhật; bản cũ vẫn được giữ cho tới khi bạn gỡ.
- Ngôn ngữ chưa có trình quản lý version (ví dụ Go cài từ apt) có thể cài mise bằng một nút bấm, tải từ GitHub của mise và kiểm tra với mã SHA-256 của bản phát hành.
- “Áp dụng ngay trong các terminal đang mở”: thay đổi đến được cả terminal đang mở ở lần nhấn Enter tiếp theo, và Node tự theo `.nvmrc` gần nhất khi bạn `cd`. Khi PATH chen ngang, tab nói rõ lý do và đề xuất đúng các dòng cần thêm vào file shell, kèm bản sao lưu.

### Cải thiện
- Trang tải về hiện đủ lệnh để tải và cài thẳng từ terminal.
- Danh sách version lọc được theo bản mới nhất của mỗi dòng, chỉ LTS, hoặc tất cả.

## 0.3.1 — 2026-10-09

### Mới
- Đồng bộ có sẵn trong app: bật trong Cài đặt → Đồng bộ và cho các máy khác tham gia bằng mã đồng bộ. Không cần tài khoản, email hay tự cài bucket. Ghi chú vẫn được mã hoá trên máy trước khi gửi đi; QuickDesk Cloud chỉ lưu dữ liệu đã mã hoá.

### Sửa lỗi
- Ghi chú, lịch sử clipboard và cài đặt của các bản trước đã trở lại. Khi cập nhật lên 0.2.0 và 0.3.0, app bắt đầu với thư mục trống thay vì chuyển dữ liệu sang; giờ QuickDesk tự gộp dữ liệu cũ vào, và giữ nguyên các thư mục cũ làm bản dự phòng.
- Mở Cài đặt không còn làm chấm màu của các tab khác chuyển sang xám.

### Cải thiện
- Không còn hỗ trợ đồng bộ qua bucket S3/R2 riêng. Nếu bạn đang dùng, đồng bộ sẽ tự tắt một lần kèm thông báo trong Cài đặt → Đồng bộ; ghi chú vẫn giữ nguyên trên máy.
- Menu usage AI trên thanh trên cùng chỉ hiện công cụ bạn đang theo dõi, mỗi hạn mức có một thanh tiến trình xanh, vàng hoặc đỏ và thời điểm đặt lại ở dòng dưới. Chọn công cụ khác trong mục Theo dõi.

## 0.3.0 — 2026-10-09

### Mới
- QuickDesk có địa chỉ riêng: [quickdesk.click](https://quickdesk.click). Gói tải về và bản cập nhật đều lấy từ đây.
- Trang web đầy đủ: trang tải về tự gợi ý gói phù hợp, trang tài liệu, nhật ký thay đổi và quyền riêng tư, có tiếng Anh và tiếng Việt.

### Cải thiện
- Dữ liệu giờ nằm ở `~/.local/share/click.quickdesk`. QuickDesk tự chuyển dữ liệu sang đó ở lần mở đầu tiên; ghi chú, lịch sử clipboard và cài đặt vẫn giữ nguyên.
- Khởi động lại sau khi cập nhật ổn định hơn.

## 0.2.2 — 2026-10-09

### Mới
- QuickDesk giờ tự cập nhật. App tự kiểm tra bản mới trong nền, cho bạn biết có gì thay đổi và chỉ cài khi bạn đồng ý. Mỗi bản cập nhật đều được kiểm tra chữ ký số của QuickDesk trước khi cài.
- Cài đặt → Cập nhật: xem phiên bản đang dùng, kiểm tra ngay, hoặc tắt tự động kiểm tra.
- Sau mỗi lần cập nhật, QuickDesk cho bạn xem có gì mới (một lần).
- Trang tải về có tiếng Việt và có link tới mã nguồn trên GitHub.

### Cải thiện
- Dùng chung một icon ở mọi nơi: dock, menu ứng dụng, cửa sổ và khay hệ thống.

## 0.2.1 — 2026-10-08

### Mới
- Giao diện mới, dùng chung cho app và trang tải về: mỗi mục một màu (ghi chú, clipboard, cổng, AI), gợi ý phím tắt trong popup và thanh tab kiểu macOS.
- Icon nét mảnh thay cho emoji trong toàn bộ app.
- Menu trên khay hệ thống gọn hơn theo kiểu macOS: tên ngắn, dấu ✓ cho "Lưu lịch sử clipboard", hạn mức AI chia nhóm theo từng công cụ.

### Cải thiện
- Demo trên trang tải về chính là app thật chạy với dữ liệu mẫu.

## 0.2.0 — 2026-10-08

### Mới
- Có gói cho Ubuntu/Debian (.deb), Fedora (.rpm) và các distro khác (AppImage), phát hành trên trang tải QuickDesk.

### Cải thiện
- Dữ liệu của bạn được tự chuyển sang thư mục dữ liệu mới của QuickDesk (`~/.local/share/io.github.tankgum.quickdesk`).
