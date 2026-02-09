<script lang="ts">
	import { fade } from 'svelte/transition';

	let activeTab = $state('unix');

	const copyToClipboard = async (text: string) => {
		try {
			await navigator.clipboard.writeText(text);
			// In a real app, I'd show a toast here
		} catch (err) {
			console.error('Failed to copy:', err);
		}
	};
</script>

<section id="install" class="py-24 bg-slate-900/30">
	<div class="container mx-auto px-4 sm:px-6">
		<div class="max-w-4xl mx-auto">
			<div class="text-center mb-16">
				<h2 class="text-3xl md:text-4xl font-bold text-white mb-4">Get Started in Seconds</h2>
				<p class="text-slate-400">
					Install RVault via our automated scripts. No dependencies, no complex setup.
				</p>
			</div>

			<div
				class="bg-slate-900 border border-slate-800 rounded-2xl overflow-hidden shadow-2xl mb-16"
			>
				<!-- Tab Headers -->
				<div class="flex flex-wrap border-b border-slate-800">
					<button
						class="flex-1 min-w-[120px] px-4 sm:px-6 py-4 text-sm font-medium transition-colors hover:bg-slate-800/50 focus:outline-none whitespace-nowrap {activeTab ===
						'unix'
							? 'text-emerald-400 bg-slate-800/50 border-b-2 border-emerald-500'
							: 'text-slate-400'}"
						onclick={() => (activeTab = 'unix')}
					>
						Linux / macOS
					</button>
					<button
						class="flex-1 min-w-[120px] px-4 sm:px-6 py-4 text-sm font-medium transition-colors hover:bg-slate-800/50 focus:outline-none whitespace-nowrap {activeTab ===
						'windows'
							? 'text-blue-400 bg-slate-800/50 border-b-2 border-blue-500'
							: 'text-slate-400'}"
						onclick={() => (activeTab = 'windows')}
					>
						Windows
					</button>
				</div>

				<!-- Tab Content -->
				<div class="p-6 md:p-8 bg-slate-950/50 min-h-[160px] flex items-center justify-center">
					{#if activeTab === 'unix'}
						<div class="w-full relative group" in:fade>
							<div
								class="absolute right-0 top-0 opacity-0 group-hover:opacity-100 transition-opacity"
							>
								<button
									class="text-xs text-slate-500 hover:text-white bg-slate-800 px-2 py-1 rounded"
									onclick={() =>
										copyToClipboard(
											"curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ata-sesli/rvault/releases/download/v0.1.1/rvault-cli-installer.sh | sh"
										)}>Copy</button
								>
							</div>
							<pre
								class="font-mono text-sm text-slate-300 overflow-x-auto whitespace-pre-wrap breaks-words"><span
									class="text-emerald-400">curl</span
								> --proto '=https' --tlsv1.2 -LsSf https://github.com/ata-sesli/rvault/releases/download/v0.1.1/rvault-cli-installer.sh | sh</pre>
						</div>
					{:else if activeTab === 'windows'}
						<div class="w-full relative group" in:fade>
							<div
								class="absolute right-0 top-0 opacity-0 group-hover:opacity-100 transition-opacity"
							>
								<button
									class="text-xs text-slate-500 hover:text-white bg-slate-800 px-2 py-1 rounded"
									onclick={() =>
										copyToClipboard(
											'powershell -ExecutionPolicy Bypass -c "irm https://github.com/ata-sesli/rvault/releases/download/v0.1.1/rvault-cli-installer.ps1 | iex"'
										)}>Copy</button
								>
							</div>
							<pre
								class="font-mono text-sm text-slate-300 overflow-x-auto whitespace-pre-wrap break-words"><span
									class="text-blue-400">powershell</span
								> -ExecutionPolicy Bypass -c "irm https://github.com/ata-sesli/rvault/releases/download/v0.1.1/rvault-cli-installer.ps1 | iex"</pre>
						</div>
					{/if}
				</div>
			</div>

			<!-- Quick Usage Guide -->
			<div class="grid md:grid-cols-2 gap-8">
				<div>
					<h3 class="text-xl font-bold text-white mb-6 flex items-center gap-2">
						<span
							class="w-8 h-8 rounded-lg bg-slate-800 flex items-center justify-center text-cyan-400 text-sm"
							>01</span
						>
						Setup & Secure
					</h3>
					<div
						class="bg-slate-900 border border-slate-800 rounded-xl p-6 group hover:border-cyan-500/30 transition-colors"
					>
						<p class="text-slate-400 text-sm mb-4">
							Initialize your vault and create a master password. This is done only once.
						</p>
						<code class="block font-mono text-sm bg-slate-950 p-3 rounded text-cyan-300"
							>rvault setup</code
						>
					</div>
				</div>

				<div>
					<h3 class="text-xl font-bold text-white mb-6 flex items-center gap-2">
						<span
							class="w-8 h-8 rounded-lg bg-slate-800 flex items-center justify-center text-emerald-400 text-sm"
							>02</span
						>
						Store Credentials
					</h3>
					<div
						class="bg-slate-900 border border-slate-800 rounded-xl p-6 group hover:border-emerald-500/30 transition-colors"
					>
						<p class="text-slate-400 text-sm mb-4">
							Add passwords easily. Format: <span class="text-slate-300"
								>service username:password</span
							>
						</p>
						<code class="block font-mono text-sm bg-slate-950 p-3 rounded text-emerald-300"
							>rvault add github user:token</code
						>
					</div>
				</div>

				<div>
					<h3 class="text-xl font-bold text-white mb-6 flex items-center gap-2">
						<span
							class="w-8 h-8 rounded-lg bg-slate-800 flex items-center justify-center text-purple-400 text-sm"
							>03</span
						>
						Instant Retrieval
					</h3>
					<div
						class="bg-slate-900 border border-slate-800 rounded-xl p-6 group hover:border-purple-500/30 transition-colors"
					>
						<p class="text-slate-400 text-sm mb-4">
							Get passwords copied to clipboard instantly. No standard output for security.
						</p>
						<code class="block font-mono text-sm bg-slate-950 p-3 rounded text-purple-300"
							>rvault get github user</code
						>
					</div>
				</div>

				<div>
					<h3 class="text-xl font-bold text-white mb-6 flex items-center gap-2">
						<span
							class="w-8 h-8 rounded-lg bg-slate-800 flex items-center justify-center text-orange-400 text-sm"
							>04</span
						>
						Multiple Vaults
					</h3>
					<div
						class="bg-slate-900 border border-slate-800 rounded-xl p-6 group hover:border-orange-500/30 transition-colors"
					>
						<p class="text-slate-400 text-sm mb-4">
							Organize work and personal credentials in separate encrypted containers.
						</p>
						<code class="block font-mono text-sm bg-slate-950 p-3 rounded text-orange-300"
							>rvault create work_vault</code
						>
					</div>
				</div>
			</div>

			<div class="mt-16 text-center">
				<p class="text-slate-500 mb-4">Ready to dive deeper?</p>
				<a
					href="https://github.com/ata-sesli/rvault"
					target="_blank"
					rel="noreferrer"
					class="inline-flex items-center gap-2 text-cyan-400 hover:text-cyan-300 font-medium transition-colors"
				>
					View Full Documentation <span aria-hidden="true">&rarr;</span>
				</a>
			</div>
		</div>
	</div>
</section>
