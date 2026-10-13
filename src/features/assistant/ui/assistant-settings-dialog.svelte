<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import {
  Banner,
  Button,
  CheckField,
  Dialog,
  SectionHeading,
  SelectField,
  TextField,
} from '@/shared/ui';
import {
  getPolicy,
  listProviders,
  mcpConnection,
  type McpConnection,
  type Policy,
  type Provider,
  type ProviderInput,
  type ProviderKind,
  removeProvider,
  saveProvider,
  setPolicy,
  testProvider,
} from '../api/assistant-api';

interface Props {
  onclose: () => void;
}

const { onclose }: Props = $props();

let providers = $state<Provider[]>([]);
let policy = $state<Policy>({ cloud_allowed: false, mcp_allowed: false });
let connection = $state<McpConnection | null>(null);
let failure = $state<string | null>(null);
let result = $state<string | null>(null);
let draft = $state<ProviderInput | null>(null);
let busy = $state(false);

async function load() {
  try {
    [providers, policy] = await Promise.all([listProviders(), getPolicy()]);
    connection = await mcpConnection();
  } catch (error) {
    failure = toCommandError(error).message;
  }
}

$effect(() => {
  void load();
});

function begin(provider?: Provider) {
  draft = provider
    ? {
        id: provider.id,
        name: provider.name,
        kind: provider.kind,
        base_url: provider.base_url,
        model: provider.model,
        api_key: null,
      }
    : {
        id: null,
        name: '',
        kind: 'anthropic',
        base_url: 'https://api.anthropic.com',
        model: '',
        api_key: null,
      };
}

async function save() {
  if (!draft) return;
  busy = true;
  failure = null;
  try {
    await saveProvider({ ...draft, api_key: draft.api_key?.trim() ? draft.api_key : null });
    draft = null;
    await load();
  } catch (error) {
    failure = toCommandError(error).message;
  } finally {
    busy = false;
  }
}

async function test(provider: Provider) {
  busy = true;
  failure = null;
  result = null;
  try {
    result = t('assistant.tested', { what: await testProvider(provider.id) });
  } catch (error) {
    failure = toCommandError(error).message;
  } finally {
    busy = false;
  }
}

async function remove(provider: Provider) {
  try {
    await removeProvider(provider.id);
    await load();
  } catch (error) {
    failure = toCommandError(error).message;
  }
}

async function update(change: Partial<Policy>) {
  try {
    policy = await setPolicy({ ...policy, ...change });
  } catch (error) {
    failure = toCommandError(error).message;
  }
}

/** Typing a different protocol offers its usual address, unless the author already wrote one. */
function kindChanged(kind: ProviderKind) {
  if (!draft) return;
  const defaults: Record<ProviderKind, string> = {
    anthropic: 'https://api.anthropic.com',
    open_ai_compatible: 'https://api.openai.com/v1',
  };
  if (Object.values(defaults).includes(draft.base_url)) draft.base_url = defaults[kind];
}
</script>

<Dialog title={t('assistant.settingsTitle')} width="lg" {onclose}>
  {#if failure}<Banner kind="error">{failure}</Banner>{/if}
  {#if result}<Banner kind="info">{result}</Banner>{/if}

  <section class="flex flex-col gap-3">
    <SectionHeading title={t('assistant.providers')}>
      {#snippet actions()}
        <Button size="sm" onclick={() => begin()}>{t('assistant.addProvider')}</Button>
      {/snippet}
    </SectionHeading>
    <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('assistant.providersHint')}</p>

    {#if draft}
      <div class="rounded border border-[var(--color-border-strong)] p-3 flex flex-col gap-3">
        <TextField label={t('assistant.name')} bind:value={draft.name} focus />
        <SelectField
          label={t('assistant.kind')}
          bind:value={draft.kind as string}
          onchange={() => draft && kindChanged(draft.kind)}
          options={[
            { value: 'anthropic', label: t('assistant.kind_anthropic') },
            { value: 'open_ai_compatible', label: t('assistant.kind_openai_compatible') },
          ]}
        />
        <TextField label={t('assistant.baseUrl')} bind:value={draft.base_url} hint={t('assistant.baseUrlHint')} mono />
        <TextField label={t('assistant.model')} bind:value={draft.model} mono />
        <TextField
          label={t('assistant.apiKey')}
          type="password"
          hint={t('assistant.apiKeyHint')}
          bind:value={
            () => draft?.api_key ?? '',
            (value) => {
              if (draft) draft.api_key = value;
            }
          }
        />
        <div class="flex gap-2">
          <Button variant="primary" size="sm" disabled={busy || draft.name.trim() === '' || draft.model.trim() === ''} onclick={save}>
            {t('common.save')}
          </Button>
          <Button variant="ghost" size="sm" onclick={() => (draft = null)}>{t('common.cancel')}</Button>
        </div>
      </div>
    {/if}

    {#if providers.length === 0 && !draft}
      <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('assistant.noProviders')}</p>
    {:else}
      <ul class="m-0 p-0 list-none flex flex-col divide-y divide-[var(--color-border-subtle)] border-y border-[var(--color-border-subtle)]">
        {#each providers as provider (provider.id)}
          <li class="flex items-center gap-3 py-2">
            <span class="flex-1 min-w-0 flex flex-col">
              <span class="text-xs truncate">{provider.name} · <span class="font-mono">{provider.model}</span></span>
              <span class="text-[11px] text-[var(--color-text-muted)] truncate">
                {provider.is_local ? t('assistant.local') : t('assistant.cloud')} · {provider.base_url} · {provider.has_key ? t('assistant.keySaved') : t('assistant.noKey')}
              </span>
            </span>
            <Button size="sm" variant="ghost" disabled={busy} onclick={() => test(provider)}>{t('assistant.test')}</Button>
            <Button size="sm" variant="ghost" onclick={() => begin(provider)}>{t('common.edit')}</Button>
            <Button size="sm" variant="ghost" onclick={() => remove(provider)}>{t('assistant.removeProvider')}</Button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="flex flex-col gap-3">
    <SectionHeading title={t('assistant.policyTitle')} />
    <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('assistant.policyHint')}</p>
    <CheckField
      label={t('assistant.cloudAllowed')}
      hint={t('assistant.cloudAllowedHint')}
      bind:checked={() => policy.cloud_allowed, (value) => void update({ cloud_allowed: value })}
    />
    <CheckField
      label={t('assistant.mcpAllowed')}
      hint={t('assistant.mcpAllowedHint')}
      bind:checked={() => policy.mcp_allowed, (value) => void update({ mcp_allowed: value })}
    />
  </section>

  {#if connection}
    <section class="flex flex-col gap-2">
      <SectionHeading title={t('assistant.mcpTitle')} />
      <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('assistant.mcpHint')}</p>
      {#if !connection.installed}
        <Banner kind="warning">{t('assistant.mcpMissing')}</Banner>
      {/if}
      <pre class="m-0 p-3 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] text-[11px] font-mono overflow-auto select-text">{connection.configuration}</pre>
    </section>
  {/if}

  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('data.close')}</Button>
  {/snippet}
</Dialog>
