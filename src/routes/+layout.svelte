<script>
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";

  let { children } = $props();
</script>

<ModeWatcher />

<div
  class="h-screen w-screen bg-background text-foreground overflow-hidden font-sans antialiased"
>
  {@render children()}
</div>
