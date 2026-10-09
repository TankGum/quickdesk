# Nhật ký thay đổi

Mỗi bản phát hành QuickDesk có gì mới. Bản tiếng Anh là
[CHANGELOG.md](CHANGELOG.md); hai file cần có cùng các phiên bản.

Mỗi bản là một mục `## <phiên bản> — <ngày>` với tối đa ba nhóm:
`### Mới`, `### Cải thiện`, `### Sửa lỗi`.

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
