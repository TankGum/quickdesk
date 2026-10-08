# QuickDesk — Technical Spec

Status: draft v1 · 2026-10-07
Inputs: project idea (Toolbox: Quick Notes, Clipboard, Port Manager), `spikes/wayland/FINDINGS.md`.

## 1. Mục tiêu

Một app desktop chạy nền (tray) gom các tiện ích developer dùng hằng ngày, mở bằng hotkey, phản hồi tức thì.

| Module | Mục tiêu | Lưu trữ | Sync |
|---|---|---|---|
| Quick Notes | Ghi một thứ trong 2 giây, tìm lại được trên mọi máy | SQLite + FTS5 | E2E encrypted qua S3/R2 |
| Clipboard | Thay thế Win+V: lịch sử clipboard, tìm kiếm | SQLite + FTS5 | Không (local-only) |
| Port Manager | Biết ngay port nào do process nào chiếm, kill/open nhanh | Không (scan OS mỗi lần mở) | Không |

**Non-goals (MVP):** clipboard image/file (Phase 2/3), auto-paste vào app trước đó trên Wayland, multi-user/sharing, mobile, rich-text notes.

**Mục tiêu hiệu năng:**
- Hotkey → popup hiện và gõ được: < 150ms
- Search 10k notes / 1k clipboard entries: < 30ms
- RAM khi idle: < 80MB

## 2. Nền tảng

| Platform | Global hotkey | Clipboard watcher | Ghi chú |
|---|---|---|---|
| Ubuntu GNOME < 48, Wayland (máy chính) | GNOME custom keybinding → `quickdesk toggle <module>` | X11/XFixes qua Xwayland | Đã verify bằng spike |
| GNOME ≥ 48 / KDE, Wayland | Portal `GlobalShortcuts` | KDE: `ext-data-control`; GNOME: X11/Xwayland | Sau MVP |
| Linux X11 session | `tauri-plugin-global-shortcut` | X11/XFixes | |
| Windows 10/11 | `tauri-plugin-global-shortcut` | `AddClipboardFormatListener` | |
| macOS | `tauri-plugin-global-shortcut` | Poll `NSPasteboard.changeCount` mỗi 500ms | Không có API dạng notify |

**Quy tắc Linux rút ra từ spike:**
- UI luôn chạy Wayland-native. **Không** ép `GDK_BACKEND=x11`, vì popup sẽ mất focus.
- Clipboard watcher là một thread riêng, mở kết nối X11 tới Xwayland, chạy ngay trong process chính.

Hotkey mặc định (đổi được trong Settings):

| Hotkey | Hành động |
|---|---|
| `Super+Alt+N` | Notes manager (main window, tab Notes) |
| `Super+Alt+V` | Clipboard popup |
| `Super+Alt+P` | Port Manager |
| *(không đặt)* | Popup ghi nhanh (có trong tray) |

> Ban đầu dùng `Super+Shift+…`. Đã đổi vì IBus Unikey/Bamboo nuốt `Super+Shift+chữ` khi đang ở trong ô nhập liệu, chỉ cho tổ hợp có Ctrl/Alt đi qua. Cấu hình cũ chưa sửa sẽ tự được nâng cấp. Đổi được trong Settings → Hotkeys: có ghi phím, phát hiện trùng với shortcut GNOME, và cảnh báo về bộ gõ.

## 3. Kiến trúc

### 3.1 Process & window model

Toàn bộ chạy trong **một process Tauri 2** (single-instance). Các cửa sổ được **tạo sẵn và ẩn ngay khi khởi động**; hotkey chỉ show/hide, không tạo mới.

| Window label | Kích thước | Hành vi |
|---|---|---|
| `note-popup` | ~480×140, không viền, skip taskbar | Ẩn khi mất focus hoặc Esc |
| `clip-popup` | ~520×420, không viền, skip taskbar | Ẩn khi mất focus hoặc Esc |
| `main` | Cửa sổ thường, có tab Notes / Clipboard / Ports / Settings | `Super+Shift+P` mở thẳng tab Ports |

Đường đi của một lệnh từ ngoài vào:

```
GNOME keybinding ──► `quickdesk toggle notes` (process mới)
                         │ tauri-plugin-single-instance chuyển argv
                         ▼
                  process đang chạy ──► WindowManager::toggle("note-popup")
```

Tray menu (trên Ubuntu cần extension AppIndicator, mặc định đã bật):
- Quick note
- Clipboard
- Ports
- Pause clipboard history
- Settings
- Quit

Nếu môi trường không có tray thì chạy lại `quickdesk` sẽ mở cửa sổ `main`.

### 3.2 Repo layout

```
quickdesk/
├── Cargo.toml                 # Rust workspace
├── crates/
│   ├── qd-core/               # db pool, migrations, settings, ids (UUIDv7), HLC, error, event bus
│   ├── qd-platform/           # session detect, hotkey backends, clipboard watcher backends
│   ├── qd-notes/              # repository + service, không phụ thuộc Tauri
│   ├── qd-clipboard/
│   ├── qd-ports/
│   └── qd-sync/               # M5: crypto, blob transport, sync engine
├── src-tauri/                 # app crate: wiring, tauri commands, tray, windows, plugins
│   └── src/commands/{notes,clipboard,ports,settings}.rs
├── src/                       # React + TypeScript (Vite)
│   ├── windows/               # NotePopup, ClipPopup, Main (route theo window label)
│   ├── modules/{notes,clipboard,ports}/   # api.ts, components/, hooks/
│   ├── shared/                # UI kit, hooks dùng chung
│   └── shared/ipc.ts          # binding IPC viết tay (xem §9)
├── spikes/
└── docs/
```

**Nguyên tắc:**
- Các crate module (`qd-notes`, `qd-clipboard`, `qd-ports`) **không import Tauri**. Chúng chỉ chứa logic và repository, test được bằng `cargo test` với SQLite in-memory.
- `src-tauri` là lớp mỏng: nhận command, gọi service, phát event.
- Thêm module mới (snippets, screenshot, …) gồm 4 bước:
  1. tạo crate mới
  2. khai báo migrations
  3. viết file commands
  4. tạo thư mục `src/modules/<x>`

  Không phải sửa core.

```rust
// qd-core
pub trait Module: Send + Sync {
    fn id(&self) -> &'static str;                 // "notes", "clipboard"
    fn migrations(&self) -> &'static [Migration]; // chạy theo thứ tự, lưu vào schema_migrations
}
```

### 3.3 Stack

| Phần | Lựa chọn | Lý do |
|---|---|---|
| Shell | Tauri 2 | Nhẹ, đa nền tảng |
| DB | `rusqlite` (feature `bundled`) + WAL | FTS5 và trigram có sẵn, cùng một phiên bản SQLite trên mọi OS |
| Migrations | Runner tự viết trong `qd-core`, bảng `schema_migrations(module, version)` | Mỗi module tự quản lý schema của mình |
| IPC types | Viết tay trong `src/shared/ipc.ts` | `tauri-specta` vẫn đang ở RC; hoãn lại (xem §9) |
| Clipboard write | `tauri-plugin-clipboard-manager` | |
| X11 watcher | `x11rb` (XFixes) | Pure Rust, không cần header X11 |
| Ports | Linux: tự parse `/proc/net/tcp{,6}`; Win/mac: crate `listeners`; `sysinfo` để lấy cmdline/kill | Xem §5.3 |
| Crypto | `chacha20poly1305` (XChaCha20-Poly1305), `argon2`, `rand` | |
| Blob storage | `object_store` (S3-compatible: R2, S3, MinIO) | Hỗ trợ list-with-offset và conditional PUT |
| Secrets | `keyring` (Secret Service / Credential Manager / Keychain) | Lưu S3 credentials và data key |
| Plugins | `single-instance`, `autostart`, `global-shortcut`, `opener`, `clipboard-manager` | |
| Frontend | React + TS + Vite, TanStack Query cho IPC cache | |
| Logging | `tracing` → file xoay vòng trong app log dir | |

### 3.4 Quy ước IPC

- Command đặt tên `<module>_<action>`, ví dụ `notes_create`, `clip_search`, `ports_kill`. Trả về `Result<T, AppError>`, trong đó `AppError { code, message }` được serialize ra JSON.
- Event đặt tên `<module>://<event>`, ví dụ `clipboard://added`, `notes://changed`, `sync://status`.
- Mọi timestamp là unix milliseconds dạng `i64`. UI tự format theo giờ local.

## 4. Core

### 4.1 Database

- Chỉ một file `quickdesk.db` trong app data dir (`~/.local/share/quickdesk/`, `%APPDATA%\quickdesk\`, `~/Library/Application Support/quickdesk/`).
- Bật `PRAGMA journal_mode=WAL; foreign_keys=ON; busy_timeout=3000`.
- Có một connection ghi duy nhất, đặt sau `Mutex`, và một pool nhỏ connection chỉ đọc.
- Bảng của mỗi module có prefix riêng: `notes_*`, `clip_*`.
- Bảng core:

```sql
CREATE TABLE schema_migrations (module TEXT, version INTEGER, applied_at INTEGER, PRIMARY KEY (module, version));
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);  -- value là JSON; local-only
```

### 4.2 Device identity & HLC

- `device_id` là UUIDv7, sinh ở lần chạy đầu và lưu trong `settings`.
- Mỗi node có một HLC (hybrid logical clock) dạng `{physical_ms}-{counter}-{device_id}`, so sánh theo thứ tự từ điển. Sync dùng HLC để quyết định thứ tự, không phụ thuộc đồng hồ máy bị lệch.

### 4.3 Hotkey

```rust
trait HotkeyBackend {
    fn register(&self, action: Action, accel: &Accelerator) -> Result<()>;
    fn unregister_all(&self) -> Result<()>;
}
```

| Backend | Khi nào dùng | Cách làm |
|---|---|---|
| `PluginBackend` | Windows, macOS, Linux X11 | `tauri-plugin-global-shortcut` |
| `GnomeKeybindingBackend` | `XDG_SESSION_TYPE=wayland` và GNOME < 48 | Ghi `org.gnome.settings-daemon.plugins.media-keys` custom-keybindings dưới path `…/custom-keybindings/quickdesk-<action>/`. **Merge** với danh sách sẵn có của user, không ghi đè. Khi tắt hoặc gỡ app thì xoá đúng các path `quickdesk-*` |
| `PortalBackend` | GNOME ≥ 48, KDE | Sau MVP |

Settings có nút "Kiểm tra hotkey" để báo nếu phím đã bị chiếm.

CLI: `quickdesk toggle notes|clipboard|ports`, `quickdesk show main`.

## 5. Modules

### 5.1 Quick Notes

**Schema** (sync-ready ngay từ M2):

```sql
CREATE TABLE notes (
  id          TEXT PRIMARY KEY,          -- UUIDv7
  body        TEXT NOT NULL,
  pinned      INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL,          -- để hiển thị
  hlc         TEXT NOT NULL,             -- để sync: version hiện tại
  base_hlc    TEXT,                      -- hlc lúc bản local bắt đầu được sửa (dùng phát hiện conflict)
  deleted_at  INTEGER,                   -- tombstone, không xoá cứng
  conflict_of TEXT,                      -- id note gốc nếu đây là bản conflict
  dirty       INTEGER NOT NULL DEFAULT 1 -- chờ push
);
CREATE INDEX notes_list ON notes (deleted_at, pinned DESC, updated_at DESC);
CREATE INDEX notes_dirty ON notes (dirty) WHERE dirty = 1;

CREATE VIRTUAL TABLE notes_fts USING fts5(
  body, content='notes', content_rowid='rowid',
  tokenize = 'unicode61 remove_diacritics 2'   -- "ghi chu" khớp "ghi chú"
);
-- triggers AFTER INSERT/UPDATE/DELETE giữ notes_fts đồng bộ
```

**Commands:**
- `notes_create(body)`
- `notes_update(id, body?, pinned?)`
- `notes_delete(id)` (soft delete)
- `notes_list(cursor, limit)`
- `notes_search(query, limit)` — FTS prefix match, sắp theo `bm25()` rồi `updated_at`

**UX popup:**

| Phím | Hành động |
|---|---|
| `Enter` | Lưu và ẩn popup |
| `Shift+Enter` | Xuống dòng |
| `Esc` | Ẩn popup, giữ draft trong RAM |
| `Ctrl+Enter` | Lưu và mở cửa sổ main |

Lưu note phải chạy hoàn toàn local và không bao giờ chờ sync.

**Main tab:** danh sách (pinned ở trên cùng), ô search, sửa inline, pin/unpin, xoá có Undo trong 5 giây.

### 5.2 Clipboard

**Watcher** (`qd-platform`), output là `ClipEvent { text, source_app: Option<String>, sensitive: bool }`:

| Backend | Platform | Cách làm |
|---|---|---|
| `X11Watcher` | Linux | `x11rb` kết nối tới `DISPLAY` → `XFixesSelectSelectionInput(CLIPBOARD)` → `ConvertSelection(UTF8_STRING)`. Tự reconnect với backoff nếu Xwayland restart |
| `WinWatcher` | Windows | Message-only window + `AddClipboardFormatListener` |
| `MacWatcher` | macOS | Poll `changeCount` mỗi 500ms |

**Pipeline:**

```
event ─► debounce 150ms ─► bỏ qua nếu: sensitive | rỗng | > 1MB | đang Pause | source_app nằm trong blocklist
      ─► hash (blake3) ─► đã có: tăng copy_count + cập nhật last_copied_at
                          chưa có: insert ─► emit clipboard://added
```

Debounce là bắt buộc: spike cho thấy một lần copy có thể sinh ra khoảng 13 event.

**Dấu hiệu "sensitive"** (password manager đánh dấu):

| Platform | Dấu hiệu |
|---|---|
| Linux | Target `x-kde-passwordManagerHint` = `secret` |
| macOS | `org.nspasteboard.ConcealedType` |
| Windows | Format `ExcludeClipboardContentFromMonitorProcessing` |

**Schema:**

```sql
CREATE TABLE clip_entries (
  id              INTEGER PRIMARY KEY,
  content         TEXT NOT NULL,
  content_hash    BLOB NOT NULL UNIQUE,
  kind            TEXT NOT NULL DEFAULT 'text',   -- 'image','files' ở Phase 2/3
  source_app      TEXT,
  pinned          INTEGER NOT NULL DEFAULT 0,
  first_copied_at INTEGER NOT NULL,
  last_copied_at  INTEGER NOT NULL,
  copy_count      INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX clip_recent ON clip_entries (pinned DESC, last_copied_at DESC);
CREATE VIRTUAL TABLE clip_fts USING fts5(content, content='clip_entries', content_rowid='id', tokenize='trigram');
```

- Dùng tokenizer `trigram` để tìm được chuỗi con như `8000`, `pods`, hay một đoạn URL. Query ngắn hơn 3 ký tự thì fallback sang `LIKE` trên 500 entry gần nhất.
- **Retention:** giữ tối đa 1000 entry không pin và tối đa 30 ngày (cấu hình được). Dọn lúc khởi động và mỗi giờ. Entry đã pin không bị xoá.

**UX popup:**
- Ô search được focus sẵn, danh sách 50 entry gần nhất.
- `↑/↓` để chọn, `Enter` để ghi vào clipboard rồi ẩn popup. Người dùng tự paste vì Wayland không cho giả lập phím.
- `Ctrl+P` pin, `Delete` xoá.
- Khi app tự ghi vào clipboard, watcher sẽ bắt lại event đó. Hash trùng nên chỉ bump entry cũ, không sinh bản ghi mới.

**Commands:**
- `clip_list(limit)`
- `clip_search(query, limit)`
- `clip_copy(id)`
- `clip_pin(id, bool)`
- `clip_delete(id)`
- `clip_clear(keep_pinned)`
- `clip_set_paused(bool)`

### 5.3 Port Manager

```rust
struct PortEntry {
    proto: Proto,            // Tcp (MVP), Udp (sau)
    addr: IpAddr, port: u16,
    pid: Option<u32>,        // None khi không đủ quyền đọc /proc/<pid>/fd
    process: Option<String>, cmdline: Option<String>,
    user: Option<String>,    // từ uid trong /proc/net/tcp, luôn có trên Linux
    container: Option<String>, // stretch: map docker-proxy → container name
}
```

**Scan Linux:**
1. Parse `/proc/net/tcp` và `/proc/net/tcp6` để lấy các socket ở trạng thái `LISTEN` (0A). Cách này **luôn thấy đủ port**, kể cả của user khác.
2. Map inode → pid bằng cách quét `/proc/*/fd`, chỉ ở những process đọc được.
3. Không map được thì UI hiện `postgres (uid) · PID ẩn — cần quyền root`.

**Scan Windows/macOS:** dùng crate `listeners`.

**Hành vi:**
- Scan khi mở tab, tự refresh mỗi 2 giây khi tab đang hiển thị, dừng khi ẩn.
- **Search:** chuỗi toàn số thì khớp theo port prefix; có chữ thì tìm trong process name hoặc cmdline.

**Actions:**

| Action | Cách làm |
|---|---|
| Kill | Có hộp xác nhận → gửi SIGTERM → sau 3s nếu process vẫn sống thì đề nghị SIGKILL. Windows dùng `TerminateProcess` |
| Open | `http://localhost:<port>` qua `opener` |
| Copy PID | Ghi PID vào clipboard |
| Copy command | Ghi cmdline vào clipboard |

**Stretch:** process là `docker-proxy` và user thuộc group docker thì gọi `GET /containers/json` qua `/var/run/docker.sock` để hiện tên container.

**Commands:** `ports_scan()`, `ports_kill(pid, force)`.

## 6. Sync — Quick Notes (M5)

### 6.1 Mô hình

**Serverless, bring-your-own-bucket:** client gọi thẳng S3 API (Cloudflare R2 khuyến nghị vì không tính phí egress; S3/MinIO cũng chạy được).
- Mọi object đều được mã hoá trên client. Bucket chỉ thấy tên object, kích thước và thời điểm ghi.
- Mỗi device **chỉ append vào prefix của chính nó**, nên ở tầng storage không bao giờ có hai device cùng ghi một object. Conflict được giải quyết ở tầng dữ liệu (§6.4).

```
s3://<bucket>/quickdesk/v1/
├── keyring.json                      # DEK đã được wrap (không chứa secret dạng rõ)
├── devices/<device_id>.bin           # metadata device đã mã hoá (tên máy, last_seq)
├── log/<device_id>/<seq:020>.bin     # batch op bất biến, đã mã hoá
└── snapshots/<hlc>.bin               # state đã compact, đã mã hoá
```

### 6.2 Crypto

- **DEK:** 32 byte ngẫu nhiên, sinh một lần ở device đầu tiên.
- **KEK:** `Argon2id(passphrase, salt, m=64MiB, t=3, p=1)`.
- `keyring.json` chứa `{salt, params, wrapped_dek_by_passphrase, wrapped_dek_by_recovery_key}`.
- **Recovery key:** 32 byte ngẫu nhiên, chỉ hiển thị **một lần** khi setup (dạng base32, chia nhóm). Quên passphrase thì dùng recovery key để khôi phục.
- **Mỗi object:** `XChaCha20-Poly1305(DEK, nonce=random 24B, AAD=object key)`. Đưa object key vào AAD để chống tráo blob giữa các path.
- **Format object:** `magic "QD1" | version u8 | nonce 24B | ciphertext`. Plaintext là JSON, **không nén** vì note nhỏ; byte version để dành cho việc thêm nén về sau.
- DEK sau khi mở khoá được cache trong OS keyring, nên không phải nhập passphrase mỗi lần khởi động. S3 credentials cũng lưu trong keyring, không bao giờ nằm trong SQLite.

### 6.3 Push / Pull

**Op format:**

```json
{ "id": "...", "hlc": "...", "base_hlc": "...", "body": "...", "pinned": false, "deleted_at": null, "created_at": 0 }
```

Mỗi op là toàn bộ record (full-state upsert, không phải diff).

**Push:**
1. Gom các note `dirty=1` thành một batch.
2. PUT `log/<me>/<seq+1>.bin` kèm `If-None-Match: *`. Nếu PUT báo đã tồn tại tức là có device bị clone trùng `device_id`: sinh `device_id` mới rồi retry.
3. Thành công thì set `dirty=0` và cập nhật `devices/<me>.bin`.

**Pull:**
- Với mỗi device khác (đọc từ `devices/`), gọi `list(prefix=log/<dev>/, start_after=<cursor>)`, tải các batch mới và apply theo thứ tự HLC.
- Cursor của từng device lưu trong bảng local `sync_cursors(device_id, last_key)`.

**Lịch chạy:**
- Push ngay sau khi lưu note (debounce 2s).
- Pull khi mở popup hoặc cửa sổ main, và mỗi 60s khi đang online.
- Mất mạng thì retry với backoff. App vẫn hoạt động đầy đủ khi offline.

**Chi phí R2 ước tính:** 3 device, mỗi device poll 60s → khoảng 130k request list/tháng, vẫn nằm trong free tier (1M Class A/tháng).

### 6.4 Conflict

> **Đã sửa khi triển khai (M2):** bản trước dựa vào cờ `dirty` để phát hiện sửa đồng thời. Cách đó **mất dữ liệu** khi hai máy cùng sửa rồi cùng push trước khi pull: cả hai bản đều đã sạch, nên bản sau fast-forward đè lên bản trước mà không tạo bản conflict. Luật mới dựa trên **dòng phiên bản**.

**Quy ước cho `base_hlc`** (phiên bản đã publish mà một bản được dẫn xuất từ đó):
- Sửa một note đang sạch: `base_hlc ← hlc` cũ, `hlc ←` HLC mới, `dirty ← 1`.
- Sửa tiếp một note đang dirty: giữ nguyên `base_hlc`.
- Nhận bản remote: lưu nguyên `base_hlc` của remote.

Bản B **kế thừa** bản A khi `B.base_hlc >= A.hlc`, nghĩa là B được tạo ra sau khi đã thấy A. Khi nhận `remote` cho note `id`, và `local` là bản hiện tại:

| Trường hợp | Xử lý |
|---|---|
| Chưa có local | Insert |
| `remote.hlc == local.hlc` | Bỏ qua |
| `remote.hlc > local.hlc` và remote kế thừa local | Fast-forward |
| `remote.hlc < local.hlc` và local kế thừa remote | Bỏ qua |
| Còn lại | **Đồng thời** → xem các dòng dưới |
| — Nội dung giống nhau | Merge im lặng: lấy bản HLC lớn hơn |
| — Một bên xoá, một bên sửa | **Bản sửa thắng.** Nếu bản sửa có HLC nhỏ hơn thì được publish lại với HLC mới lớn hơn cả hai bản, để mọi máy fast-forward theo |
| — Cả hai đều xoá | Lấy bản HLC lớn hơn |
| — Chỉ khác `pinned` | Lấy bản HLC lớn hơn |
| — `body` khác nhau | **Giữ cả hai:** bản HLC lớn hơn giữ `id`; bản thua thành note `id = UUIDv5(id ‖ loser.hlc)`, `hlc = loser.hlc`, `conflict_of = id` |

Bản conflict có id **và** hlc tất định, nên mọi máy đều tạo ra đúng một bản conflict giống hệt nhau. Khi máy giữ bản thắng là local, nó cập nhật `base_hlc = remote.hlc` và push lại để các máy khác biết bản đó đã "thấy" remote.

**Test:**
- `qd-notes/src/sync.rs`: các kịch bản cố định, cộng **300 lịch sử ngẫu nhiên** trên 3 máy (tạo/sửa/xoá/khôi phục/pin, giao op từng phần theo thứ tự ngẫu nhiên). Mọi máy đều hội tụ về cùng trạng thái hiển thị, kể cả `pinned`.
- `qd-sync/tests/engine.rs`: engine chạy đầu-cuối qua store in-memory (offline, sai key, khôi phục DB, reset seq, compaction, máy mới tham gia sau compaction).
- `qd-sync/tests/s3_live.rs`: chạy với S3 thật (đã kiểm với RustFS ở local).

### 6.5 Compaction

- Khi tổng số batch vượt 500, device bất kỳ ghi một `snapshots/<hlc>.bin` (toàn bộ state, kèm vector `{device_id: last_seq}`).
- Batch cũ hơn snapshot **và** cũ hơn 30 ngày thì xoá.
- Device mới (hoặc offline quá 30 ngày) bootstrap từ snapshot mới nhất rồi đọc log phía sau. Các thay đổi chưa push của device đó vẫn đi qua luật conflict ở §6.4 như bình thường.

### 6.6 Setup UX

Settings → Sync có các bước:
1. Nhập endpoint, bucket, access key và secret (R2 API token chỉ cấp quyền trên đúng 1 bucket).
2. Test kết nối.
3. Nếu bucket chưa có `keyring.json`: đặt passphrase → app hiện recovery key → người dùng xác nhận đã lưu.
4. Nếu đã có: nhập passphrase (hoặc recovery key) để mở khoá DEK.

Trạng thái sync hiển thị ở footer của main: `Synced · 2 phút trước` / `Offline` / `Lỗi: …`.

### 6.7 v2 (tuỳ chọn): Cloudflare Worker relay

Dùng **cùng định dạng object**. Worker giữ R2 binding (client không còn giữ S3 key, chỉ dùng device token). Thêm Durable Object để đẩy thông báo "có batch mới" qua WebSocket, giúp sync gần realtime và bỏ được việc poll. Client chỉ cần thêm một implementation `BlobTransport` mới.

```rust
trait BlobTransport {
    async fn put_if_absent(&self, key: &str, bytes: Bytes) -> Result<PutOutcome>;
    async fn get(&self, key: &str) -> Result<Option<Bytes>>;
    async fn list_after(&self, prefix: &str, after: Option<&str>) -> Result<Vec<String>>;
    async fn delete(&self, key: &str) -> Result<()>;
}
// impl: S3Transport (v1), WorkerTransport (v2), MemoryTransport (tests)
```

## 7. Milestones

| # | Nội dung | Tiêu chí hoàn thành |
|---|---|---|
| **M1** | Cài Rust + deps của Tauri. Workspace, `qd-core` (db, migrations, settings, HLC), tray, single-instance + CLI `toggle`, hotkey (`GnomeKeybindingBackend` + `PluginBackend`), các window tạo sẵn, logging | Trên máy này: `Super+Shift+N` mở popup WebKit **có focus** 10/10 lần từ nhiều app khác nhau (đóng các mục "unverified" của spike); latency < 150ms; tắt app thì shortcut được dọn sạch |
| **M2** | Quick Notes local | Ghi note trong < 2s kể từ lúc bấm hotkey; tìm kiếm không phân biệt dấu; sửa/pin/xoá + Undo; unit test repository |
| **M3** | Port Manager | Thấy đủ port LISTEN kể cả của user khác (PID ẩn có ghi chú rõ ràng); kill/open/copy; search theo port và theo tên |
| **M4** | Clipboard (text) | Bắt được copy từ terminal, browser, VS Code, app GNOME khi app đang ẩn; dedup; trigram search; pin; retention; pause; bỏ qua sensitive (test với KeePassXC) |
| **M5** | Sync Notes qua R2 | Test mô phỏng hội tụ đều pass; 2 máy thật sửa cùng note khi offline → online không mất dữ liệu; restore bằng recovery key |
| Sau | Clipboard image (Phase 2) và files (Phase 3), `PortalBackend`, Windows/macOS packaging, Worker relay, command palette, modules mới | |

## 8. Rủi ro còn mở

| Rủi ro | Mức | Giảm thiểu |
|---|---|---|
| Popup WebKitGTK có thể không giữ được hành vi focus như cửa sổ GTK thường | Trung bình | Kiểm tra ngay đầu M1. Nếu fail: dùng `xdg-activation` trong lúc trigger, hoặc giữ popup đã map nhưng ẩn bằng opacity |
| Wayland không cho client tự đặt vị trí cửa sổ | Thấp | Chấp nhận vị trí do Mutter chọn (thường là giữa màn hình) |
| GNOME bỏ Xwayland hoặc chuyển sang chạy theo nhu cầu | Thấp–TB | Watcher tự reconnect; về lâu dài bổ sung backend `ext-data-control` hoặc GNOME Shell extension |
| Ghi vào settings GNOME của user | Thấp | Chỉ ghi các path `quickdesk-*`, merge thay vì ghi đè, có nút gỡ trong Settings |
| Metadata trên bucket (số device, số batch, thời điểm ghi) bị lộ | Thấp | Chấp nhận với tool cá nhân; ghi rõ trong tài liệu |
| Quên passphrase và mất recovery key | — | Không khôi phục được dữ liệu sync. Local DB vẫn còn nguyên, có thể re-init sync |

## 9. Trạng thái triển khai (2026-10-08)

M1 → M5 đã xong. Những chỗ khác so với spec ban đầu:

| Mục | Spec | Thực tế | Lý do |
|---|---|---|---|
| Luật conflict | Dựa vào `dirty` | Dựa vào dòng phiên bản (§6.4) | Bản cũ mất dữ liệu khi cả hai máy push trước khi pull |
| Nén object | zstd | Không nén | Note nhỏ; tránh thêm dependency C |
| Binding IPC | `tauri-specta` | Viết tay | tauri-specta còn RC |
| Frontend | Vite mới nhất | Vite 5 | Máy dev đang dùng Node 18 |
| DB | 1 writer + pool đọc | 1 connection + Mutex | Đủ nhanh với WAL; thêm pool khi cần |
| Port Manager | Docker là stretch | **Đã có:** map port → container qua `/var/run/docker.sock`, kèm nút Stop container | Trên máy dev, phần lớn port là của Docker |
| Clipboard Windows/macOS | Listener native | Fallback poll 500ms qua `arboard`, **chưa lọc được nội dung sensitive** | Chưa có máy Windows/macOS để kiểm thử |
| Hotkey | `Super+Shift+N/V/P`, Notes mở popup | `Super+Alt+N/V/P`; Notes mở màn quản lý; popup ghi nhanh là tuỳ chọn | Theo phản hồi người dùng, cộng với vấn đề của Unikey |
| Clipboard: chọn mục | Chỉ copy (Wayland không cho giả lập phím) | **Auto-paste** qua xdg-desktop-portal RemoteDesktop (chỉ quyền bàn phím, restore token nên chỉ hỏi 1 lần, session đóng sau 45s rảnh), gõ Shift+Insert sau khi ghi CLIPBOARD + PRIMARY; Ctrl+Enter = chỉ copy | Người dùng cần paste trực tiếp như Win+V |
| Bucket layout | `quickdesk/v1/…` | `<prefix>/v1/…`, prefix cấu hình được (mặc định `quickdesk`) | Cho phép nhiều app dùng chung một bucket |
| Pull | Chỉ đọc log của máy khác | Đọc cả log của chính mình (từ cursor) | Khôi phục được thay đổi của mình khi DB local bị restore từ bản cũ |
| Build Linux | — | Cần thêm `libdbus-1-dev` | Dùng cho Secret Service (OS keyring) |

Đã có thêm: clipboard **ảnh** (lưu file + thumbnail, giới hạn 10 MB/ảnh, 200 MB tổng) và **file** (chỉ lưu đường dẫn), dán lại bằng Ctrl+V; autostart; gói .deb.

Chưa làm: `PortalBackend` (GNOME ≥ 48 / KDE), Worker relay (§6.7), ảnh/file trên Windows/macOS, đổi passphrase từ UI (API đã có: `Keyring::change_passphrase`).
