import { expect, test } from 'bun:test';
import { joinPath, projectFolderName } from '@/features/project/state/project-path';

test('a project folder name is safe and ends in .ontara', () => {
  expect(projectFolderName('The Ashen Empire')).toBe('The Ashen Empire.ontara');
  expect(projectFolderName('  a/b:c  ')).toBe('a-b-c.ontara');
  expect(projectFolderName('   ')).toBe('Untitled.ontara');
  expect(projectFolderName('..hidden')).toBe('hidden.ontara');
});

test('paths are joined with the separator the folder already uses', () => {
  expect(joinPath('/home/ana/Universes', 'Aurea.ontara')).toBe('/home/ana/Universes/Aurea.ontara');
  expect(joinPath('/home/ana/Universes/', 'Aurea.ontara')).toBe('/home/ana/Universes/Aurea.ontara');
  expect(joinPath('C:\\Users\\Ana', 'Aurea.ontara')).toBe('C:\\Users\\Ana\\Aurea.ontara');
});
