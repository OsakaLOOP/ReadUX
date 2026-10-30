import { invoke, isTauri } from '@tauri-apps/api/core';
import { staticReaderRequest } from './staticLibrary';

export async function readerRequest<T>(command: string, payload: Record<string, unknown> = {}): Promise<T> {
  if (!isTauri()) return staticReaderRequest(command, payload) as Promise<T>;
  return invoke<T>(command, payload);
}
