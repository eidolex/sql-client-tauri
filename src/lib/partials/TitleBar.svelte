<script lang="ts">
  import { getAppState } from "$lib/stores/state.svelte";
  import { Minus, Square, X } from "lucide-svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { page } from "$app/state";

  const appState = getAppState();
  const appWindow = getCurrentWindow();

  const currentSpace = $derived.by(() => {
    const id = page.params.id;
    if (!id) return null;
    return appState.spaces.get(id);
  });

  async function minimizeWindow() {
    await appWindow.minimize();
  }

  async function maximizeWindow() {
    await appWindow.toggleMaximize();
  }

  async function closeWindow() {
    await appWindow.close();
  }
</script>

<div
  class="h-10 bg-background/95 backdrop-blur-sm border-b flex items-center justify-between px-4 select-none shrink-0"
>
  <!-- Left Section: Connection Info (Draggable) -->
  <div class="flex items-center gap-3 flex-1" data-tauri-drag-region>
    <div class="flex items-center gap-2 pointer-events-none">
      {#if currentSpace}
        <div
          class="w-2 h-2 rounded-full"
          class:bg-green-500={currentSpace.status === "connected"}
          class:bg-yellow-500={currentSpace.status === "connecting"}
          class:bg-gray-400={currentSpace.status === "initial"}
        ></div>
        <span class="text-sm font-semibold text-foreground">
          {currentSpace.name}
        </span>
        {#if currentSpace.config.host}
          <span class="text-xs text-muted-foreground">
            {currentSpace.config.host}:{currentSpace.config.port}
          </span>
        {/if}
        <span class="text-xs text-muted-foreground">
          | {currentSpace.currentDatabase}
        </span>
      {:else}
        <span class="text-sm font-semibold text-foreground">SQL Client</span>
      {/if}
    </div>
  </div>

  <!-- Center Section: Draggable Area -->
  <!-- <div class="flex-1 h-full" data-tauri-drag-region></div> -->

  <!-- Right Section: Window Controls (Not Draggable) -->
  <div class="flex items-center gap-2">
    <button
      onclick={minimizeWindow}
      class="w-8 h-8 rounded-md flex items-center justify-center hover:bg-muted/50 transition-colors"
      title="Minimize"
    >
      <Minus class="w-4 h-4 text-muted-foreground" />
    </button>
    <button
      onclick={maximizeWindow}
      class="w-8 h-8 rounded-md flex items-center justify-center hover:bg-muted/50 transition-colors"
      title="Maximize"
    >
      <Square class="w-3.5 h-3.5 text-muted-foreground" />
    </button>
    <button
      onclick={closeWindow}
      class="w-8 h-8 rounded-md flex items-center justify-center hover:bg-destructive/10 hover:text-destructive transition-colors"
      title="Close"
    >
      <X class="w-4 h-4" />
    </button>
  </div>
</div>
