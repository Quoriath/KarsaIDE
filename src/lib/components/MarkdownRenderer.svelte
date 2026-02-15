<script>
  import { onMount } from 'svelte';
  import { marked } from 'marked';
  import hljs from 'highlight.js';
  import { Copy, Check } from 'lucide-svelte';
  
  // Configure marked to use highlight.js
  marked.setOptions({
    highlight: function(code, lang) {
      const language = hljs.getLanguage(lang) ? lang : 'plaintext';
      return hljs.highlight(code, { language }).value;
    },
    langPrefix: 'hljs language-',
  });

  let { content = '' } = $props();
  let htmlContent = $state('');

  $effect(() => {
    htmlContent = marked.parse(content);
  });

  // Handle copy button clicks
  function handleCopy(code) {
    navigator.clipboard.writeText(code);
  }

  // We need to hydrate the HTML with copy buttons after render
  // This is a bit tricky with Svelte's reactivity, so we might need a custom action or
  // just simple DOM manipulation after update.
  
  let container;
  
  $effect(() => {
    if (container && htmlContent) {
      // Find all pre blocks and add copy buttons if not present
      const pres = container.querySelectorAll('pre');
      pres.forEach((pre) => {
        if (pre.querySelector('.copy-btn')) return; // Already processed
        
        const wrapper = document.createElement('div');
        wrapper.className = 'relative group';
        
        // Clone pre to wrapper
        pre.parentNode.insertBefore(wrapper, pre);
        wrapper.appendChild(pre);
        
        const code = pre.querySelector('code')?.innerText || '';
        
        const btn = document.createElement('button');
        btn.className = 'copy-btn absolute top-2 right-2 p-1.5 rounded-md bg-muted/50 hover:bg-muted text-muted-foreground hover:text-foreground opacity-0 group-hover:opacity-100 transition-opacity';
        btn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-copy"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>`;
        
        btn.onclick = () => {
           navigator.clipboard.writeText(code);
           btn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-check text-green-500"><path d="M20 6 9 17l-5-5"/></svg>`;
           setTimeout(() => {
              btn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-copy"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>`;
           }, 2000);
        };
        
        wrapper.appendChild(btn);
      });
    }
  });

</script>

<div class="markdown-body text-sm leading-7 text-foreground/90" bind:this={container}>
  {@html htmlContent}
</div>
