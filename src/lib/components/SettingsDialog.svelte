<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Spinner } from "$lib/components/ui/spinner";
  import { Switch } from "$lib/components/ui/switch";
  import { MAX_GAP_MINUTES, type Settings } from "$lib/api";

  let {
    open = $bindable(false),
    settings,
    onchange,
    checking,
    updateStatus,
    oncheck
  }: {
    open?: boolean;
    settings: Settings;
    /** Saves the new settings; the dialog shows the saved values. */
    onchange: (settings: Settings) => void;
    /** Whether an update check is in progress. */
    checking: boolean;
    /** The result of the last check started from here, if any. */
    updateStatus: string | null;
    oncheck: () => void;
  } = $props();

  const gapMinutes = $derived(Math.round(settings.queueGapSecs / 60));

  /** Saves a valid wait; anything else puts the saved value back. */
  function changeGap(input: HTMLInputElement) {
    const minutes = input.valueAsNumber;
    if (Number.isInteger(minutes) && minutes >= 0 && minutes <= MAX_GAP_MINUTES) {
      onchange({ ...settings, queueGapSecs: minutes * 60 });
    } else {
      input.value = String(gapMinutes);
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-h-[calc(100vh-2rem)] overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Settings</Dialog.Title>
      <Dialog.Description>Changes are saved right away.</Dialog.Description>
    </Dialog.Header>

    <Field.FieldGroup>
      <Field.FieldSet>
        <Field.FieldLegend>Queue</Field.FieldLegend>
        <Field.FieldGroup>
          <Field.Field orientation="horizontal">
            <Field.FieldContent>
              <Field.FieldLabel for="parallel-queue">Run the queue in parallel</Field.FieldLabel>
              <Field.FieldDescription>
                Start every game in the queue at once instead of one after another. Each one still
                stops after its own duration.
              </Field.FieldDescription>
            </Field.FieldContent>
            <Switch
              id="parallel-queue"
              checked={settings.parallelQueue}
              onCheckedChange={(parallelQueue) => onchange({ ...settings, parallelQueue })}
            />
          </Field.Field>
          <Field.Field orientation="horizontal" data-disabled={settings.parallelQueue || undefined}>
            <Field.FieldContent>
              <Field.FieldLabel for="queue-gap">Wait between games (minutes)</Field.FieldLabel>
              <Field.FieldDescription>
                Pause after each game before starting the next one. Only when the queue runs one
                after another.
              </Field.FieldDescription>
            </Field.FieldContent>
            <Input
              id="queue-gap"
              type="number"
              class="w-20"
              min={0}
              max={MAX_GAP_MINUTES}
              step={1}
              value={gapMinutes}
              disabled={settings.parallelQueue}
              onchange={(event) => changeGap(event.currentTarget)}
            />
          </Field.Field>
          <Field.Field orientation="horizontal">
            <Field.FieldContent>
              <Field.FieldLabel for="vary-queue-times">Vary times</Field.FieldLabel>
              <Field.FieldDescription>
                Each game runs up to 10% longer than its duration, never shorter, and each wait
                between games is up to 50% shorter or longer.
              </Field.FieldDescription>
            </Field.FieldContent>
            <Switch
              id="vary-queue-times"
              checked={settings.varyQueueTimes}
              onCheckedChange={(varyQueueTimes) => onchange({ ...settings, varyQueueTimes })}
            />
          </Field.Field>
        </Field.FieldGroup>
      </Field.FieldSet>

      <Field.FieldSeparator />

      <Field.FieldSet>
        <Field.FieldLegend>Updates</Field.FieldLegend>
        <Field.FieldGroup>
          <Field.Field orientation="horizontal">
            <Field.FieldContent>
              <Field.FieldLabel for="check-for-updates"
                >Check for updates at startup</Field.FieldLabel
              >
              <Field.FieldDescription>
                Look for a new version on GitHub when Orbsmith starts. Updates are only installed
                when you choose to.
              </Field.FieldDescription>
            </Field.FieldContent>
            <Switch
              id="check-for-updates"
              checked={settings.checkForUpdates}
              onCheckedChange={(checkForUpdates) => onchange({ ...settings, checkForUpdates })}
            />
          </Field.Field>
          <div class="flex items-center gap-3">
            <Button variant="outline" size="sm" disabled={checking} onclick={oncheck}>
              {#if checking}
                <Spinner />
              {/if}
              Check now
            </Button>
            {#if updateStatus}
              <span class="text-sm text-muted-foreground" role="status">{updateStatus}</span>
            {/if}
          </div>
        </Field.FieldGroup>
      </Field.FieldSet>
    </Field.FieldGroup>
  </Dialog.Content>
</Dialog.Root>
