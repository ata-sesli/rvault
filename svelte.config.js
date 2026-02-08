import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://svelte.dev/docs/kit/integrations
	// for more information about preprocessors
	preprocess: vitePreprocess(),

	kit: {
		paths: {
            base: process.env.NODE_ENV === 'production' ? '/rvault' : '',
        },
		appDir: 'internal',
        adapter: adapter({
            // Crucial for SPA routing on GitHub Pages
            fallback: '404.html' 
        })
	}
};

export default config;
