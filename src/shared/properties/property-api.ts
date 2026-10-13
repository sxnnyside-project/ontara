import { call } from '@/shared/backend';
import type { PropertyValue, PropertyValueId, PropertyValueInput } from '@/shared/model';

/** Adds a value, or updates it when the input carries an identifier. */
export function savePropertyValue(value: PropertyValueInput): Promise<PropertyValue> {
  return call('save_property_value', { value });
}

export function removePropertyValue(id: PropertyValueId): Promise<void> {
  return call('remove_property_value', { id });
}
