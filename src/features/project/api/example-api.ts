import { call } from '@/shared/backend';

/** Mirrors `ExampleLoaded` in `src-tauri/src/example.rs`. */
export interface ExampleLoaded {
  entities: number;
}

/** Fills an empty universe with the example, written in `locale`. */
export function loadExampleUniverse(locale: string): Promise<ExampleLoaded> {
  return call('load_example_universe', { locale });
}
