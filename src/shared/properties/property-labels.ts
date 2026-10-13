import { tDefault } from '@/shared/i18n';
import type { PropertyDefinition, PropertyGroup } from '@/shared/model';

/** A property's name in the interface language, unless the author renamed it. */
export function definitionLabel(definition: PropertyDefinition): string {
  if (!definition.builtin) return definition.label;
  return tDefault(`propertyNames.${definition.owner.key}_${definition.key}`, definition.label);
}

/** A property group's name in the interface language, unless the author renamed it. */
export function groupLabel(group: PropertyGroup | undefined, key: string): string {
  return group ? tDefault(`groupNames.${group.key}`, group.label) : key;
}
