import { expect, test } from 'bun:test';
import type { MatrixCell } from '@/features/matrix/api/matrix-api';
import {
  cellKey,
  indexCells,
  kindLook,
  presenceByColumn,
  presenceByRow,
} from '@/features/matrix/state/matrix-model';

function cell(entity: string, context: string, kind: string, isPresence = true): MatrixCell {
  return {
    entity,
    context,
    appearance: `${entity}-${context}-${kind}`,
    kind,
    is_presence: isPresence,
    viewpoint: null,
    canon_state: 'established',
    position: 0,
  };
}

const cells = [
  cell('ayla', 's1', 'physical_presence'),
  cell('ayla', 's2', 'dialogue'),
  cell('orin', 's2', 'confirmed_absence', false),
  cell('orin', 's3', 'indirect_mention'),
];

test('cells are found where their row meets their column', () => {
  const index = indexCells(cells);

  expect(index.get(cellKey('ayla', 's1'))?.length).toBe(1);
  expect(index.get(cellKey('ayla', 's3'))).toBeUndefined();
});

test('totals count presence and leave confirmed absence out', () => {
  expect(presenceByRow(cells).get('ayla')).toBe(2);
  expect(presenceByRow(cells).get('orin')).toBe(1);
  expect(presenceByColumn(cells).get('s2')).toBe(1);
});

test('a project-defined kind still gets a look, by whether it is a presence', () => {
  expect(kindLook('custom_vision', true).icon).toBe('i-mingcute:user-3-line');
  expect(kindLook('custom_void', false).icon).toBe('i-mingcute:close-line');
});
