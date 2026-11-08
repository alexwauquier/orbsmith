<script lang="ts">
  import * as Avatar from "$lib/components/ui/avatar";
  import { iconUrl, type AppSummary } from "$lib/api";
  import { cn } from "$lib/utils";

  let {
    app,
    class: className
  }: { app: Pick<AppSummary, "id" | "name" | "iconHash">; class?: string } = $props();

  const src = $derived(iconUrl(app, 64));
  const initials = $derived(
    app.name
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((word) => word[0]?.toUpperCase())
      .join("")
  );
</script>

<Avatar.Root class={cn("rounded-lg after:rounded-lg", className)}>
  {#if src}
    <Avatar.Image {src} alt="" class="rounded-lg" />
  {/if}
  <Avatar.Fallback class="rounded-lg">{initials}</Avatar.Fallback>
</Avatar.Root>
