<script lang="ts">
import { EntityPicker } from '@/features/entity';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import type { EntityRef, Temporal } from '@/shared/model';
import { revision } from '@/shared/revision';
import { Banner, Button, SelectField, TemporalField, TextField } from '@/shared/ui';
import {
  type CausalityKind,
  type EventParticipant,
  type EventScale,
  type EventView,
  addEventCausality,
  getEvent,
  removeEventCausality,
  saveEventDetail,
} from '../api/timeline-api';

interface Props {
  entity: EntityRef;
  onopen: (id: string) => void;
}

const { entity, onopen }: Props = $props();

const SCALES: EventScale[] = ['personal', 'local', 'regional', 'national', 'world'];
const KINDS: CausalityKind[] = ['direct', 'contributing', 'enabling', 'prevented', 'triggered'];

let view = $state<EventView | null>(null);
let period = $state<Temporal>({ kind: 'unknown' });
let location = $state('');
let scale = $state('');
let consequences = $state('');
let participants = $state<EventParticipant[]>([]);
let newParticipant = $state('');
let newRole = $state('participant');
let newCause = $state('');
let newCauseKind = $state<string>('direct');
let error = $state<string | null>(null);
let busy = $state(false);

$effect(() => {
  void revision.value;
  let cancelled = false;
  getEvent(entity.id)
    .then((found) => {
      if (cancelled) return;
      view = found;
      period = found.detail?.period ?? { kind: 'unknown' };
      location = found.detail?.location ?? '';
      scale = found.detail?.scale ?? '';
      consequences = found.detail?.consequences ?? '';
      participants = (found.detail?.participants ?? []).map((participant) => ({ ...participant }));
    })
    .catch((failure) => {
      if (!cancelled) error = toCommandError(failure).message;
    });
  return () => {
    cancelled = true;
  };
});

async function save() {
  busy = true;
  error = null;
  try {
    await saveEventDetail({
      entity: entity.id,
      period,
      location: location === '' ? null : location,
      scale: scale === '' ? null : (scale as EventScale),
      consequences,
      participants,
    });
    revision.bump();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

function addParticipant() {
  if (newParticipant === '' || newRole.trim() === '') return;
  participants = [...participants, { entity: newParticipant, role: newRole.trim(), side: null }];
  newParticipant = '';
}

async function addCause() {
  if (newCause === '') return;
  try {
    await addEventCausality({
      cause: newCause,
      effect: entity.id,
      kind: newCauseKind as CausalityKind,
      note: null,
    });
    newCause = '';
    revision.bump();
  } catch (failure) {
    error = toCommandError(failure).message;
  }
}

async function removeCause(id: string) {
  try {
    await removeEventCausality(id);
    revision.bump();
  } catch (failure) {
    error = toCommandError(failure).message;
  }
}

const participantNames = $derived(
  new Map<string, string>(
    (view?.participants ?? []).map((entry): [string, string] => [
      entry.entity.id,
      entry.entity.name,
    ]),
  ),
);
</script>

<section class="flex flex-col gap-4 max-w-2xl">
  {#if error}<Banner kind="error">{error}</Banner>{/if}
  {#if view}
    {#key view}
      <TemporalField label={t('event.when')} bind:value={period} />
    {/key}
    <EntityPicker
      label={t('event.where')}
      bind:value={location}
      filter={{ entity_type: 'place' }}
      selected={view.location}
      placeholder={t('event.noPlace')}
    />
    <SelectField
      label={t('event.scale')}
      bind:value={scale}
      options={SCALES.map((value) => ({ value, label: t(`event.scale_${value}` as 'event.scale_local') }))}
      placeholder={t('event.noScale')}
    />
    <TextField label={t('event.consequences')} bind:value={consequences} multiline rows={3} />

    <div class="flex flex-col gap-2">
      <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
        {t('event.participants')}
      </h3>
      {#each participants as participant, index (index)}
        <div class="flex items-center gap-2 text-xs">
          <button
            type="button"
            class="cursor-pointer bg-transparent border-0 p-0 text-left text-[var(--color-text-primary)] hover:underline"
            onclick={() => onopen(participant.entity)}
          >
            {participantNames.get(participant.entity) ?? participant.entity}
          </button>
          <span class="text-[var(--color-text-muted)] font-mono">{participant.role}</span>
          <Button
            size="sm"
            variant="ghost"
            icon="i-mingcute:close-line"
            title={t('common.remove')}
            onclick={() => (participants = participants.filter((_, position) => position !== index))}
          />
        </div>
      {:else}
        <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('event.noParticipants')}</p>
      {/each}
      <div class="flex items-end gap-2 flex-wrap">
        <div class="w-64">
          <EntityPicker
            label={t('event.addParticipant')}
            bind:value={newParticipant}
            filter={{ role: 'generic' }}
            exclude={participants.map((participant) => participant.entity)}
          />
        </div>
        <div class="w-40"><TextField label={t('event.role')} bind:value={newRole} /></div>
        <Button size="sm" icon="i-mingcute:add-line" onclick={addParticipant}>{t('common.add')}</Button>
      </div>
    </div>

    <div>
      <Button variant="primary" disabled={busy} onclick={save}>{t('common.save')}</Button>
    </div>

    <div class="flex flex-col gap-2 pt-4 border-t border-[var(--color-border-subtle)]">
      <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
        {t('event.causes')}
      </h3>
      {#each view.causes as cause (cause.id)}
        <div class="flex items-center gap-2 text-xs">
          <button type="button" class="cursor-pointer bg-transparent border-0 p-0 hover:underline" onclick={() => onopen(cause.other.id)}>
            {cause.other.name}
          </button>
          <span class="font-mono text-[var(--color-text-muted)]">{t(`event.kind_${cause.kind}` as 'event.kind_direct')}</span>
          <Button size="sm" variant="ghost" icon="i-mingcute:close-line" title={t('common.remove')} onclick={() => removeCause(cause.id)} />
        </div>
      {:else}
        <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('event.noCauses')}</p>
      {/each}
      <div class="flex items-end gap-2 flex-wrap">
        <div class="w-64">
          <EntityPicker
            label={t('event.addCause')}
            bind:value={newCause}
            filter={{ role: 'event' }}
            exclude={[entity.id]}
          />
        </div>
        <div class="w-40">
          <SelectField
            label={t('event.kind')}
            bind:value={newCauseKind}
            options={KINDS.map((value) => ({ value, label: t(`event.kind_${value}` as 'event.kind_direct') }))}
          />
        </div>
        <Button size="sm" icon="i-mingcute:add-line" onclick={addCause}>{t('common.add')}</Button>
      </div>
      {#if view.effects.length > 0}
        <h3 class="m-0 mt-2 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
          {t('event.effects')}
        </h3>
        {#each view.effects as effect (effect.id)}
          <div class="flex items-center gap-2 text-xs">
            <button type="button" class="cursor-pointer bg-transparent border-0 p-0 hover:underline" onclick={() => onopen(effect.other.id)}>
              {effect.other.name}
            </button>
            <span class="font-mono text-[var(--color-text-muted)]">{t(`event.kind_${effect.kind}` as 'event.kind_direct')}</span>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</section>
