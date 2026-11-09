<script lang="ts">
  import * as Item from "$lib/components/ui/item";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import PauseIcon from "@lucide/svelte/icons/pause";
  import PlayIcon from "@lucide/svelte/icons/play";
  import XIcon from "@lucide/svelte/icons/x";
  import type { AppSummary, Quest, QuestsState } from "$lib/api";
  import { formatQuestDuration } from "$lib/time";

  let {
    quests,
    parallel,
    appsById,
    onstart,
    onstop,
    onremove
  }: {
    quests: QuestsState;
    parallel: boolean;
    appsById: Map<string, AppSummary>;
    onstart: () => void;
    onstop: () => void;
    onremove: (appId: string) => void;
  } = $props();

  const empty = $derived(quests.active.length === 0 && quests.queued.length === 0);
</script>

{#snippet quest(quest: Quest, active: boolean, position: number)}
  <Item.Root variant="outline" size="sm">
    <Item.Media variant="image">
      <AppIcon
        app={{
          id: quest.appId,
          name: quest.appName,
          iconHash: appsById.get(quest.appId)?.iconHash ?? null
        }}
        class="size-full"
      />
    </Item.Media>
    <Item.Content class="min-w-0">
      <Item.Title class="w-full truncate">{quest.appName}</Item.Title>
      <Item.Description>{formatQuestDuration(quest.durationSecs)}</Item.Description>
    </Item.Content>
    <Item.Actions>
      {#if active}
        <Badge variant="secondary">Running</Badge>
      {:else}
        <Badge variant="outline" class="tabular-nums">#{position}</Badge>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Remove {quest.appName} from the quests"
          onclick={() => onremove(quest.appId)}
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
      No quests yet. Choose a game, then <span class="font-medium">Add to quests</span> to queue it with
      its quest duration.
    </p>
  {:else}
    <div class="flex items-center gap-2">
      <p class="text-xs text-muted-foreground">
        {parallel ? "All at once" : "One after another"}
      </p>
      {#if quests.running}
        <Button variant="outline" size="sm" class="ml-auto" onclick={onstop}>
          <PauseIcon data-icon="inline-start" />
          Pause
        </Button>
      {:else}
        <Button size="sm" class="ml-auto" onclick={onstart}>
          <PlayIcon data-icon="inline-start" />
          Start quests
        </Button>
      {/if}
    </div>
    <div class="flex flex-col gap-2">
      {#each quests.active as item (item.appId)}
        {@render quest(item, true, 0)}
      {/each}
      {#each quests.queued as item, index (item.appId)}
        {@render quest(item, false, index + 1)}
      {/each}
    </div>
  {/if}
</div>
