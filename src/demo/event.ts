// Demo build only: replaces `@tauri-apps/api/event`.
import { on } from "./bus";

export function listen<T>(event: string, fn: (e: { event: string; payload: T }) => void): Promise<() => void> {
  return Promise.resolve(on(event, fn as (e: { event: string; payload: unknown }) => void));
}
