<section class="py-24 relative overflow-hidden">
	<!-- Background glow -->
	<div
		class="absolute top-1/2 left-0 -translate-y-1/2 w-96 h-96 bg-cyan-500/5 rounded-full blur-3xl -z-10"
	></div>

	<div class="container mx-auto px-4 sm:px-6">
		<div class="lg:grid lg:grid-cols-2 lg:gap-16 items-center">
			<div class="mb-12 lg:mb-0">
				<h2 class="text-3xl md:text-4xl font-bold text-white mb-6">
					Intelligent Terminal Interface
				</h2>
				<p class="text-slate-400 text-lg mb-8 leading-relaxed">
					RVault isn't just a command-line tool; it features a full-fledged, interactive terminal
					user interface designed for speed and usability.
				</p>

				<div class="space-y-6">
					<div class="flex items-start gap-4">
						<div
							class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400 shrink-0"
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
								><path d="m18 16 4-4-4-4" /><path d="m6 8-4 4 4 4" /><path d="m14.5 4-5 16" /></svg
							>
						</div>
						<div>
							<h3 class="text-white font-semibold mb-1">Vim-like Keybindings</h3>
							<p class="text-slate-400 text-sm">
								Navigate lists with <code class="bg-slate-800 px-1 py-0.5 rounded text-slate-200"
									>j/k</code
								> or arrow keys for maximum efficiency. Keep your hands on the keyboard.
							</p>
						</div>
					</div>

					<div class="flex items-start gap-4">
						<div
							class="w-10 h-10 rounded-lg bg-cyan-500/10 flex items-center justify-center text-cyan-400 shrink-0"
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
								><path
									d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"
								/></svg
							>
						</div>
						<div>
							<h3 class="text-white font-semibold mb-1">Instant Feedback</h3>
							<p class="text-slate-400 text-sm">
								Get immediate toast notifications and context-aware help at the bottom of the
								screen.
							</p>
						</div>
					</div>

					<div class="flex items-start gap-4">
						<div
							class="w-10 h-10 rounded-lg bg-indigo-500/10 flex items-center justify-center text-indigo-400 shrink-0"
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
								><rect width="18" height="18" x="3" y="3" rx="2" /><path d="M7 7h10" /><path
									d="M7 12h10"
								/><path d="M7 17h10" /></svg
							>
						</div>
						<div>
							<h3 class="text-white font-semibold mb-1">Rich Data Display</h3>
							<p class="text-slate-400 text-sm">
								View comprehensive details including Platform, User ID, and precise timestamps
								neatly organized.
							</p>
						</div>
					</div>
				</div>
			</div>

			<!-- TUI Mockup -->
			<div class="relative group">
				<div
					class="absolute -inset-1 bg-gradient-to-r from-cyan-500 to-emerald-500 rounded-xl blur opacity-25 group-hover:opacity-50 transition duration-1000"
				></div>
				<div
					class="relative w-full aspect-[4/3] md:aspect-video bg-[#1e293b] border border-slate-700 rounded-xl shadow-2xl font-mono text-xs md:text-sm overflow-hidden flex flex-col"
				>
					<!-- Header -->
					<div
						class="flex items-center gap-6 px-4 py-3 border-b border-slate-700 bg-slate-800/50 text-slate-400"
					>
						<span class="font-bold text-slate-200 tracking-wider">RVault</span>
						<div class="h-4 w-px bg-slate-700"></div>
						<div class="flex gap-1 bg-slate-900/50 p-1 rounded-lg border border-slate-700/50">
							<span class="bg-cyan-600 text-white px-3 py-0.5 rounded text-xs font-medium shadow-sm"
								>Vaults</span
							>
							<span
								class="text-slate-500 px-3 py-0.5 rounded text-xs hover:text-slate-300 transition-colors"
								>Generator</span
							>
						</div>
					</div>

					<!-- Sub-Header -->
					<div
						class="px-4 py-2 text-slate-500 border-b border-slate-700/50 bg-slate-900/30 text-[10px] uppercase tracking-wider font-bold"
					>
						Entries
					</div>

					<!-- List Content -->
					<div class="flex-1 bg-[#0f172a] p-4 font-mono">
						<div
							class="flex items-center gap-4 text-slate-400 group/row hover:bg-slate-800/30 p-1 -mx-1 rounded cursor-pointer transition-colors border-l-2 border-transparent hover:border-cyan-500"
						>
							<span class="min-w-[80px]">dsad</span>
							<span class="text-slate-700">|</span>
							<span class="min-w-[100px]">sad</span>
							<span class="text-slate-700">|</span>
							<span class="text-slate-500">February 09 2026 00:38</span>
						</div>
					</div>

					<!-- Footer Status Bar -->
					<div
						class="border-t border-slate-700 bg-slate-800/80 px-4 py-2 text-[10px] text-slate-500 overflow-hidden"
					>
						<div class="flex items-center gap-3 whitespace-nowrap overflow-x-auto scrollbar-hide">
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Navigate:</span> ↑/↓</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Copy:</span> Enter</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"><span class="text-slate-400">Pin:</span> p</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"><span class="text-slate-400">Add:</span> a</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Edit:</span> e</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Delete:</span> d</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Switch Tab:</span> Tab</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Themes:</span> t</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Sort:</span> S</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Lock:</span> Q</span
							>
							<span class="text-slate-700">|</span>
							<span class="flex items-center gap-1"
								><span class="text-slate-400">Quit:</span> q</span
							>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</section>

<style>
	.scrollbar-hide::-webkit-scrollbar {
		display: none;
	}
	.scrollbar-hide {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}
</style>
