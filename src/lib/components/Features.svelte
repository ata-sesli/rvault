<script lang="ts">
	import FeatureCard from './FeatureCard.svelte';
</script>

<section id="features" class="py-24 bg-slate-900/30">
	<div class="container mx-auto px-4 sm:px-6">
		<div class="text-center mb-16">
			<h2 class="text-3xl md:text-4xl font-bold text-white mb-4">Everything you need</h2>
			<p class="text-slate-400 max-w-2xl mx-auto">
				Built for power users who demand security without sacrificing speed or usability.
			</p>
		</div>

		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			<FeatureCard
				title="Modern TUI"
				description="A beautiful, responsive Terminal User Interface designed for efficiency with Vim-like keybindings."
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					><rect width="20" height="14" x="2" y="3" rx="2" /><line
						x1="8"
						x2="16"
						y1="21"
						y2="21"
					/><line x1="12" x2="12" y1="17" y2="21" /></svg
				>
			</FeatureCard>

			<FeatureCard
				title="Military-Grade Encryption"
				description="Your data is secured with ChaCha20-Poly1305 encryption and Argon2 key derivation."
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					><rect width="18" height="11" x="3" y="11" rx="2" ry="2" /><path
						d="M7 11V7a5 5 0 0 1 10 0v4"
					/></svg
				>
			</FeatureCard>

			<FeatureCard
				title="Fully Local"
				description="Zero cloud dependencies. Your vault lives on your machine and never leaves without your permission."
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					><path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z" /><circle
						cx="12"
						cy="10"
						r="3"
					/></svg
				>
			</FeatureCard>

			<FeatureCard
				title="Secure Generator"
				description="Generate strong, unique passwords instantly with a visual strength indicator."
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					><path
						d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"
					/></svg
				>
			</FeatureCard>

			<FeatureCard
				title="Theme System"
				description="Customize your experience with 8+ premium themes including Gruvbox, Catppuccin, and Nord."
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					><circle cx="13.5" cy="6.5" r=".5" fill="currentColor" /><circle
						cx="17.5"
						cy="10.5"
						r=".5"
						fill="currentColor"
					/><circle cx="8.5" cy="7.5" r=".5" fill="currentColor" /><circle
						cx="6.5"
						cy="12.5"
						r=".5"
						fill="currentColor"
					/><path
						d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.172 0-1.107.953-2 2.125-2 1.107 0 2-.953 2-2.125 0-.52-.148-.883-.438-1.173-.29-.289-.438-.688-.438-1.125C15.992 5.344 14.195 2 12 2z"
					/></svg
				>
			</FeatureCard>

			<FeatureCard
				title="Rust-Powered"
				description="Built for blazing speed and memory safety with zero-cost abstractions."
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" /></svg
				>
			</FeatureCard>
		</div>
	</div>
</section>
