import { beforeEach, expect, test } from 'bun:test';
import { navigation } from '@/features/navigation/state/navigation.svelte';

beforeEach(() => navigation.clear());

test('opening entities builds a trail that can be walked back and forward', () => {
  navigation.open('ayla');
  navigation.open('orin');
  navigation.open('ashmere');

  expect(navigation.current).toBe('ashmere');
  navigation.back();
  expect(navigation.current).toBe('orin');
  navigation.back();
  expect(navigation.current).toBe('ayla');
  expect(navigation.canGoBack).toBe(false);
  navigation.forward();
  expect(navigation.current).toBe('orin');
});

test('opening something new after going back drops the forward trail', () => {
  navigation.open('a');
  navigation.open('b');
  navigation.back();
  navigation.open('c');

  expect(navigation.canGoForward).toBe(false);
  navigation.back();
  expect(navigation.current).toBe('a');
});

test('opening the entity already shown does not grow the trail', () => {
  navigation.open('a');
  navigation.open('a');

  expect(navigation.canGoBack).toBe(false);
});

test('a removed entity is forgotten so back never lands on it', () => {
  navigation.open('a');
  navigation.open('b');
  navigation.open('c');

  navigation.forget('b');
  navigation.back();

  expect(navigation.current).toBe('a');
});

test('forgetting the entity on screen moves to its neighbour', () => {
  navigation.open('a');
  navigation.open('b');

  navigation.forget('b');

  expect(navigation.current).toBe('a');
});

test('the same entity can be followed through another view and back', () => {
  navigation.open('ayla');
  navigation.show('graph');
  navigation.show('timeline');

  expect(navigation.current).toBe('ayla');
  expect(navigation.view).toBe('timeline');
  navigation.back();
  expect(navigation.view).toBe('graph');
  navigation.back();
  expect(navigation.view).toBe('universe');
  expect(navigation.current).toBe('ayla');
});

test('opening an entity keeps the current view unless another is asked for', () => {
  navigation.open('ayla');
  navigation.show('matrix');
  navigation.open('orin');
  navigation.open('mara', 'universe');

  expect(navigation.view).toBe('universe');
  navigation.back();
  expect(navigation.current).toBe('orin');
  expect(navigation.view).toBe('matrix');
});

test('a view can be shown before any entity is chosen', () => {
  navigation.show('consistency');

  expect(navigation.current).toBeNull();
  expect(navigation.view).toBe('consistency');
});

test('showing the view already on screen does not grow the trail', () => {
  navigation.open('ayla');
  navigation.show('universe');

  expect(navigation.canGoBack).toBe(false);
});

test('closing the page keeps the view and can be walked back', () => {
  navigation.clear();
  navigation.open('a', 'universe');

  navigation.close();

  expect(navigation.current).toBeNull();
  expect(navigation.view).toBe('universe');
  navigation.back();
  expect(navigation.current).toBe('a');
});
