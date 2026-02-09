<script lang="ts">
	import { onMount } from 'svelte';
	import Features from '$lib/components/Features.svelte';
	import TUIShowcase from '$lib/components/TUIShowcase.svelte';
	import Installation from '$lib/components/Installation.svelte';
	import Footer from '$lib/components/Footer.svelte';

	// Simple fade-in animation check
	let mounted = $state(false);
	onMount(() => {
		mounted = true;
	});
</script>

<div class="relative overflow-hidden pt-24 pb-16 lg:pt-32 lg:pb-24">
	<!-- Background gradients -->
	<div
		class="absolute top-0 left-1/2 -translate-x-1/2 w-full h-full max-w-7xl pointer-events-none select-none"
	>
		<div class="absolute top-20 left-10 w-72 h-72 bg-cyan-500/10 rounded-full blur-3xl"></div>
		<div class="absolute top-10 right-10 w-96 h-96 bg-emerald-500/10 rounded-full blur-3xl"></div>
	</div>

	<div class="container mx-auto px-4 sm:px-6 relative z-10 text-center">
		<div
			class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-slate-900/50 border border-slate-800 text-sm text-slate-400 mb-8 backdrop-blur-sm"
		>
			<span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
			v0.1.1 Now Available
		</div>

		<h1 class="text-5xl md:text-7xl font-bold tracking-tight text-white mb-6">
			The Ultimate <br class="hidden md:block" />
			<span class="text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-emerald-400"
				>Secure Password Manager</span
			>
		</h1>

		<p class="text-lg md:text-xl text-slate-400 max-w-2xl mx-auto mb-10 leading-relaxed">
			Stop compromising your digital security. RVault is a blazing-fast, military-grade password
			manager built in Rust. No cloud, no subscription fees, just pure security.
		</p>

		<div class="flex flex-col sm:flex-row gap-4 justify-center items-center">
			<a
				href="#install"
				class="h-12 px-8 rounded-full bg-white text-slate-950 font-semibold flex items-center gap-2 hover:bg-slate-200 transition-colors"
			>
				Get Started
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="20"
					height="20"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"><path d="M5 12h14" /><path d="m12 5 7 7-7 7" /></svg
				>
			</a>

			<a
				href="https://github.com/ata-sesli/rvault"
				target="_blank"
				rel="noreferrer"
				class="h-12 px-8 rounded-full bg-slate-900 border border-slate-800 text-white font-medium flex items-center gap-2 hover:bg-slate-800 transition-colors group"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="20"
					height="20"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="text-slate-400 group-hover:text-white transition-colors"
					><path
						d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36.5-8 0C6 2 5 2 4 3c-1.25.68-2 2-2 3.5-.28 1.15-.28 2.35 0 3.5A5.403 5.403 0 0 0 4 13.5c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"
					/><path d="M9 18c-4.51 2-5-2-7-2" /></svg
				>
				View on GitHub
			</a>
		</div>
	</div>
</div>

<div class="relative z-10">
	<Features />
	<TUIShowcase />
	<Installation />
	<Footer />
</div>
