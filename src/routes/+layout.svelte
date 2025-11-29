<script>
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  import { setAppState } from "$lib/state.svelte";

  let { children } = $props();

  setAppState();
</script>

<ModeWatcher />

<div
  class="h-screen w-screen bg-background text-foreground overflow-hidden font-sans antialiased"
>
  {@render children()}
</div>
