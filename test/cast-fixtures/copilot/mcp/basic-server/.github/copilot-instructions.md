# ZManager AI Coding Instructions

A dual-pane Windows file manager in Rust with TUI (Ratatui) and GUI (Tauri v2 + React 19) frontends.

## Architecture

```
crates/
├── zmanager-core/         # Platform-agnostic core (domain types, business logic)
├── zmanager-transfer-win/ # Windows transfer engine (CopyFileExW, clipboard)
├── zmanager-tui/          # Terminal UI (Ratatui + Crossterm)
└── zmanager-tauri/        # GUI backend + React frontend in gui/ subfolder
```

**Core principle**: All business logic lives in `zmanager-core`. Frontends are thin layers. Transfer engine is Windows-specific for cross-platform future-proofing.

## Critical: Rust ↔ TypeScript Type Alignment

All Rust enums use `#[serde(rename_all = "snake_case")]`. TypeScript **MUST** match with lowercase strings:

```rust
// Rust: crates/zmanager-core/src/entry.rs
#[serde(rename_all = "snake_case")]
pub enum EntryKind { File, Directory, Symlink, Junction }
```

```typescript
// TypeScript: crates/zmanager-tauri/gui/src/types/index.ts
export type EntryKind = "file" | "directory" | "symlink" | "junction"; // ✅
export type EntryKind = "File" | "Directory";  // ❌ NEVER - breaks IPC
```

## Development Commands

```bash
# Rust
cargo build                                    # All crates
cargo test --workspace                         # 147+ tests
cargo clippy --workspace --all-targets -- -D warnings

# GUI (from crates/zmanager-tauri/gui/)
bun install && bun run dev                     # Vite dev server
bun run check                                  # Biome lint + format
```

## Code Patterns

### Rust Module Structure
```rust
//! Module docstring
use crate::{ZError, ZResult};  // Always use project error types

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]  // Required for IPC types
pub struct Thing { /* fields */ }

#[cfg(test)]
mod tests { /* inline tests */ }
```

### Tauri Commands (`crates/zmanager-tauri/src/commands.rs`)
```rust
#[tauri::command]
pub async fn zmanager_list_dir(path: String) -> IpcResponse<DirListing> {
    match list_directory(&path, None, None) {
        Ok(listing) => IpcResponse::success(listing),
        Err(e) => IpcResponse::failure(e.to_string()),
    }
}
```

Commands return `IpcResponse<T>` with `{ ok: bool, data?, error? }` shape. All commands prefixed `zmanager_*`.

### React Frontend Stack
- **State**: Zustand stores in `gui/src/stores/` (fileSystem, clipboard, favorites, ui)
- **Data fetching**: TanStack Query for async operations
- **Styling**: Tailwind CSS v4, Biome for lint/format
- **Virtualization**: `@tanstack/react-virtual` for large file lists
- **DnD**: `@dnd-kit` for drag-and-drop

### Transfer Engine (`zmanager-transfer-win`)

The transfer engine wraps Windows native `CopyFileExW` for high-performance file operations:

```rust
use zmanager_transfer_win::{copy_file_with_progress, CopyProgress, ConflictPolicy};

// Single file copy with progress callback
copy_file_with_progress(
    &source_path,
    &dest_path,
    Some(Box::new(|progress: CopyProgress| {
        println!("{}% - {} bytes/sec", progress.percentage_int(), progress.speed_bps);
    })),
    cancel_token.clone(),
).await?;
```

**Key types**:
- `CopyProgress`: Contains `bytes_copied`, `total_bytes`, `speed_bps`, `eta_seconds`
- `Conflict`: Detected when destination exists (has `source_is_newer()` helper)
- `ConflictPolicy`: `Overwrite`, `Skip`, `Rename`, `Ask`
- `TransferPlan`: Pre-computed transfer with `TransferPlanBuilder`

**Clipboard integration** via `CF_HDROP`:
```rust
use zmanager_transfer_win::{read_files_from_clipboard, write_files_to_clipboard, DropEffect};

write_files_to_clipboard(&paths, DropEffect::Copy)?;  // or DropEffect::Move
let content = read_files_from_clipboard()?;  // Returns ClipboardContent
```

### Zustand Store Pattern

Stores follow a consistent structure in `gui/src/stores/`:

```typescript
// gui/src/stores/example.store.ts
import { create } from "zustand";

interface ExampleState {
  data: SomeType | null;
  isLoading: boolean;
  error: string | null;
  // Actions
  loadData: () => Promise<void>;
  reset: () => void;
}

export const useExampleStore = create<ExampleState>((set, get) => ({
  data: null,
  isLoading: false,
  error: null,

  loadData: async () => {
    set({ isLoading: true, error: null });
    const result = await invoke<IpcResponse<SomeType>>("zmanager_get_data");
    if (result.ok) {
      set({ data: result.data, isLoading: false });
    } else {
      set({ error: result.error, isLoading: false });
    }
  },

  reset: () => set({ data: null, error: null }),
}));
```

**Store conventions**:
- Export via `gui/src/stores/index.ts` for clean imports
- Type-export interfaces: `export type { PaneId, PaneState }`
- Actions handle IPC errors by setting `error` state

### Virtualized List Pattern

For file lists with 50k+ entries, use `@tanstack/react-virtual`:

```tsx
// gui/src/components/VirtualizedFileList.tsx
const ROW_HEIGHT = 28;  // Fixed height for virtualization
const OVERSCAN = 5;     // Extra rows rendered above/below viewport

const virtualizer = useVirtualizer({
  count: entries.length,
  getScrollElement: () => parentRef.current,
  estimateSize: () => ROW_HEIGHT,
  overscan: OVERSCAN,
});

return (
  <div ref={parentRef} style={{ overflow: "auto", height: "100%" }}>
    <div style={{ height: virtualizer.getTotalSize() }}>
      {virtualizer.getVirtualItems().map((virtualRow) => (
        <div
          key={virtualRow.key}
          style={{
            position: "absolute",
            top: virtualRow.start,
            height: ROW_HEIGHT,
          }}
        >
          <FileRow entry={entries[virtualRow.index]} />
        </div>
      ))}
    </div>
  </div>
);
```


### Error Handling
- Rust: Return `ZResult<T>` (alias for `Result<T, ZError>`). Use `thiserror`. Never `.unwrap()` in library code.
- TypeScript: Handle `IpcResponse.ok === false` case on all Tauri invokes.

## Key Files

| Purpose | File |
|---------|------|
| Core API exports | `crates/zmanager-core/src/lib.rs` |
| IPC contract spec | `docs/IPC_Contract.md` |
| Sprint roadmap | `docs/Sprint_Roadmap.md` |
| TypeScript types | `crates/zmanager-tauri/gui/src/types/index.ts` |
| Tauri commands | `crates/zmanager-tauri/src/commands.rs` |

## Common Pitfalls

- **Long paths**: Use `\\?\` prefix for paths ≥240 chars (Windows limitation)
- **Recycle Bin**: Use `SHFileOperationW`, not `std::fs::remove_*`
- **Async Rust**: Use `spawn_blocking` for CPU-intensive work to avoid blocking Tokio
- **Tracing**: Use `tracing::{debug, info, warn, error}`, not the `log` crate
- **TypeScript enums**: Always lowercase to match Rust serde - `"directory"` not `"Directory"`
