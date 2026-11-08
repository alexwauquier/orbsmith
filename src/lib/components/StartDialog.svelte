<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Input } from "$lib/components/ui/input";
  import { Spinner } from "$lib/components/ui/spinner";
  import AppIcon from "$lib/components/AppIcon.svelte";
  import { DETECTION_DELAY_SECS, MAX_TIMER_MINUTES, type AppSummary } from "$lib/api";

  let {
    app,
    onclose,
    onstart
  }: {
    app: AppSummary | null;
    onclose: () => void;
    /** Resolves once the simulation started; the dialog stays open on error. */
    onstart: (app: AppSummary, durationSecs: number | undefined) => Promise<void>;
  } = $props();

  const maxQuestMinutes = Math.floor((MAX_TIMER_MINUTES * 60 - DETECTION_DELAY_SECS) / 60);

  let questMinutes = $state(15);
  let autoStop = $state(true);
  let starting = $state(false);

  const valid = $derived(
    !autoStop ||
      (Number.isInteger(questMinutes) && questMinutes >= 1 && questMinutes <= maxQuestMinutes)
  );

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!app || !valid) return;
    starting = true;
    try {
      await onstart(app, autoStop ? questMinutes * 60 + DETECTION_DELAY_SECS : undefined);
    } finally {
      starting = false;
    }
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
            <Field.FieldLabel for="quest-minutes">Quest duration (minutes)</Field.FieldLabel>
            <Input
              id="quest-minutes"
              type="number"
              min={1}
              max={maxQuestMinutes}
              step={1}
              bind:value={questMinutes}
              disabled={!autoStop}
              aria-invalid={!valid || undefined}
            />
            <Field.FieldDescription>
              {#if autoStop}
                Stops after {questMinutes || 0} minutes.
              {:else}
                Runs until you stop it.
              {/if}
            </Field.FieldDescription>
          </Field.Field>
        </Field.FieldGroup>

        <Dialog.Footer>
          <Button type="button" variant="outline" onclick={onclose}>Cancel</Button>
          <Button type="submit" disabled={!valid || starting}>
            {#if starting}
              <Spinner data-icon="inline-start" />
            {/if}
            Start
          </Button>
        </Dialog.Footer>
      </form>
    {/if}
  </Dialog.Content>
</Dialog.Root>
