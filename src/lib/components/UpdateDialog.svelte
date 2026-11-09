<script lang="ts">
  import * as Alert from "$lib/components/ui/alert";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import type { UpdateInfo } from "$lib/api";
  import { parseReleaseNotes } from "$lib/release-notes";

  let {
    update,
    onclose,
    oninstall
  }: {
    update: UpdateInfo | null;
    onclose: () => void;
    /** Settles only if the update fails: Orbsmith restarts otherwise. */
    oninstall: () => Promise<void>;
  } = $props();

  let installing = $state(false);
  let error = $state<string | null>(null);

  const sections = $derived(parseReleaseNotes(update?.notes ?? ""));

  async function install() {
    installing = true;
    error = null;
    try {
      await oninstall();
    } catch (err) {
      error = typeof err === "string" ? err : err instanceof Error ? err.message : String(err);
    } finally {
      installing = false;
    }
  }
</script>

<Dialog.Root
  bind:open={
    () => update !== null,
    (open) => {
      if (!open && !installing) onclose();
    }
  }
>
  <Dialog.Content>
    {#if update}
      <Dialog.Header>
        <Dialog.Title>Orbsmith {update.version} is available</Dialog.Title>
        <Dialog.Description>You have version {update.currentVersion}.</Dialog.Description>
      </Dialog.Header>

      {#if sections.length > 0}
        <div class="flex max-h-64 flex-col gap-3 overflow-y-auto text-sm">
          {#each sections as section, i (i)}
            <section class="flex flex-col gap-1">
              {#if section.title}
                <h3 class="font-medium">{section.title}</h3>
              {/if}
              <ul class="flex list-disc flex-col gap-1 pl-5 text-muted-foreground">
                {#each section.items as item, j (j)}
                  <li>{item}</li>
                {/each}
              </ul>
            </section>
          {/each}
        </div>
      {/if}

      <p class="text-sm text-muted-foreground">
        Running games stop and the queue is paused with its time left. Orbsmith restarts once the
        update is installed. Your system may ask for your password.
      </p>

      {#if error}
        <Alert.Root variant="destructive">
          <CircleAlertIcon />
          <Alert.Title>The update failed</Alert.Title>
          <Alert.Description>{error}</Alert.Description>
        </Alert.Root>
      {/if}

      <Dialog.Footer>
        <Button variant="outline" disabled={installing} onclick={onclose}>Later</Button>
        <Button disabled={installing} onclick={install}>
          {#if installing}
            <Spinner />
            Updating…
          {:else}
            Update and restart
          {/if}
        </Button>
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>
