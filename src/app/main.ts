import '@fontsource/newsreader/index.css';
import '@fontsource/geist-sans/index.css';
import '@fontsource/geist-mono/index.css';
import 'virtual:uno.css';
import './tokens.css';

import { mount } from 'svelte';
import App from './app.svelte';

const target = document.getElementById('app');
if (!target) {
  throw new Error('Missing #app mount point in index.html');
}

mount(App, { target });
