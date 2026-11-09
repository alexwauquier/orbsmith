<script lang="ts">
  import * as Item from "$lib/components/ui/item";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import ChevronUpIcon from "@lucide/svelte/icons/chevron-up";
  import PauseIcon from "@lucide/svelte/icons/pause";
  import PlayIcon from "@lucide/svelte/icons/play";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import XIcon from "@lucide/svelte/icons/x";
  import type { AppSummary, QueueItem, QueueState } from "$lib/api";
  import { formatCountdown, formatDuration } from "$lib/time";

  let {
    queue,
    parallel,
    appsById,
    onstart,
    onpause,
    onclear,
    onremove,
    onmove
  }: {
    queue: QueueState;
    parallel: boolean;
    appsById: Map<string, AppSummary>;
    onstart: () => void;
    onpause: () => void;
    /** Empties the queue and stops its running games. */
    onclear: () => void;
    onremove: (appId: string) => void;
    /** Moves a waiting game one place towards the front (`up`) or the back. */
    onmove: (appId: string, up: boolean) => void;
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
      <Item.Description class="tabular-nums">
        {#if entry.leftSecs !== null && !active}
          {formatCountdown(entry.leftSecs * 1000)} left of {formatDuration(entry.durationSecs)}
        {:else}
          {formatDuration(entry.durationSecs)}
        {/if}
      </Item.Description>
    </Item.Content>
    <Item.Actions>
      {#if active}
        <Badge variant="secondary">Running</Badge>
      {:else}
        <div class="flex flex-col">
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Move {entry.appName} up"
            disabled={position === 1}
            onclick={() => onmove(entry.appId, true)}
          >
            <ChevronUpIcon />
          </Button>
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Move {entry.appName} down"
            disabled={position === queue.queued.length}
            onclick={() => onmove(entry.appId, false)}
          >
            <ChevronDownIcon />
          </Button>
        </div>
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
        {parallel ? "All at once" : "One after another"}{queue.paused ? " · Paused" : ""}
      </p>
      <Button
        variant="ghost"
        size="icon-sm"
        class="ml-auto"
        aria-label="Clear the queue"
        title="Clear the queue"
        onclick={onclear}
      >
        <Trash2Icon />
      </Button>
      {#if queue.running}
        <Button variant="outline" size="sm" onclick={onpause}>
          <PauseIcon data-icon="inline-start" />
          Pause
        </Button>
      {:else}
        <Button size="sm" onclick={onstart}>
          <PlayIcon data-icon="inline-start" />
          {queue.paused ? "Resume" : "Start queue"}
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
