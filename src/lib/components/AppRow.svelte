<script lang="ts">
  import * as Item from "$lib/components/ui/item";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import PlayIcon from "@lucide/svelte/icons/play";
  import StarIcon from "@lucide/svelte/icons/star";
  import type { AppSummary } from "$lib/api";

  let {
    app,
    running,
    queued,
    favorite,
    onstart,
    ontogglefavorite
  }: {
    app: AppSummary;
    running: boolean;
    /** Waiting in the queue; a running queued game shows as running. */
    queued: boolean;
    favorite: boolean;
    onstart: () => void;
    ontogglefavorite: () => void;
  } = $props();
</script>

<Item.Root size="sm">
  <Item.Media variant="image">
    <AppIcon {app} class="size-full" />
  </Item.Media>
  <Item.Content class="min-w-0">
    <Item.Title class="w-full truncate">{app.name}</Item.Title>
    <Item.Description class="truncate font-mono text-xs">{app.executable}</Item.Description>
  </Item.Content>
  <Item.Actions>
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label={favorite ? "Remove from favorites" : "Add to favorites"}
      aria-pressed={favorite}
      onclick={ontogglefavorite}
    >
      <StarIcon class={favorite ? "fill-primary text-primary" : "text-muted-foreground"} />
    </Button>
    {#if running}
      <Badge variant="secondary">Running</Badge>
    {:else}
      {#if queued}
        <Badge variant="outline">In queue</Badge>
      {/if}
      <Button variant="secondary" size="sm" onclick={onstart}>
        <PlayIcon data-icon="inline-start" />
        Start
      </Button>
    {/if}
  </Item.Actions>
</Item.Root>
