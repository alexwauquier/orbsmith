<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Switch } from "$lib/components/ui/switch";
  import type { Settings } from "$lib/api";

  let {
    open = $bindable(false),
    settings,
    onchange
  }: {
    open?: boolean;
    settings: Settings;
    /** Saves the new settings; the dialog shows the saved values. */
    onchange: (settings: Settings) => void;
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
    </Field.FieldGroup>
  </Dialog.Content>
</Dialog.Root>
