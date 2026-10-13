import { call } from '@/shared/backend';
import type { EntityId } from '@/shared/model';

export type ProviderKind = 'anthropic' | 'open_ai_compatible';

/** A configured provider. Never carries its key. Mirrors `ProviderView` in `src-tauri/src/assistant`. */
export interface Provider {
  id: string;
  name: string;
  kind: ProviderKind;
  base_url: string;
  model: string;
  has_key: boolean;
  /** It runs on this computer, so nothing leaves it. */
  is_local: boolean;
}

/** A provider to add or change. The key is only ever sent, never read back. */
export interface ProviderInput {
  id: string | null;
  name: string;
  kind: ProviderKind;
  base_url: string;
  model: string;
  api_key: string | null;
}

export type TaskKind = 'describe' | 'relate' | 'analyze' | 'generate';

/** What to ask, of which provider, about what. Mirrors `TaskRequest`. */
export interface TaskRequest {
  kind: TaskKind;
  entity: EntityId | null;
  instruction: string;
  provider: string;
}

/** Exactly what a task would send, and where. Mirrors `RequestPreview`. */
export interface RequestPreview {
  text: string;
  characters: number;
  entities: number;
  truncated: boolean;
  provider: string;
  model: string;
  destination: string;
  is_local: boolean;
  /** Why it cannot be run, when it cannot. */
  blocked: string | null;
}

/** Mirrors `TaskOutcome`. */
export interface TaskOutcome {
  proposal: string;
  dropped: number;
}

/** What assistants may reach in the project. Mirrors `PolicyView`. */
export interface Policy {
  cloud_allowed: boolean;
  mcp_allowed: boolean;
}

/** How a client reaches the project. Mirrors `Connection` in `src-tauri/src/mcp.rs`. */
export interface McpConnection {
  command: string;
  arguments: string[];
  installed: boolean;
  configuration: string;
}

export function listProviders(): Promise<Provider[]> {
  return call('list_providers');
}

export function saveProvider(provider: ProviderInput): Promise<Provider> {
  return call('save_provider', { provider });
}

export function removeProvider(id: string): Promise<void> {
  return call('remove_provider', { id });
}

export function testProvider(id: string): Promise<string> {
  return call('test_provider', { id });
}

export function previewTask(task: TaskRequest): Promise<RequestPreview> {
  return call('preview_task', { task });
}

export function runTask(task: TaskRequest): Promise<TaskOutcome> {
  return call('run_task', { task });
}

export function getPolicy(): Promise<Policy> {
  return call('get_ai_policy');
}

export function setPolicy(policy: Policy): Promise<Policy> {
  return call('set_ai_policy', {
    cloudAllowed: policy.cloud_allowed,
    mcpAllowed: policy.mcp_allowed,
  });
}

export function mcpConnection(): Promise<McpConnection> {
  return call('mcp_connection');
}
