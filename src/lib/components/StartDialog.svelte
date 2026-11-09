<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Input } from "$lib/components/ui/input";
  import { Spinner } from "$lib/components/ui/spinner";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import ListPlusIcon from "@lucide/svelte/icons/list-plus";
  import { DETECTION_DELAY_SECS, MAX_TIMER_MINUTES, type AppSummary } from "$lib/api";

  let {
    app,
    onclose,
    onstart,
    onenqueue
  }: {
    app: AppSummary | null;
    onclose: () => void;
    /** Resolves once the simulation started; the dialog stays open on error. */
    onstart: (app: AppSummary, durationSecs: number | undefined) => Promise<void>;
    /** Resolves once the game is queued; the dialog stays open on error. */
    onenqueue: (app: AppSummary, durationSecs: number) => Promise<void>;
  } = $props();

  const maxMinutes = Math.floor((MAX_TIMER_MINUTES * 60 - DETECTION_DELAY_SECS) / 60);

  let minutes = $state(15);
  let autoStop = $state(true);
  let busy = $state(false);

  const valid = $derived(
    !autoStop || (Number.isInteger(minutes) && minutes >= 1 && minutes <= maxMinutes)
  );

  const durationSecs = $derived(minutes * 60 + DETECTION_DELAY_SECS);

  async function run(action: () => Promise<void>) {
    busy = true;
    try {
      await action();
    } finally {
      busy = false;
    }
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const target = app;
    if (!target || !valid) return;
    void run(() => onstart(target, autoStop ? durationSecs : undefined));
  }

  function addToQueue() {
    const target = app;
    if (!target || !valid || !autoStop) return;
    void run(() => onenqueue(target, durationSecs));
  }
</script>

<Dialog.Root
  bind:open={
    () => app !== null,
    (open) => {
      if (!open) onclose();
    }
  }
>
  <Dialog.Content>
    {#if app}
      <form class="flex flex-col gap-6" onsubmit={submit}>
        <Dialog.Header class="flex-row items-center gap-3">
          <AppIcon {app} class="size-10" />
          <div class="flex min-w-0 flex-col gap-1">
            <Dialog.Title class="truncate">{app.name}</Dialog.Title>
            <Dialog.Description class="truncate font-mono text-xs">
              {app.executable}
            </Dialog.Description>
          </div>
        </Dialog.Header>

        <Field.FieldGroup>
          <Field.Field orientation="horizontal">
            <Checkbox id="auto-stop" bind:checked={autoStop} />
            <Field.FieldLabel for="auto-stop">Stop automatically</Field.FieldLabel>
          </Field.Field>
          <Field.Field data-invalid={!valid || undefined} data-disabled={!autoStop || undefined}>
            <Field.FieldLabel for="duration-minutes">Duration (minutes)</Field.FieldLabel>
            <Input
              id="duration-minutes"
              type="number"
              min={1}
              max={maxMinutes}
              step={1}
              bind:value={minutes}
              disabled={!autoStop}
              aria-invalid={!valid || undefined}
            />
            <Field.FieldDescription>
              {#if autoStop}
                Stops after {minutes || 0} minutes.
              {:else}
                Runs until you stop it.
              {/if}
            </Field.FieldDescription>
          </Field.Field>
        </Field.FieldGroup>

        <Dialog.Footer>
          <Button type="button" variant="outline" onclick={onclose}>Cancel</Button>
          <!-- A queued game needs a duration, so it requires the auto-stop timer. -->
          <Button
            type="button"
            variant="secondary"
            disabled={!valid || !autoStop || busy}
            onclick={addToQueue}
          >
            <ListPlusIcon data-icon="inline-start" />
            Add to queue
          </Button>
          <Button type="submit" disabled={!valid || busy}>
            {#if busy}
              <Spinner data-icon="inline-start" />
            {/if}
            Start now
          </Button>
        </Dialog.Footer>
      </form>
    {/if}
  </Dialog.Content>
</Dialog.Root>
