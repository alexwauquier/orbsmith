<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Field from "$lib/components/ui/field";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { MAX_TIMER_MINUTES, type Simulation } from "$lib/api";

  let {
    simulation,
    onclose,
    onset
  }: {
    simulation: Simulation | null;
    onclose: () => void;
    /** `null` removes the timer. */
    onset: (appId: string, durationSecs: number | null) => Promise<void>;
  } = $props();

  let minutes = $state(10);

  const valid = $derived(Number.isInteger(minutes) && minutes >= 1 && minutes <= MAX_TIMER_MINUTES);

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (simulation && valid) void onset(simulation.appId, minutes * 60);
  }
</script>

<Dialog.Root
  bind:open={
    () => simulation !== null,
    (open) => {
      if (!open) onclose();
    }
  }
>
  <Dialog.Content>
    {#if simulation}
      <form class="flex flex-col gap-6" onsubmit={submit}>
        <Dialog.Header>
          <Dialog.Title>Auto-stop timer</Dialog.Title>
          <Dialog.Description>{simulation.appName}</Dialog.Description>
        </Dialog.Header>

        <Field.Field data-invalid={!valid || undefined}>
          <Field.FieldLabel for="timer-minutes">Stop in (minutes)</Field.FieldLabel>
          <Input
            id="timer-minutes"
            type="number"
            min={1}
            max={MAX_TIMER_MINUTES}
            step={1}
            bind:value={minutes}
            aria-invalid={!valid || undefined}
          />
        </Field.Field>

        <Dialog.Footer>
          {#if simulation.stopAt !== null}
            <Button
              type="button"
              variant="ghost"
              class="sm:mr-auto"
              onclick={() => simulation && onset(simulation.appId, null)}
            >
              Remove timer
            </Button>
          {/if}
          <Button type="button" variant="outline" onclick={onclose}>Cancel</Button>
          <Button type="submit" disabled={!valid}>Set timer</Button>
        </Dialog.Footer>
      </form>
    {/if}
  </Dialog.Content>
</Dialog.Root>
