import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Keep existing bookmarked local preview URLs working during the migration.
function legacyPreview(server) {
  server.middlewares.use((request, response, next) => {
    const base = process.env.BASE_PATH || '';
    const url = new URL(request.url, 'http://localhost');
    if (url.pathname !== `${base}/index.html`) return next();
    response.writeHead(302, { Location: `${base}/${url.search}` });
    response.end();
  });
}

export default defineConfig({
  plugins: [{ name: 'legacy-preview-url', configureServer: legacyPreview }, sveltekit({
    files: { params: 'src/params.js' },
    adapter: adapter({ fallback: 'index.html' }),
    paths: { base: process.env.BASE_PATH || '' }
  })]
});
