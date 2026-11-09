<script lang="ts">
  import { onMount } from "svelte";
  import * as Alert from "$lib/components/ui/alert";
  import * as Empty from "$lib/components/ui/empty";
  import * as InputGroup from "$lib/components/ui/input-group";
  import * as Item from "$lib/components/ui/item";
  import * as Tabs from "$lib/components/ui/tabs";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Separator } from "$lib/components/ui/separator";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Spinner } from "$lib/components/ui/spinner";
  import AppRow from "$lib/components/AppRow.svelte";
  import QueuePanel from "$lib/components/QueuePanel.svelte";
  import RunningCard from "$lib/components/RunningCard.svelte";
  import SettingsDialog from "$lib/components/SettingsDialog.svelte";
  import StartDialog from "$lib/components/StartDialog.svelte";
  import TimerDialog from "$lib/components/TimerDialog.svelte";
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import GamepadIcon from "@lucide/svelte/icons/gamepad-2";
  import RefreshIcon from "@lucide/svelte/icons/refresh-cw";
  import SearchIcon from "@lucide/svelte/icons/search";
  import SearchXIcon from "@lucide/svelte/icons/search-x";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import XIcon from "@lucide/svelte/icons/x";
  import {
    getApps,
    listSimulations,
    addToQueue,
    getSettings,
    listQueue,
    onQueueChanged,
    onSimulationsChanged,
    removeFromQueue,
    setSettings,
    startQueue,
    stopQueue,
    setSimulationTimer,
    startSimulation,
    stopSimulation,
    type AppSummary,
    type QueueState,
    type Settings,
    type Simulation
  } from "$lib/api";
  import { resolveApps } from "$lib/library";
  import { LibraryStore } from "$lib/library.svelte";
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
  let queue = $state.raw<QueueState>({ running: false, active: [], queued: [] });
  let settings = $state.raw<Settings>({ parallelQueue: false });
  let settingsOpen = $state(false);
  let sideTab = $state("running");

  const results = $derived(searchApps(apps, query, RESULT_LIMIT));
  const appsById = $derived(new Map(apps.map((app) => [app.id, app])));
  const runningIds = $derived(new Set(simulations.map((sim) => sim.appId)));

  const queueCount = $derived(queue.active.length + queue.queued.length);

  const library = new LibraryStore();
  const favoriteApps = $derived(resolveApps(library.favorites, appsById));
  const recentApps = $derived(
    resolveApps(
      library.recent.filter((id) => !library.isFavorite(id)),
      appsById
    )
  );

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
      library.markUsed(app.id);
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

  async function reloadQueue() {
    await attempt(async () => {
      queue = await listQueue();
    });
  }

  async function enqueue(app: AppSummary, durationSecs: number) {
    const ok = await attempt(async () => {
      queue = await addToQueue(app.id, durationSecs);
    });
    if (ok) {
      starting = null;
      library.markUsed(app.id);
      sideTab = "queue";
    }
  }

  async function changeSettings(next: Settings) {
    await attempt(async () => {
      settings = await setSettings(next);
    });
  }

  onMount(() => {
    void Promise.all([
      loadApps(false),
      reloadSimulations(),
      reloadQueue(),
      attempt(async () => {
        settings = await getSettings();
      })
    ]).finally(() => (loading = false));

    const tick = setInterval(() => (now = Date.now()), 1000);
    const listeners = [
      onSimulationsChanged(() => void reloadSimulations()),
      onQueueChanged(() => void reloadQueue())
    ];
    return () => {
      clearInterval(tick);
      for (const listener of listeners) void listener.then((stopListening) => stopListening());
    };
  });
</script>

{#snippet row(app: AppSummary)}
  <AppRow
    {app}
    running={runningIds.has(app.id)}
    favorite={library.isFavorite(app.id)}
    onstart={() => (starting = app)}
    ontogglefavorite={() => library.toggleFavorite(app.id)}
  />
{/snippet}

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
      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="Settings"
        onclick={() => (settingsOpen = true)}
      >
        <SettingsIcon />
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
          {#if favoriteApps.length === 0 && recentApps.length === 0}
            <Empty.Root>
              <Empty.Header>
                <Empty.Media variant="icon"><GamepadIcon /></Empty.Media>
                <Empty.Title>Find a game</Empty.Title>
                <Empty.Description>
                  Search by name, then start it to make it appear as running. Games you start or
                  star show up here.
                </Empty.Description>
              </Empty.Header>
            </Empty.Root>
          {:else}
            <div class="flex flex-col gap-4">
              {#if favoriteApps.length > 0}
                <section class="flex flex-col gap-1" aria-labelledby="favorites-heading">
                  <h2 id="favorites-heading" class="px-1 text-xs font-medium text-muted-foreground">
                    Favorites
                  </h2>
                  <Item.Group class="gap-1">
                    {#each favoriteApps as app (app.id)}
                      {@render row(app)}
                    {/each}
                  </Item.Group>
                </section>
              {/if}
              {#if recentApps.length > 0}
                <section class="flex flex-col gap-1" aria-labelledby="recent-heading">
                  <h2 id="recent-heading" class="px-1 text-xs font-medium text-muted-foreground">
                    Recent
                  </h2>
                  <Item.Group class="gap-1">
                    {#each recentApps as app (app.id)}
                      {@render row(app)}
                    {/each}
                  </Item.Group>
                </section>
              {/if}
            </div>
          {/if}
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
              {@render row(app)}
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

    <aside class="flex w-80 shrink-0 flex-col p-4">
      <Tabs.Root bind:value={sideTab} class="min-h-0 flex-1">
        <Tabs.List class="w-full">
          <Tabs.Trigger value="running">
            Running
            {#if simulations.length > 0}
              <Badge variant="secondary">{simulations.length}</Badge>
            {/if}
          </Tabs.Trigger>
          <Tabs.Trigger value="queue">
            Queue
            {#if queueCount > 0}
              <Badge variant="secondary">{queueCount}</Badge>
            {/if}
          </Tabs.Trigger>
        </Tabs.List>
        <Tabs.Content value="running" class="min-h-0">
          <ScrollArea class="h-full">
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
        </Tabs.Content>
        <Tabs.Content value="queue" class="min-h-0">
          <ScrollArea class="h-full">
            <QueuePanel
              {queue}
              parallel={settings.parallelQueue}
              {appsById}
              onstart={() =>
                void attempt(async () => {
                  queue = await startQueue();
                })}
              onstop={() =>
                void attempt(async () => {
                  queue = await stopQueue();
                })}
              onremove={(appId) =>
                void attempt(async () => {
                  queue = await removeFromQueue(appId);
                })}
            />
          </ScrollArea>
        </Tabs.Content>
      </Tabs.Root>
    </aside>
  </div>
</div>

<StartDialog app={starting} onclose={() => (starting = null)} onstart={start} onenqueue={enqueue} />
<SettingsDialog
  bind:open={settingsOpen}
  {settings}
  onchange={(next) => void changeSettings(next)}
/>
<TimerDialog simulation={editingTimer} onclose={() => (editingTimer = null)} onset={setTimer} />
