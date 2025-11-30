<script lang="ts">
  import { Plus, Sun, Moon } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { cn } from "$lib/utils";
  import { setMode } from "mode-watcher";
  import SpaceList from "./SpaceList.svelte";
  import { page } from "$app/state";
</script>

<div
  class="w-[70px] bg-muted/10 border-r flex flex-col items-center py-4 gap-4 shrink-0"
>
  <!-- <Separator class="w-8" /> -->

  <!-- Active Connections -->
  <SpaceList />

  <!-- New Connection / Home -->
  <Button
    variant={page.route.id === "/(space)/new" ? "default" : "ghost"}
    size="icon"
    class={cn(
      "size-12 rounded-xl transition-all duration-200",
      page.route.id === "/(space)/new"
        ? "bg-primary text-primary-foreground hover:bg-primary/90"
        : "hover:bg-muted",
    )}
    href="/new"
    title="New Connection"
  >
    <Plus size={24} />
  </Button>

  <div class="mt-auto">
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <Button variant="ghost" size="icon" class="w-10 h-10 rounded-full">
          <Sun
            class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
          />
          <Moon
            class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
          />
          <span class="sr-only">Toggle theme</span>
        </Button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end">
        <DropdownMenu.Item onclick={() => setMode("light")}>
          Light
        </DropdownMenu.Item>
        <DropdownMenu.Item onclick={() => setMode("dark")}>
          Dark
        </DropdownMenu.Item>
        <DropdownMenu.Item onclick={() => setMode("system")}>
          System
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </div>
</div>
