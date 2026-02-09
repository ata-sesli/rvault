<script lang="ts">
    import { type Snippet } from 'svelte';

    let { title, description, children } = $props<{
        title: string;
        description: string;
        children: Snippet;
    }>();
</script>

<div class="p-6 rounded-2xl bg-slate-900 border border-slate-800 hover:border-cyan-500/30 transition-colors group">
    <div class="w-12 h-12 rounded-lg bg-slate-950 flex items-center justify-center text-cyan-400 mb-4 group-hover:scale-110 transition-transform duration-300">
        {@render children()}
    </div>
    <h3 class="text-xl font-bold text-white mb-2">{title}</h3>
    <p class="text-slate-400 leading-relaxed text-sm">
        {description}
    </p>
</div>
