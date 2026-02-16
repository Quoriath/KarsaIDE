<script>
  import { onMount, onDestroy, tick } from 'svelte';
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
    breaks: true,
    gfm: true // GitHub Flavored Markdown (includes tables)
  });

  let { content = '' } = $props();
  let htmlContent = $state('');

  $effect(() => {
    htmlContent = marked.parse(content);
  });

  let container;
  
  // Handle code block enhancements (copy button + language badge)
  $effect(() => {
    if (container && htmlContent) {
      const pres = container.querySelectorAll('pre');
      pres.forEach((pre) => {
        if (pre.querySelector('.code-enhancement-toolbar')) return;
        
        // Wrap pre in a container for positioning
        const wrapper = document.createElement('div');
        wrapper.className = 'relative group/code-block my-4';
        
        pre.parentNode.insertBefore(wrapper, pre);
        wrapper.appendChild(pre);
        
        // Extract language from class if available (e.g., hljs language-javascript)
        const codeElem = pre.querySelector('code');
        const classList = Array.from(codeElem?.classList || []);
        const langMatch = classList.find(c => c.startsWith('language-'));
        const lang = langMatch ? langMatch.replace('language-', '').toUpperCase() : 'CODE';
        
        const toolbar = document.createElement('div');
        toolbar.className = 'code-enhancement-toolbar absolute top-0 right-0 left-0 h-8 px-3 flex items-center justify-between bg-muted/30 border-b border-border/20 backdrop-blur-sm opacity-0 group-hover/code-block:opacity-100 transition-all duration-200 pointer-events-none rounded-t-lg';
        
        toolbar.innerHTML = `
          <span class="text-[9px] font-black tracking-[0.2em] text-muted-foreground/60 uppercase pointer-events-none">${lang}</span>
          <div class="flex items-center gap-2 pointer-events-auto"></div>
        `;
        
        const actionArea = toolbar.querySelector('div');
        
        // Copy Button
        const copyBtn = document.createElement('button');
        copyBtn.className = 'flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-primary/10 text-muted-foreground hover:text-primary transition-all duration-200';
        copyBtn.innerHTML = `
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
          <span class="text-[9px] font-bold uppercase tracking-wider">Copy</span>
        `;
        
        copyBtn.onclick = () => {
           const codeText = codeElem?.innerText || '';
           navigator.clipboard.writeText(codeText);
           copyBtn.innerHTML = `
             <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="text-green-500"><path d="M20 6 9 17l-5-5"/></svg>
             <span class="text-[9px] font-bold uppercase tracking-wider text-green-500">Copied</span>
           `;
           setTimeout(() => {
              copyBtn.innerHTML = `
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
                <span class="text-[9px] font-bold uppercase tracking-wider">Copy</span>
              `;
           }, 2000);
        };
        
        actionArea.appendChild(copyBtn);
        wrapper.prepend(toolbar);
        
        // Add some top padding to pre to account for toolbar
        pre.style.paddingTop = '24px';
      });
    }
  });

</script>

<div class="markdown-body text-foreground/90" bind:this={container}>
  {@html htmlContent}
</div>

<style>
  :global(.code-enhancement-toolbar) {
    z-index: 10;
  }
</style>
