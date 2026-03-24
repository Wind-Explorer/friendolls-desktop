<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Snippet } from "svelte";
  import { setSprite } from "./sprites";
  import { calculateDirection, moveTowards, clampPosition } from "./physics";
  import { updateIdle } from "./idle";
  import { appState } from "../../../../events/app-state";

  interface Props {
    targetX: number;
    targetY: number;
    spriteUrl: string;
    initialX?: number;
    initialY?: number;
    scale?: number;
    opacity?: number;
    children?: Snippet;
  }

  let {
    targetX,
    targetY,
    spriteUrl,
    initialX = 32,
    initialY = 32,
    scale = 1.0,
    opacity = 1.0,
    children,
  }: Props = $props();

  let nekoEl: HTMLDivElement;
  let wrapperEl: HTMLDivElement;
  let animationFrameId: number;

  let nekoPos = $state({ x: initialX, y: initialY });
  let frameCount = 0;
  let idleTime = 0;
  let idleAnimation: string | null = $state(null);
  let idleAnimationFrame = 0;

  let lastFrameTimestamp: number | null = null;

  function frame(timestamp: number) {
    if (!lastFrameTimestamp) {
      lastFrameTimestamp = timestamp;
    }
    if (timestamp - lastFrameTimestamp > 100) {
      lastFrameTimestamp = timestamp;
      doFrame();
    }
    animationFrameId = requestAnimationFrame(frame);
  }

  function doFrame() {
    frameCount += 1;
    const targetPos = { x: targetX, y: targetY };

    const { direction, distance } = calculateDirection(
      nekoPos.x,
      nekoPos.y,
      targetPos.x,
      targetPos.y,
    );

    if (distance < 10 || distance < 48) {
      const idleResult = updateIdle(
        nekoEl,
        nekoPos,
        targetPos,
        idleAnimation,
        idleAnimationFrame,
        idleTime,
      );
      idleAnimation = idleResult.idleAnimation;
      idleAnimationFrame = idleResult.idleAnimationFrame;
      idleTime = idleResult.idleTime;
      return;
    }

    idleAnimation = null;
    idleAnimationFrame = 0;

    if (idleTime > 1) {
      setSprite(nekoEl, "alert", 0);
      idleTime = Math.min(idleTime, 7);
      idleTime -= 1;
      return;
    }

    setSprite(nekoEl, direction, frameCount);

    const newPos = moveTowards(nekoPos.x, nekoPos.y, targetPos.x, targetPos.y);
    nekoPos = newPos;

    nekoEl.style.transform = `scale(${scale ?? 1.0})`;
    nekoEl.style.opacity = `${opacity ?? 1.0}`;
    wrapperEl.style.transform = `translate(${nekoPos.x - 16}px, ${nekoPos.y - 16}px)`;
  }

  onMount(() => {
    nekoEl.style.backgroundImage = `url(${spriteUrl})`;
    nekoEl.style.opacity = `${opacity ?? 1.0}`;
    wrapperEl.style.transform = `translate(${nekoPos.x - 16}px, ${nekoPos.y - 16}px)`;
    animationFrameId = requestAnimationFrame(frame);
  });

  onDestroy(() => {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
  });

  $effect(() => {
    if (nekoEl && spriteUrl && $appState) {
      nekoEl.style.transform = `scale(${scale ?? 1.0})`;
      nekoEl.style.backgroundImage = `url(${spriteUrl})`;
      nekoEl.style.opacity = `${opacity ?? 1.0}`;
    }
  });
</script>

<div
  bind:this={wrapperEl}
  class="pointer-events-none fixed z-999 size-8 select-none"
  style="position: fixed; width: 32px; height: 32px;"
>
  <div class="relative">
    <div
      bind:this={nekoEl}
      class="size-8"
      style="position: absolute; image-rendering: pixelated;"
    ></div>
    <div class="absolute size-8">
      {@render children?.()}
    </div>
  </div>
</div>
