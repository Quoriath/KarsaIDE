<script>
  import { uiState } from '../../stores/uiState.svelte.js';
  import { X, AlertCircle } from 'lucide-svelte';
  import { cn } from '../../utils.js';

  function handleConfirm() {
    uiState.dialog.onConfirm?.();
    uiState.closeDialog();
  }

  function handleCancel() {
    uiState.dialog.onCancel?.();
    uiState.closeDialog();
  }
</script>

{#if uiState.dialog.visible}
  <div 
    class="fixed inset-0 z-[200] bg-black/40 backdrop-blur-[2px] flex items-center justify-center p-4 animate-in fade-in duration-200"
    onclick={handleCancel}
  >
    <div 
      class="bg-background border border-border shadow-2xl rounded-xl w-full max-w-md overflow-hidden animate-in zoom-in-95 duration-200"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-6 py-4 border-b border-border flex items-center justify-between">
        <h3 class="text-sm font-bold uppercase tracking-widest text-foreground/80">{uiState.dialog.title}</h3>
        <button 
          onclick={handleCancel}
          class="p-1 hover:bg-muted rounded-md text-muted-foreground transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Content -->
      <div class="px-6 py-8 flex items-start gap-4">
        {#if uiState.dialog.variant === 'danger'}
          <div class="w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center text-destructive shrink-0">
            <AlertCircle size={20} />
          </div>
        {/if}
        <div class="flex-1">
          <p class="text-sm leading-relaxed text-muted-foreground">{uiState.dialog.message}</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-muted/10 border-t border-border flex items-center justify-end gap-3">
        <button 
          onclick={handleCancel}
          class="px-4 py-2 text-xs font-bold text-muted-foreground hover:text-foreground transition-colors"
        >
          {uiState.dialog.cancelLabel}
        </button>
        <button 
          onclick={handleConfirm}
          class={cn(
            "px-5 py-2 text-xs font-black uppercase tracking-widest rounded-lg shadow-lg transition-all active:scale-95",
            uiState.dialog.variant === 'danger' 
              ? "bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-destructive/20" 
              : "bg-primary text-primary-foreground hover:bg-primary/90 shadow-primary/20"
          )}
        >
          {uiState.dialog.confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
