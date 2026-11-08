<script lang="ts">
  import * as Item from "$lib/components/ui/item";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import TimerIcon from "@lucide/svelte/icons/timer";
  import SquareIcon from "@lucide/svelte/icons/square";
  import type { Simulation } from "$lib/api";
  import { formatCountdown } from "$lib/time";

  let {
    simulation,
    iconHash,
    now,
    onstop,
    onedittimer
  }: {
    simulation: Simulation;
    iconHash: string | null;
    now: number;
    onstop: () => void;
    onedittimer: () => void;
  } = $props();
</script>

<Item.Root variant="outline" size="sm" class="flex-wrap">
  <Item.Media variant="image">
    <AppIcon app={{ id: simulation.appId, name: simulation.appName, iconHash }} class="size-full" />
  </Item.Media>
  <Item.Content class="min-w-0">
    <Item.Title class="w-full truncate">{simulation.appName}</Item.Title>
    <Item.Description class="truncate font-mono text-xs">{simulation.executable}</Item.Description>
  </Item.Content>
  <Item.Footer>
    <Badge variant={simulation.stopAt === null ? "outline" : "secondary"} class="tabular-nums">
      {simulation.stopAt === null
        ? `Running ${formatCountdown(now - simulation.startedAt)}`
        : `${formatCountdown(simulation.stopAt - now)} left`}
    </Badge>
    <Item.Actions>
      <Button variant="ghost" size="icon-sm" aria-label="Edit timer" onclick={onedittimer}>
        <TimerIcon />
      </Button>
      <Button variant="outline" size="sm" onclick={onstop}>
        <SquareIcon data-icon="inline-start" />
        Stop
      </Button>
    </Item.Actions>
  </Item.Footer>
</Item.Root>
