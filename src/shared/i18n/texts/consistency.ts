import type { TextGroup } from '../define';

/** Consistency checks and their findings. */
export const consistencyTexts = {
  consistency: {
    noMatchTitle: { en: 'No findings match', es: 'Ningún hallazgo coincide' },
    noMatchHint: {
      en: 'Change the severity or the check to see the rest.',
      es: 'Cambia la gravedad o la comprobación para ver el resto.',
    },
    title: { en: 'Consistency checks', es: 'Comprobaciones de consistencia' },
    subtitle: {
      en: 'Deterministic rules over what is recorded. They report; they never change canon.',
      es: 'Reglas deterministas sobre lo registrado. Informan; nunca cambian el canon.',
    },
    rerun: { en: 'Re-run Rules', es: 'Re-ejecutar Reglas' },
    affects: { en: 'Affects', es: 'Afecta' },
    errors: { en: '{count} error(s)', es: '{count} error(es)' },
    warnings: { en: '{count} warning(s)', es: '{count} advertencia(s)' },
    infos: { en: '{count} note(s)', es: '{count} nota(s)' },
    examined: {
      en: 'Checked {entities} entities, {relationships} relationships, {appearances} appearances, {events} events',
      es: 'Revisadas {entities} entidades, {relationships} relaciones, {appearances} apariciones, {events} eventos',
    },
    severity: { en: 'Severity', es: 'Gravedad' },
    severityError: { en: 'Errors', es: 'Errores' },
    severityWarning: { en: 'Warnings', es: 'Advertencias' },
    severityInfo: { en: 'Notes', es: 'Notas' },
    rule: { en: 'Check', es: 'Comprobación' },
    all: { en: 'All', es: 'Todas' },
    cleanTitle: { en: 'Nothing contradicts itself', es: 'Nada se contradice' },
    cleanHint: {
      en: 'Every check passed over what is recorded so far.',
      es: 'Todas las comprobaciones pasaron sobre lo registrado hasta ahora.',
    },
    rule_participant_after_death: {
      en: 'Takes part after dying',
      es: 'Participa después de morir',
    },
    explain_participant_after_death: {
      en: '{subject} happens on day {event_day}, after {related} died on day {death_day}.',
      es: '{subject} ocurre el día {event_day}, después de que {related} muriera el día {death_day}.',
    },
    rule_participant_before_birth: {
      en: 'Takes part before being born',
      es: 'Participa antes de nacer',
    },
    explain_participant_before_birth: {
      en: '{subject} happens on day {event_day}, before {related} was born on day {birth_day}.',
      es: '{subject} ocurre el día {event_day}, antes de que {related} naciera el día {birth_day}.',
    },
    rule_parent_not_older: {
      en: 'Parent not older than child',
      es: 'Progenitor no mayor que el hijo',
    },
    explain_parent_not_older: {
      en: '{subject} was born on day {parent_birth_day}, not before their child {related}, born on day {child_birth_day}.',
      es: '{subject} nació el día {parent_birth_day}, no antes de su hijo {related}, nacido el día {child_birth_day}.',
    },
    rule_effect_before_cause: { en: 'Effect before its cause', es: 'Efecto antes de su causa' },
    explain_effect_before_cause: {
      en: '{subject} starts on day {effect_day}, before its cause {related} on day {cause_day}.',
      es: '{subject} empieza el día {effect_day}, antes de su causa {related} del día {cause_day}.',
    },
    rule_presence_and_absence: {
      en: 'Present and absent at once',
      es: 'Presente y ausente a la vez',
    },
    explain_presence_and_absence: {
      en: '{subject} is recorded both present and confirmed absent in {related}.',
      es: '{subject} figura a la vez presente y con ausencia confirmada en {related}.',
    },
    rule_two_places_at_once: { en: 'In two places at once', es: 'En dos lugares a la vez' },
    explain_two_places_at_once: {
      en: '{subject} takes part in {related}, which overlap in time at different places.',
      es: '{subject} participa en {related}, que coinciden en el tiempo en lugares distintos.',
    },
    rule_canon_on_uncanon: {
      en: 'Canon resting on non-canon',
      es: 'Canon apoyado en lo que no es canon',
    },
    explain_canon_on_uncanon: {
      en: 'A {record} of {subject} rests on {related}, which is {state}.',
      es: 'Un registro ({record}) de {subject} se apoya en {related}, que está {state}.',
    },
    rule_required_property_missing: {
      en: 'Required trait missing',
      es: 'Falta un rasgo obligatorio',
    },
    explain_required_property_missing: {
      en: '{subject} has no value for the required trait "{property}".',
      es: '{subject} no tiene valor para el rasgo obligatorio «{property}».',
    },
    rule_viewpoint_absent: { en: 'Told through someone absent', es: 'Narrada por alguien ausente' },
    explain_viewpoint_absent: {
      en: '{subject} is told through {related}, who is not present in it.',
      es: '{subject} está narrada a través de {related}, que no está presente en ella.',
    },
    rule_office_overfilled: { en: 'More holders than seats', es: 'Más titulares que puestos' },
    explain_office_overfilled: {
      en: '{subject}: the office "{office}" has {seats} seat(s) but {holders} holders at once.',
      es: '{subject}: el cargo «{office}» tiene {seats} puesto(s) pero {holders} titulares a la vez.',
    },
    rule_section_not_placed: {
      en: 'Section outside the structure',
      es: 'Sección fuera de la estructura',
    },
    explain_section_not_placed: {
      en: '{subject} is an editorial unit that is not placed in the structure of the work.',
      es: '{subject} es una unidad editorial que no está colocada en la estructura de la obra.',
    },
    rule_event_undated: { en: 'Event without a date', es: 'Evento sin fecha' },
    explain_event_undated: {
      en: '{subject} has no known day, so it cannot be placed on the timeline.',
      es: '{subject} no tiene un día conocido, así que no se puede situar en la línea de tiempo.',
    },
    rule_no_connections: { en: 'Not connected to anything', es: 'Sin conexiones' },
    explain_no_connections: {
      en: '{subject} has no relationships, appearances or events.',
      es: '{subject} no tiene relaciones, apariciones ni eventos.',
    },
    rule_duplicate_name: { en: 'Duplicate name', es: 'Nombre duplicado' },
    explain_duplicate_name: {
      en: '{subject} shares its name and type with {related}.',
      es: '{subject} comparte nombre y tipo con {related}.',
    },
  },
} satisfies TextGroup;
