<script lang="ts">
  import * as Item from "$lib/components/ui/item";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import PauseIcon from "@lucide/svelte/icons/pause";
  import PlayIcon from "@lucide/svelte/icons/play";
  import XIcon from "@lucide/svelte/icons/x";
  import type { AppSummary, QueueItem, QueueState } from "$lib/api";
  import { formatDuration } from "$lib/time";

  let {
    queue,
    parallel,
    appsById,
    onstart,
    onstop,
    onremove
  }: {
    queue: QueueState;
    parallel: boolean;
    appsById: Map<string, AppSummary>;
    onstart: () => void;
    onstop: () => void;
    onremove: (appId: string) => void;
  } = $props();

  const empty = $derived(queue.active.length === 0 && queue.queued.length === 0);
</script>

{#snippet row(entry: QueueItem, active: boolean, position: number)}
  <Item.Root variant="outline" size="sm">
    <Item.Media variant="image">
      <AppIcon
        app={{
          id: entry.appId,
          name: entry.appName,
          iconHash: appsById.get(entry.appId)?.iconHash ?? null
        }}
        class="size-full"
      />
    </Item.Media>
    <Item.Content class="min-w-0">
      <Item.Title class="w-full truncate">{entry.appName}</Item.Title>
      <Item.Description>{formatDuration(entry.durationSecs)}</Item.Description>
    </Item.Content>
    <Item.Actions>
      {#if active}
        <Badge variant="secondary">Running</Badge>
      {:else}
        <Badge variant="outline" class="tabular-nums">#{position}</Badge>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Remove {entry.appName} from the queue"
          onclick={() => onremove(entry.appId)}
        >
          <XIcon />
        </Button>
      {/if}
    </Item.Actions>
  </Item.Root>
{/snippet}

<div class="flex flex-col gap-3">
  {#if empty}
    <p class="text-sm text-muted-foreground">
      The queue is empty. Choose a game, then <span class="font-medium">Add to queue</span> to queue it
      with its duration.
    </p>
  {:else}
    <div class="flex items-center gap-2">
      <p class="text-xs text-muted-foreground">
        {parallel ? "All at once" : "One after another"}
      </p>
      {#if queue.running}
        <Button variant="outline" size="sm" class="ml-auto" onclick={onstop}>
          <PauseIcon data-icon="inline-start" />
          Pause
        </Button>
      {:else}
        <Button size="sm" class="ml-auto" onclick={onstart}>
          <PlayIcon data-icon="inline-start" />
          Start queue
        </Button>
      {/if}
    </div>
    <div class="flex flex-col gap-2">
      {#each queue.active as item (item.appId)}
        {@render row(item, true, 0)}
      {/each}
      {#each queue.queued as item, index (item.appId)}
        {@render row(item, false, index + 1)}
      {/each}
    </div>
  {/if}
</div>
