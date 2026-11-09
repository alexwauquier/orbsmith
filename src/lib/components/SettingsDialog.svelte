<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { Switch } from "$lib/components/ui/switch";
  import type { Settings } from "$lib/api";

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
</script>

<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Settings</Dialog.Title>
      <Dialog.Description>Changes are saved right away.</Dialog.Description>
    </Dialog.Header>

    <Field.FieldGroup>
      <Field.Field orientation="horizontal">
        <Field.FieldContent>
          <Field.FieldLabel for="parallel-queue">Run the queue in parallel</Field.FieldLabel>
          <Field.FieldDescription>
            Start every game in the queue at once instead of one after another. Each one still stops
            after its own duration.
          </Field.FieldDescription>
        </Field.FieldContent>
        <Switch
          id="parallel-queue"
          checked={settings.parallelQueue}
          onCheckedChange={(parallelQueue) => onchange({ ...settings, parallelQueue })}
        />
      </Field.Field>
      <Field.FieldSeparator />
      <Field.Field orientation="horizontal">
        <Field.FieldContent>
          <Field.FieldLabel for="check-for-updates">Check for updates at startup</Field.FieldLabel>
          <Field.FieldDescription>
            Look for a new version on GitHub when Orbsmith starts. Updates are only installed when
            you choose to.
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
  </Dialog.Content>
</Dialog.Root>
