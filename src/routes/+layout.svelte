<script>
  import "../app.css";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { ModeWatcher } from "mode-watcher";
  import TitleBar from "$lib/partials/TitleBar.svelte";

  let { children } = $props();
</script>

<ModeWatcher />
<Toaster />

<div
  class="h-screen w-screen bg-background text-foreground overflow-hidden font-sans antialiased flex flex-col"
>
  <TitleBar />
  <div class="flex-1 overflow-hidden">
    {@render children()}
  </div>
</div>
