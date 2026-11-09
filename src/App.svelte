<script lang="ts">
  import { onMount } from "svelte";
  import * as Alert from "$lib/components/ui/alert";
  import * as Empty from "$lib/components/ui/empty";
  import * as InputGroup from "$lib/components/ui/input-group";
  import * as Item from "$lib/components/ui/item";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Spinner } from "$lib/components/ui/spinner";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import RunningCard from "$lib/components/RunningCard.svelte";
  import StartDialog from "$lib/components/StartDialog.svelte";
  import TimerDialog from "$lib/components/TimerDialog.svelte";
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import GamepadIcon from "@lucide/svelte/icons/gamepad-2";
  import PlayIcon from "@lucide/svelte/icons/play";
  import RefreshIcon from "@lucide/svelte/icons/refresh-cw";
  import SearchIcon from "@lucide/svelte/icons/search";
  import SearchXIcon from "@lucide/svelte/icons/search-x";
  import XIcon from "@lucide/svelte/icons/x";
  import {
    getApps,
    listSimulations,
    onSimulationsChanged,
    setSimulationTimer,
    startSimulation,
    stopSimulation,
    type AppSummary,
    type Simulation
  } from "$lib/api";
  import { searchApps } from "$lib/search";
  import { formatAgo } from "$lib/time";
  import logo from "../assets/logo.svg";

  const RESULT_LIMIT = 50;

  // Raw: ~10k entries that are replaced wholesale, never mutated.
  let apps = $state.raw<AppSummary[]>([]);
  let fetchedAt = $state<number | null>(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let simulations = $state.raw<Simulation[]>([]);
  let error = $state<string | null>(null);
  let query = $state("");
  let now = $state(Date.now());
  let starting = $state<AppSummary | null>(null);
  let editingTimer = $state<Simulation | null>(null);

  const results = $derived(searchApps(apps, query, RESULT_LIMIT));
  const appsById = $derived(new Map(apps.map((app) => [app.id, app])));
  const runningIds = $derived(new Set(simulations.map((sim) => sim.appId)));

  function message(err: unknown): string {
    return typeof err === "string" ? err : err instanceof Error ? err.message : String(err);
  }

  /** Runs `action`, showing its error in the banner instead of throwing. */
  async function attempt(action: () => Promise<void>): Promise<boolean> {
    try {
      await action();
      error = null;
      return true;
    } catch (err) {
      error = message(err);
      return false;
    }
  }

  async function loadApps(forceFetch: boolean) {
    await attempt(async () => {
      const list = await getApps(forceFetch);
      apps = list.apps;
      fetchedAt = list.fetchedAt;
    });
  }

  async function refresh() {
    refreshing = true;
    await loadApps(true);
    refreshing = false;
  }

  async function reloadSimulations() {
    await attempt(async () => {
      simulations = await listSimulations();
    });
  }

  async function start(app: AppSummary, durationSecs: number | undefined) {
    const ok = await attempt(async () => {
      await startSimulation(app.id, durationSecs);
    });
    if (ok) {
      starting = null;
      // Reload rather than append: the change event may already have.
      await reloadSimulations();
    }
  }

  async function stop(appId: string) {
    await attempt(() => stopSimulation(appId));
    await reloadSimulations();
  }

  async function setTimer(appId: string, durationSecs: number | null) {
    const ok = await attempt(async () => {
      const updated = await setSimulationTimer(appId, durationSecs);
      simulations = simulations.map((sim) => (sim.appId === appId ? updated : sim));
    });
    if (ok) editingTimer = null;
  }

  onMount(() => {
    void Promise.all([loadApps(false), reloadSimulations()]).finally(() => (loading = false));

    const tick = setInterval(() => (now = Date.now()), 1000);
    const unlisten = onSimulationsChanged(() => void reloadSimulations());
    return () => {
      clearInterval(tick);
      void unlisten.then((stopListening) => stopListening());
    };
  });
</script>

<div class="flex h-screen flex-col">
  <header class="flex items-center gap-3 px-4 py-3">
    <img src={logo} alt="" class="size-7" />
    <h1 class="text-lg font-semibold">Orbsmith</h1>
    <div class="ml-auto flex items-center gap-2">
      {#if fetchedAt !== null}
        <span class="text-xs text-muted-foreground">
          {apps.length.toLocaleString()} games · updated {formatAgo(fetchedAt, now)}
        </span>
      {/if}
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Refresh the game list"
        disabled={loading || refreshing}
        onclick={refresh}
      >
        {#if refreshing}
          <Spinner />
        {:else}
          <RefreshIcon />
        {/if}
      </Button>
    </div>
  </header>
  <Separator />

  {#if error}
    <Alert.Root variant="destructive" class="mx-4 mt-3 w-auto">
      <CircleAlertIcon />
      <Alert.Title>Something went wrong</Alert.Title>
      <Alert.Description>{error}</Alert.Description>
      <Alert.Action>
        <Button variant="ghost" size="icon-sm" aria-label="Dismiss" onclick={() => (error = null)}>
          <XIcon />
        </Button>
      </Alert.Action>
    </Alert.Root>
  {/if}

  <div class="flex min-h-0 flex-1">
    <main class="flex min-w-0 flex-1 flex-col gap-3 p-4">
      <InputGroup.Root>
        <InputGroup.Input
          placeholder="Search games"
          bind:value={query}
          disabled={loading}
          aria-label="Search games"
        />
        <InputGroup.Addon>
          <SearchIcon />
        </InputGroup.Addon>
      </InputGroup.Root>

      <ScrollArea class="min-h-0 flex-1">
        {#if loading}
          <div class="flex flex-col gap-2">
            {#each { length: 6 }, i (i)}
              <Skeleton class="h-14 w-full" />
            {/each}
          </div>
        {:else if !query.trim()}
          <Empty.Root>
            <Empty.Header>
              <Empty.Media variant="icon"><GamepadIcon /></Empty.Media>
              <Empty.Title>Find a game</Empty.Title>
              <Empty.Description>
                Search by name, then start it to make it appear as running.
              </Empty.Description>
            </Empty.Header>
          </Empty.Root>
        {:else if results.length === 0}
          <Empty.Root>
            <Empty.Header>
              <Empty.Media variant="icon"><SearchXIcon /></Empty.Media>
              <Empty.Title>No game found</Empty.Title>
              <Empty.Description>
                Nothing matches “{query.trim()}” on this platform. Try another name, or refresh the
                list.
              </Empty.Description>
            </Empty.Header>
          </Empty.Root>
        {:else}
          <Item.Group class="gap-1">
            {#each results as app (app.id)}
              <Item.Root size="sm">
                <Item.Media variant="image">
                  <AppIcon {app} class="size-full" />
                </Item.Media>
                <Item.Content class="min-w-0">
                  <Item.Title class="w-full truncate">{app.name}</Item.Title>
                  <Item.Description class="truncate font-mono text-xs">
                    {app.executable}
                  </Item.Description>
                </Item.Content>
                <Item.Actions>
                  {#if runningIds.has(app.id)}
                    <Badge variant="secondary">Running</Badge>
                  {:else}
                    <Button variant="secondary" size="sm" onclick={() => (starting = app)}>
                      <PlayIcon data-icon="inline-start" />
                      Start
                    </Button>
                  {/if}
                </Item.Actions>
              </Item.Root>
            {/each}
            {#if results.length === RESULT_LIMIT}
              <p class="py-2 text-center text-xs text-muted-foreground">
                Showing the first {RESULT_LIMIT} matches. Refine the search to see more.
              </p>
            {/if}
          </Item.Group>
        {/if}
      </ScrollArea>
    </main>

    <Separator orientation="vertical" />

    <aside class="flex w-80 shrink-0 flex-col gap-3 p-4">
      <div class="flex items-center gap-2">
        <h2 class="text-sm font-medium">Running</h2>
        {#if simulations.length > 0}
          <Badge variant="secondary">{simulations.length}</Badge>
        {/if}
      </div>
      <ScrollArea class="min-h-0 flex-1">
        {#if simulations.length === 0}
          <p class="text-sm text-muted-foreground">Nothing is running.</p>
        {:else}
          <div class="flex flex-col gap-2">
            {#each simulations as sim (sim.appId)}
              <RunningCard
                simulation={sim}
                iconHash={appsById.get(sim.appId)?.iconHash ?? null}
                {now}
                onstop={() => stop(sim.appId)}
                onedittimer={() => (editingTimer = sim)}
              />
            {/each}
          </div>
        {/if}
      </ScrollArea>
    </aside>
  </div>
</div>

<StartDialog app={starting} onclose={() => (starting = null)} onstart={start} />
<TimerDialog simulation={editingTimer} onclose={() => (editingTimer = null)} onset={setTimer} />
