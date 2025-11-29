<script lang="ts">
  import Button from "$lib/components/ui/button/button.svelte";
  import { Plus } from "lucide-svelte";
  import SavedConnectionList from "$lib/partials/new/SavedConnectionList.svelte";
  import { setConnectionState } from "$lib/stores/connection.state.svelte";
  import ConnectionForm from "$lib/partials/new/ConnectionForm.svelte";

  const connectionState = setConnectionState();
</script>

<div class="flex-1 overflow-hidden relative">
  <div class="flex h-full bg-background text-foreground">
    <div class="w-64 border-r flex flex-col bg-muted/30">
      <div class="p-4 border-b flex justify-between items-center">
        <h3 class="font-semibold text-sm">Connections</h3>
        <Button
          variant="ghost"
          size="icon"
          class="h-8 w-8"
          onclick={() => (connectionState.selectedConnectionId = null)}
        >
          <Plus class="h-4 w-4" />
        </Button>
      </div>
      <SavedConnectionList />
    </div>

    <!-- Main: Connection Form -->
    <div class="flex-1 flex flex-col overflow-y-auto bg-background">
      <ConnectionForm />
    </div>
  </div>
</div>
