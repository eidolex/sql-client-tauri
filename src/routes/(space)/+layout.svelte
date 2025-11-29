<script>
  import ActivityBar from "../../lib/partials/ActivityBar.svelte";

  let { children } = $props();
</script>

<div class="flex w-full h-full">
  <ActivityBar />
  <!-- Content Area -->
  {@render children()}
</div>
