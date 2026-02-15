<script>
  let { name, icon, selected, onClick, recommended = false } = $props();
</script>

<button
  onclick={onClick}
  class="relative p-4 rounded-xl border-2 transition-all text-center
         {selected 
           ? 'border-neon-cyan bg-neon-cyan/10' 
           : 'border-zinc-700 hover:border-zinc-600 bg-zinc-900'}"
>
  {#if recommended}
    <span class="absolute -top-2 left-1/2 -translate-x-1/2 px-2 py-0.5 bg-neon-pink text-xs font-medium text-white rounded-full">
      Recommended
    </span>
  {/if}
  <div class="text-2xl mb-1">{icon}</div>
  <div class="text-sm font-medium {selected ? 'text-white' : 'text-gray-400'}">{name}</div>
</button>
