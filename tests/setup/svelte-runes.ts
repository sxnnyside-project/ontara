import { plugin } from 'bun';
import { compileModule } from 'svelte/compiler';

const stripTypes = new Bun.Transpiler({ loader: 'ts' });

plugin({
  name: 'svelte-runes-modules',
  setup(build) {
    build.onLoad({ filter: /\.svelte\.ts$/ }, async ({ path }) => {
      const source = stripTypes.transformSync(await Bun.file(path).text());
      const { js } = compileModule(source, { filename: path, generate: 'client' });
      return { contents: js.code, loader: 'js' };
    });
  },
});
